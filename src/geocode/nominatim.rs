//! Nominatim HTTP client — forward and reverse geocoding with IATA airport
//! fallback via multi-strategy name cleaning.

use super::{airports, sanitize};
use serde::Deserialize;
use std::time::Duration;

const NOMINATIM_SEARCH_URL: &str = "https://nominatim.openstreetmap.org/search";
const NOMINATIM_REVERSE_URL: &str = "https://nominatim.openstreetmap.org/reverse";
const USER_AGENT: &str = "TravelMapper/1.0 (github.com/butlerx/travel-export)";

#[derive(Debug, Deserialize)]
struct NominatimResult {
    lat: String,
    lon: String,
}

#[derive(Debug, Deserialize)]
struct NominatimReverseResult {
    address: Option<NominatimAddress>,
}

#[derive(Debug, Deserialize)]
struct NominatimAddress {
    country_code: Option<String>,
}

/// Nominatim-backed geocoder with embedded IATA airport fallback.
pub struct Geocoder {
    pub(super) client: reqwest::Client,
}

impl Geocoder {
    /// Create a geocoder with a custom HTTP client.
    #[must_use]
    pub fn with_client(client: reqwest::Client) -> Self {
        Self { client }
    }

    /// Forward-geocode a location string via Nominatim, optionally scoped to a
    /// country code. Returns latitude/longitude or `None` on failure.
    pub async fn geocode(&self, query: &str, country_code: Option<&str>) -> Option<(f64, f64)> {
        let mut request = self
            .client
            .get(NOMINATIM_SEARCH_URL)
            .header(reqwest::header::USER_AGENT, USER_AGENT)
            .query(&[("q", query), ("format", "json"), ("limit", "1")]);

        if let Some(code) = country_code {
            request = request.query(&[("countrycodes", code)]);
        }

        let response = request.send().await;
        tokio::time::sleep(Duration::from_secs(1)).await;

        let response = match response {
            Ok(response) => response,
            Err(error) => {
                tracing::warn!(query, country_code, %error, "nominatim request failed");
                return None;
            }
        };

        let candidates = match response.json::<Vec<NominatimResult>>().await {
            Ok(candidates) => candidates,
            Err(error) => {
                tracing::warn!(query, country_code, %error, "nominatim response parse failed");
                return None;
            }
        };

        let Some(first) = candidates.first() else {
            tracing::warn!(query, country_code, "nominatim returned no results");
            return None;
        };

        let lat = match first.lat.parse::<f64>() {
            Ok(lat) => lat,
            Err(error) => {
                tracing::warn!(query, country_code, %error, "failed to parse nominatim latitude");
                return None;
            }
        };

        let lon = match first.lon.parse::<f64>() {
            Ok(lon) => lon,
            Err(error) => {
                tracing::warn!(query, country_code, %error, "failed to parse nominatim longitude");
                return None;
            }
        };

        tracing::debug!(query, country_code, lat, lon, "nominatim geocode resolved");
        Some((lat, lon))
    }

    pub(super) async fn geocode_with_fallbacks(
        &self,
        name: &str,
        country_code: Option<&str>,
        address_query: Option<&str>,
    ) -> Option<(f64, f64)> {
        if let Some(iata) = sanitize::extract_iata_code(name)
            && let Some(coords) = airports::lookup(iata)
        {
            tracing::debug!(name, iata, "resolved via embedded IATA code");
            return Some(coords);
        }

        if let Some(coords) = self.geocode(name, country_code).await {
            return Some(coords);
        }

        let sanitized = sanitize::sanitize_location_name(name);
        if sanitized != name
            && !sanitized.is_empty()
            && let Some(coords) = self.geocode(&sanitized, country_code).await
        {
            tracing::debug!(name, sanitized, "resolved via sanitized name");
            return Some(coords);
        }

        if let Some(city) = sanitize::extract_city_segment(name)
            && let Some(coords) = self.geocode(city, country_code).await
        {
            tracing::debug!(name, city, "resolved via city segment fallback");
            return Some(coords);
        }

        if country_code.is_some()
            && let Some(coords) = self.geocode(name, None).await
        {
            tracing::debug!(name, "resolved by dropping country_code filter");
            return Some(coords);
        }

        if let Some(addr) = address_query
            && let Some(coords) = self.geocode(addr, None).await
        {
            tracing::debug!(name, addr, "resolved via address query");
            return Some(coords);
        }

        None
    }

    pub(super) async fn reverse_geocode(&self, lat: f64, lng: f64) -> Option<String> {
        let lat_str = lat.to_string();
        let lng_str = lng.to_string();
        let response = self
            .client
            .get(NOMINATIM_REVERSE_URL)
            .header(reqwest::header::USER_AGENT, USER_AGENT)
            .query(&[
                ("lat", &lat_str),
                ("lon", &lng_str),
                ("format", &"json".to_string()),
            ])
            .send()
            .await;
        tokio::time::sleep(Duration::from_secs(1)).await;

        let response = match response {
            Ok(r) => r,
            Err(error) => {
                tracing::warn!(lat, lng, %error, "nominatim reverse request failed");
                return None;
            }
        };

        let result = match response.json::<NominatimReverseResult>().await {
            Ok(r) => r,
            Err(error) => {
                tracing::warn!(lat, lng, %error, "nominatim reverse response parse failed");
                return None;
            }
        };

        let cc = result.address?.country_code?;
        tracing::debug!(lat, lng, country_code = %cc, "nominatim reverse geocode resolved");
        Some(cc)
    }
}

impl Default for Geocoder {
    fn default() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

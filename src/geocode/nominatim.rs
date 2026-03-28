//! Nominatim geocoding via the [`nominatim`] crate — forward search and
//! reverse geocoding with IATA airport fallback via multi-strategy name
//! cleaning. Optionally backed by a SQLite cache to avoid repeated API calls.

use super::{airports, sanitize};
use crate::db;
use sqlx::SqlitePool;
use std::time::Duration;

const USER_AGENT: &str = "TravelMapper/1.0 (github.com/butlerx/travel-export)";

pub struct Geocoder {
    client: nominatim::Client,
    pool: Option<SqlitePool>,
}

impl Geocoder {
    #[must_use]
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            client: nominatim::Client::new(nominatim::IdentificationMethod::from_user_agent(
                USER_AGENT,
            )),
            pool: Some(pool),
        }
    }

    async fn geocode(&self, query: &str) -> Option<(f64, f64)> {
        if let Some(ref pool) = self.pool
            && let Ok(Some(cached)) = (db::geocode_cache::Get { query }).execute(pool).await
        {
            tracing::debug!(query, lat = cached.0, lng = cached.1, "geocode cache hit");
            return Some(cached);
        }

        let response = self.client.search(query).await;
        tokio::time::sleep(Duration::from_secs(1)).await;

        let candidates = match response {
            Ok(candidates) => candidates,
            Err(error) => {
                tracing::warn!(query, %error, "nominatim request failed");
                return None;
            }
        };

        let Some(first) = candidates.first() else {
            tracing::warn!(query, "nominatim returned no results");
            return None;
        };

        let lat = match first.lat.parse::<f64>() {
            Ok(lat) => lat,
            Err(error) => {
                tracing::warn!(query, %error, "failed to parse nominatim latitude");
                return None;
            }
        };

        let lon = match first.lon.parse::<f64>() {
            Ok(lon) => lon,
            Err(error) => {
                tracing::warn!(query, %error, "failed to parse nominatim longitude");
                return None;
            }
        };

        tracing::debug!(query, lat, lon, "nominatim geocode resolved");

        if let Some(ref pool) = self.pool
            && let Err(err) = (db::geocode_cache::Upsert {
                query,
                lat,
                lng: lon,
            })
            .execute(pool)
            .await
        {
            tracing::warn!(query, %err, "failed to cache geocode result");
        }

        Some((lat, lon))
    }

    pub(super) async fn geocode_with_fallbacks(
        &self,
        name: &str,
        address_query: Option<&str>,
    ) -> Option<(f64, f64)> {
        if let Some(iata) = sanitize::extract_iata_code(name)
            && let Some(airport) = airports::lookup_enriched(iata)
        {
            tracing::debug!(name, iata, "resolved via embedded IATA code");
            return Some((airport.latitude, airport.longitude));
        }

        if let Some(coords) = self.geocode(name).await {
            return Some(coords);
        }

        let sanitized = sanitize::sanitize_location_name(name);
        if sanitized != name
            && !sanitized.is_empty()
            && let Some(coords) = self.geocode(&sanitized).await
        {
            tracing::debug!(name, sanitized, "resolved via sanitized name");
            return Some(coords);
        }

        if let Some(city) = sanitize::extract_city_segment(name)
            && let Some(coords) = self.geocode(city).await
        {
            tracing::debug!(name, city, "resolved via city segment fallback");
            return Some(coords);
        }

        if let Some(addr) = address_query
            && let Some(coords) = self.geocode(addr).await
        {
            tracing::debug!(name, addr, "resolved via address query");
            return Some(coords);
        }

        None
    }
}

impl Default for Geocoder {
    fn default() -> Self {
        Self {
            client: nominatim::Client::new(nominatim::IdentificationMethod::from_user_agent(
                USER_AGENT,
            )),
            pool: None,
        }
    }
}

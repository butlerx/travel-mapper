pub mod airports;

use crate::db::hops::Row;
use regex::Regex;
use serde::Deserialize;
use std::sync::LazyLock;
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

pub struct Geocoder {
    client: reqwest::Client,
}

/// Extract a 3-letter IATA airport code from parenthesized text,
/// e.g. `"Marrakech Menara Airport (RAK)"` → `Some("RAK")`.
#[must_use]
fn extract_iata_code(name: &str) -> Option<&str> {
    static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\(([A-Z]{3})\)").unwrap());
    RE.captures(name).map(|c| c.get(1).unwrap().as_str())
}

/// Strip noisy substrings that confuse Nominatim: Irish eircodes, European
/// postcodes, and terminal identifiers.
#[must_use]
fn sanitize_location_name(name: &str) -> String {
    // D12 AN22, A96 K7W4, etc.
    static EIRCODE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"\b[A-Z]\d{2}\s?[A-Z]{2}\d{2}\b").unwrap());
    // 08820, 08004, 12345, etc.
    static POSTCODE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\b\d{4,5}\b").unwrap());
    // "Terminal 2A", "Terminal 1"
    static TERMINAL: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"(?i)\bTerminal\s+\w+\b").unwrap());

    let s = EIRCODE.replace_all(name, "");
    let s = POSTCODE.replace_all(&s, "");
    let s = TERMINAL.replace_all(&s, "");

    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Return the last comma-separated segment as a city fallback,
/// e.g. `"Carrer del Poeta 45, Barcelona"` → `Some("Barcelona")`.
#[must_use]
fn extract_city_segment(name: &str) -> Option<&str> {
    let last = name.rsplit(',').next()?.trim();
    if last.is_empty() || last == name.trim() {
        None
    } else {
        Some(last)
    }
}

/// Map an IANA timezone to its primary ISO 3166-1 alpha-2 country code.
///
/// Covers the 29 timezones observed in our `TripIt` data. Returns `None` for
/// unknown or ambiguous zones (e.g. `UTC`).
#[must_use]
fn tz_to_country(tz: &str) -> Option<&'static str> {
    match tz {
        "Africa/Casablanca" => Some("ma"),
        "America/Chicago" | "America/Denver" | "America/Los_Angeles" | "America/New_York" => {
            Some("us")
        }
        "America/Vancouver" => Some("ca"),
        "Asia/Dubai" => Some("ae"),
        "Asia/Nicosia" => Some("cy"),
        "Asia/Seoul" => Some("kr"),
        "Asia/Tokyo" => Some("jp"),
        "Europe/Amsterdam" => Some("nl"),
        "Europe/Athens" => Some("gr"),
        "Europe/Berlin" => Some("de"),
        "Europe/Bratislava" => Some("sk"),
        "Europe/Brussels" => Some("be"),
        "Europe/Copenhagen" => Some("dk"),
        "Europe/Dublin" => Some("ie"),
        "Europe/Istanbul" => Some("tr"),
        "Europe/Lisbon" => Some("pt"),
        "Europe/London" => Some("gb"),
        "Europe/Madrid" => Some("es"),
        "Europe/Malta" => Some("mt"),
        "Europe/Moscow" => Some("ru"),
        "Europe/Paris" => Some("fr"),
        "Europe/Prague" => Some("cz"),
        "Europe/Rome" => Some("it"),
        "Europe/Stockholm" => Some("se"),
        "Europe/Warsaw" => Some("pl"),
        "Europe/Zurich" => Some("ch"),
        _ => None,
    }
}

/// Pre-pass over hops that compares each timezone-derived country against the
/// stored country code. When they conflict the coordinates are zeroed so the
/// later geocoding loop will re-resolve them with the correct country hint.
fn apply_tz_sanity_check(hops: &mut [Row]) {
    for hop in hops.iter_mut() {
        if let Some(expected) = hop.origin_tz.as_deref().and_then(tz_to_country) {
            match hop.origin_country.as_deref() {
                Some(stored) if stored.eq_ignore_ascii_case(expected) => {}
                Some(stored) => {
                    tracing::info!(
                        origin = hop.origin_name,
                        stored_country = stored,
                        tz_country = expected,
                        tz = hop.origin_tz.as_deref().unwrap_or_default(),
                        "origin country/tz mismatch — clearing coords for re-geocode"
                    );
                    hop.origin_lat = 0.0;
                    hop.origin_lng = 0.0;
                    hop.origin_country = Some(expected.to_string());
                }
                None => {
                    hop.origin_country = Some(expected.to_string());
                }
            }
        }

        if let Some(expected) = hop.dest_tz.as_deref().and_then(tz_to_country) {
            match hop.dest_country.as_deref() {
                Some(stored) if stored.eq_ignore_ascii_case(expected) => {}
                Some(stored) => {
                    tracing::info!(
                        dest = hop.dest_name,
                        stored_country = stored,
                        tz_country = expected,
                        tz = hop.dest_tz.as_deref().unwrap_or_default(),
                        "dest country/tz mismatch — clearing coords for re-geocode"
                    );
                    hop.dest_lat = 0.0;
                    hop.dest_lng = 0.0;
                    hop.dest_country = Some(expected.to_string());
                }
                None => {
                    hop.dest_country = Some(expected.to_string());
                }
            }
        }
    }
}

impl Geocoder {
    #[must_use]
    pub fn with_client(client: reqwest::Client) -> Self {
        Self { client }
    }

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

    async fn geocode_with_fallbacks(
        &self,
        name: &str,
        country_code: Option<&str>,
        address_query: Option<&str>,
    ) -> Option<(f64, f64)> {
        if let Some(iata) = extract_iata_code(name)
            && let Some(coords) = airports::lookup(iata)
        {
            tracing::debug!(name, iata, "resolved via embedded IATA code");
            return Some(coords);
        }

        if let Some(coords) = self.geocode(name, country_code).await {
            return Some(coords);
        }

        let sanitized = sanitize_location_name(name);
        if sanitized != name
            && !sanitized.is_empty()
            && let Some(coords) = self.geocode(&sanitized, country_code).await
        {
            tracing::debug!(name, sanitized, "resolved via sanitized name");
            return Some(coords);
        }

        if let Some(city) = extract_city_segment(name)
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

    async fn reverse_geocode(&self, lat: f64, lng: f64) -> Option<String> {
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

    pub async fn resolve_trip_coords(&self, mut hops: Vec<Row>) -> Vec<Row> {
        let len = hops.len();

        apply_tz_sanity_check(&mut hops);

        for i in 0..len {
            if is_missing(hops[i].origin_lat, hops[i].origin_lng) {
                let origin_name = hops[i].origin_name.clone();
                if origin_name.is_empty() {
                    tracing::warn!(
                        hop_index = i,
                        travel_type = ?hops[i].travel_type,
                        "cannot geocode origin: name is empty"
                    );
                } else {
                    let country = self.infer_country_for_origin(&hops, i).await;
                    let addr = hops[i].origin_address_query.clone();
                    if let Some((lat, lng)) = self
                        .geocode_with_fallbacks(&origin_name, country.as_deref(), addr.as_deref())
                        .await
                    {
                        hops[i].origin_lat = lat;
                        hops[i].origin_lng = lng;
                    } else {
                        tracing::warn!(
                            hop_index = i,
                            travel_type = ?hops[i].travel_type,
                            name = origin_name,
                            "failed to resolve origin coordinates after all fallbacks"
                        );
                    }
                }
            }

            if is_missing(hops[i].dest_lat, hops[i].dest_lng) {
                let dest_name = hops[i].dest_name.clone();
                if dest_name.is_empty() {
                    tracing::warn!(
                        hop_index = i,
                        travel_type = ?hops[i].travel_type,
                        "cannot geocode destination: name is empty"
                    );
                } else {
                    let country = self.infer_country_for_dest(&hops, i).await;
                    let addr = hops[i].dest_address_query.clone();
                    if let Some((lat, lng)) = self
                        .geocode_with_fallbacks(&dest_name, country.as_deref(), addr.as_deref())
                        .await
                    {
                        hops[i].dest_lat = lat;
                        hops[i].dest_lng = lng;
                    } else {
                        tracing::warn!(
                            hop_index = i,
                            travel_type = ?hops[i].travel_type,
                            name = dest_name,
                            "failed to resolve destination coordinates after all fallbacks"
                        );
                    }
                }
            }
        }

        // Second pass: fill empty destinations from the next hop's origin.
        // This covers transport legs where TripIt left the destination blank —
        // the next segment's departure point is the same location.
        for i in 0..len.saturating_sub(1) {
            if is_missing(hops[i].dest_lat, hops[i].dest_lng)
                && hops[i].dest_name.is_empty()
                && !is_missing(hops[i + 1].origin_lat, hops[i + 1].origin_lng)
            {
                tracing::info!(
                    hop_index = i,
                    next_origin = hops[i + 1].origin_name,
                    "filling empty dest from next hop origin"
                );
                let (left, right) = hops.split_at_mut(i + 1);
                let current = &mut left[i];
                let next = &right[0];
                current.dest_name.clone_from(&next.origin_name);
                current.dest_lat = next.origin_lat;
                current.dest_lng = next.origin_lng;
                current.dest_country.clone_from(&next.origin_country);
            }
        }

        hops
    }

    async fn infer_country_for_origin(&self, hops: &[Row], idx: usize) -> Option<String> {
        if let Some(cc) = &hops[idx].origin_country {
            return Some(cc.clone());
        }
        if idx > 0 && !is_missing(hops[idx - 1].dest_lat, hops[idx - 1].dest_lng) {
            let cc = self
                .reverse_geocode(hops[idx - 1].dest_lat, hops[idx - 1].dest_lng)
                .await;
            if cc.is_some() {
                return cc;
            }
        }
        if !is_missing(hops[idx].dest_lat, hops[idx].dest_lng) {
            return self
                .reverse_geocode(hops[idx].dest_lat, hops[idx].dest_lng)
                .await;
        }
        None
    }

    async fn infer_country_for_dest(&self, hops: &[Row], idx: usize) -> Option<String> {
        if let Some(cc) = &hops[idx].dest_country {
            return Some(cc.clone());
        }
        if !is_missing(hops[idx].origin_lat, hops[idx].origin_lng) {
            let cc = self
                .reverse_geocode(hops[idx].origin_lat, hops[idx].origin_lng)
                .await;
            if cc.is_some() {
                return cc;
            }
        }
        if idx + 1 < hops.len() && !is_missing(hops[idx + 1].origin_lat, hops[idx + 1].origin_lng) {
            return self
                .reverse_geocode(hops[idx + 1].origin_lat, hops[idx + 1].origin_lng)
                .await;
        }
        None
    }
}

impl Default for Geocoder {
    fn default() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

fn is_missing(lat: f64, lng: f64) -> bool {
    (lat == 0.0) && (lng == 0.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::hops::TravelType;

    fn hop(
        origin_name: &str,
        origin_lat: f64,
        origin_lng: f64,
        dest_name: &str,
        dest_lat: f64,
        dest_lng: f64,
    ) -> Row {
        Row {
            travel_type: TravelType::Rail,
            origin_name: origin_name.to_string(),
            origin_lat,
            origin_lng,
            origin_country: None,
            dest_name: dest_name.to_string(),
            dest_lat,
            dest_lng,
            dest_country: None,
            start_date: "2025-01-01".to_string(),
            end_date: "2025-01-01".to_string(),
            raw_json: None,
            origin_address_query: None,
            dest_address_query: None,
            origin_tz: None,
            dest_tz: None,
            flight_detail: None,
            rail_detail: None,
            cruise_detail: None,
            transport_detail: None,
        }
    }

    #[test]
    fn is_missing_detects_zero_coords() {
        assert!(is_missing(0.0, 0.0));
        assert!(!is_missing(53.35, -6.29));
        assert!(!is_missing(0.0, -6.29));
        assert!(!is_missing(53.35, 0.0));
    }

    #[tokio::test]
    async fn resolve_trip_coords_skips_hops_with_valid_coords() {
        let geocoder = Geocoder::default();
        let hops = vec![hop("Seoul", 37.55, 126.97, "Busan", 35.11, 129.04)];

        let resolved = geocoder.resolve_trip_coords(hops).await;

        assert!((resolved[0].origin_lat - 37.55).abs() < f64::EPSILON);
        assert!((resolved[0].origin_lng - 126.97).abs() < f64::EPSILON);
        assert!((resolved[0].dest_lat - 35.11).abs() < f64::EPSILON);
        assert!((resolved[0].dest_lng - 129.04).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn infer_country_for_origin_prefers_hop_hint_over_reverse_geocode() {
        let geocoder = Geocoder::default();
        let mut marrakech = hop("Marrakech", 0.0, 0.0, "Essaouira", 0.0, 0.0);
        marrakech.origin_country = Some("ma".to_string());
        let hops = vec![
            hop("Dublin", 53.35, -6.25, "Belfast", 54.6, -5.93),
            marrakech,
        ];
        assert_eq!(
            geocoder.infer_country_for_origin(&hops, 1).await.as_deref(),
            Some("ma"),
            "hop's own origin_country hint should win over reverse geocoding neighbor coords"
        );
    }

    #[tokio::test]
    async fn infer_country_for_dest_prefers_hop_hint_over_reverse_geocode() {
        let geocoder = Geocoder::default();
        let mut dublin_to_essaouira = hop("Dublin", 53.35, -6.25, "Essaouira", 0.0, 0.0);
        dublin_to_essaouira.dest_country = Some("ma".to_string());
        let hops = vec![dublin_to_essaouira];
        assert_eq!(
            geocoder.infer_country_for_dest(&hops, 0).await.as_deref(),
            Some("ma"),
            "hop's own dest_country hint should win over reverse geocoding origin coords"
        );
    }

    #[tokio::test]
    async fn infer_country_for_origin_returns_none_without_context() {
        let geocoder = Geocoder::default();
        let hops = vec![hop("Mallow", 0.0, 0.0, "Westport", 0.0, 0.0)];
        assert_eq!(
            geocoder.infer_country_for_origin(&hops, 0).await,
            None,
            "no coords or hints available anywhere"
        );
    }

    #[tokio::test]
    async fn infer_country_for_dest_returns_none_without_context() {
        let geocoder = Geocoder::default();
        let hops = vec![hop("Dublin", 0.0, 0.0, "Mallow", 0.0, 0.0)];
        assert_eq!(
            geocoder.infer_country_for_dest(&hops, 0).await,
            None,
            "no coords or hints available anywhere"
        );
    }

    #[test]
    fn extract_iata_code_finds_parenthesized_code() {
        assert_eq!(
            extract_iata_code("Marrakech Menara Airport (RAK)"),
            Some("RAK")
        );
        assert_eq!(
            extract_iata_code("Barcelona El Prat Airport (BCN) Terminal 2"),
            Some("BCN")
        );
    }

    #[test]
    fn extract_iata_code_returns_none_for_no_match() {
        assert_eq!(extract_iata_code("Hotel Lobby"), None);
        assert_eq!(extract_iata_code("DUB"), None);
        assert_eq!(extract_iata_code("(ab)"), None);
        assert_eq!(extract_iata_code("(ABCD)"), None);
    }

    #[test]
    fn sanitize_location_name_strips_eircode() {
        assert_eq!(
            sanitize_location_name("Greenmount Avenue 1, Dublin 12 D12 AN22"),
            "Greenmount Avenue 1, Dublin 12"
        );
    }

    #[test]
    fn sanitize_location_name_strips_postcode() {
        assert_eq!(
            sanitize_location_name("El Prat de Llobregat 08820"),
            "El Prat de Llobregat"
        );
        assert_eq!(
            sanitize_location_name("Carrer del Poeta Cabanyes 45, Barcelona 08004"),
            "Carrer del Poeta Cabanyes 45, Barcelona"
        );
    }

    #[test]
    fn sanitize_location_name_strips_terminal() {
        assert_eq!(
            sanitize_location_name("Barcelona El Prat Airport Terminal 2A"),
            "Barcelona El Prat Airport"
        );
    }

    #[test]
    fn sanitize_location_name_handles_combined_noise() {
        assert_eq!(
            sanitize_location_name(
                "Barcelona El Prat Airport Terminal 2A El Prat de Llobregat 08820"
            ),
            "Barcelona El Prat Airport El Prat de Llobregat"
        );
    }

    #[test]
    fn extract_city_segment_returns_last_comma_part() {
        assert_eq!(
            extract_city_segment("Carrer del Poeta Cabanyes 45, Barcelona 08004"),
            Some("Barcelona 08004")
        );
        assert_eq!(
            extract_city_segment("Dar Najat, 30 Derb Lalla Chacha"),
            Some("30 Derb Lalla Chacha")
        );
    }

    #[test]
    fn extract_city_segment_returns_none_without_comma() {
        assert_eq!(extract_city_segment("Hotel Lobby"), None);
        assert_eq!(extract_city_segment(""), None);
    }

    #[tokio::test]
    async fn resolve_trip_coords_fills_empty_dest_from_next_hop_origin() {
        let geocoder = Geocoder::default();

        let mut hop1 = hop("Dublin", 53.35, -6.25, "", 0.0, 0.0);
        hop1.dest_name = String::new();
        let hop2 = hop("Cork", 51.9, -8.47, "Galway", 53.27, -9.06);

        let resolved = geocoder.resolve_trip_coords(vec![hop1, hop2]).await;

        assert_eq!(resolved[0].dest_name, "Cork");
        assert!((resolved[0].dest_lat - 51.9).abs() < f64::EPSILON);
        assert!((resolved[0].dest_lng - 8.47_f64.copysign(-1.0)).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn resolve_trip_coords_does_not_overwrite_named_dest() {
        let geocoder = Geocoder::default();

        let hop1 = hop("Dublin", 53.35, -6.25, "Limerick", 0.0, 0.0);
        let hop2 = hop("Cork", 51.9, -8.47, "Galway", 53.27, -9.06);

        let resolved = geocoder.resolve_trip_coords(vec![hop1, hop2]).await;

        assert_eq!(
            resolved[0].dest_name, "Limerick",
            "named dest should not be overwritten even when coords are 0,0"
        );
    }

    #[tokio::test]
    async fn resolve_trip_coords_copies_country_from_next_hop() {
        let geocoder = Geocoder::default();

        let mut hop1 = hop("Dublin", 53.35, -6.25, "", 0.0, 0.0);
        hop1.dest_name = String::new();
        let mut hop2 = hop("Cork", 51.9, -8.47, "Galway", 53.27, -9.06);
        hop2.origin_country = Some("ie".to_string());

        let resolved = geocoder.resolve_trip_coords(vec![hop1, hop2]).await;

        assert_eq!(
            resolved[0].dest_country.as_deref(),
            Some("ie"),
            "dest_country should be copied from next hop's origin_country"
        );
    }

    #[test]
    fn tz_to_country_maps_known_timezones() {
        assert_eq!(tz_to_country("Europe/Dublin"), Some("ie"));
        assert_eq!(tz_to_country("America/New_York"), Some("us"));
        assert_eq!(tz_to_country("Europe/London"), Some("gb"));
        assert_eq!(tz_to_country("Asia/Tokyo"), Some("jp"));
        assert_eq!(tz_to_country("Africa/Casablanca"), Some("ma"));
    }

    #[test]
    fn tz_to_country_returns_none_for_unknown() {
        assert_eq!(tz_to_country("UTC"), None);
        assert_eq!(tz_to_country("Pacific/Auckland"), None);
        assert_eq!(tz_to_country(""), None);
    }

    #[tokio::test]
    async fn resolve_trip_coords_clears_coords_on_tz_country_mismatch() {
        let geocoder = Geocoder::default();

        let mut h = hop("Westport", 41.14, -73.36, "Dublin", 53.35, -6.25);
        h.origin_country = Some("US".to_string());
        h.origin_tz = Some("Europe/Dublin".to_string());
        h.dest_tz = Some("Europe/Dublin".to_string());

        let resolved = geocoder.resolve_trip_coords(vec![h]).await;

        assert_eq!(
            resolved[0].origin_country.as_deref(),
            Some("ie"),
            "origin_country should be overwritten by timezone-derived country"
        );
        assert_eq!(
            resolved[0].dest_country.as_deref(),
            Some("ie"),
            "dest_country should be set from timezone when None"
        );
    }

    #[tokio::test]
    async fn resolve_trip_coords_preserves_coords_when_tz_matches_country() {
        let geocoder = Geocoder::default();

        let mut h = hop("Seoul", 37.55, 126.97, "Busan", 35.11, 129.04);
        h.origin_country = Some("kr".to_string());
        h.dest_country = Some("kr".to_string());
        h.origin_tz = Some("Asia/Seoul".to_string());
        h.dest_tz = Some("Asia/Seoul".to_string());

        let resolved = geocoder.resolve_trip_coords(vec![h]).await;

        assert!((resolved[0].origin_lat - 37.55).abs() < f64::EPSILON);
        assert!((resolved[0].origin_lng - 126.97).abs() < f64::EPSILON);
        assert!((resolved[0].dest_lat - 35.11).abs() < f64::EPSILON);
        assert!((resolved[0].dest_lng - 129.04).abs() < f64::EPSILON);
    }
}

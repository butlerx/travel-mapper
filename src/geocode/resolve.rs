//! Trip coordinate resolution — fills missing lat/lng on hops via geocoding,
//! airport lookups, timezone sanity checks, and neighbour-hop inference.

use super::{nominatim::Geocoder, sanitize::tz_to_country};
use crate::db::hops::Row;

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

fn is_missing(lat: f64, lng: f64) -> bool {
    (lat == 0.0) && (lng == 0.0)
}

impl Geocoder {
    /// Resolve missing coordinates for a list of hops using airport lookups,
    /// Nominatim geocoding, and chained fallback strategies.
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
            id: 0,
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
            boat_detail: None,
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
}

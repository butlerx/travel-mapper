//! Trip fetching, pagination, and deduplication logic.

use super::{
    Hop,
    client::{FetchError, TripItApi},
    ensure_list, parsers,
};
use serde_json::Value;

async fn fetch_trip_objects(
    api: &dyn TripItApi,
    geocoder: &crate::geocode::Geocoder,
    trip_id: &str,
) -> Result<Vec<Hop>, FetchError> {
    let data = api.get_trip_objects(trip_id).await?;

    let mut hops = Vec::new();

    for obj_type in ["AirObject", "RailObject", "CruiseObject", "TransportObject"] {
        for obj in ensure_list(&data[obj_type]) {
            match obj_type {
                "AirObject" => hops.extend(parsers::parse_air(obj)),
                "RailObject" => hops.extend(parsers::parse_rail(obj)),
                "CruiseObject" => hops.extend(parsers::parse_cruise(obj)),
                "TransportObject" => hops.extend(parsers::parse_transport(obj)),
                other => tracing::warn!(object_type = other, trip_id, "unknown object type"),
            }
        }
    }

    Ok(geocoder.resolve_trip_coords(hops).await)
}

async fn fetch_paginated(api: &dyn TripItApi, past: bool) -> Result<Vec<Value>, FetchError> {
    let mut trips = Vec::new();
    let mut page = 1u64;
    let page_size = 25u64;

    loop {
        let data = api.list_trips(past, page, page_size).await?;

        let batch = ensure_list(&data["Trip"]);
        let batch_len = batch.len();
        trips.extend(batch.into_iter().cloned());

        let max_page = data
            .get("max_page")
            .and_then(|v| v.as_str().or_else(|| v.as_u64().map(|_| "")))
            .and_then(|s| {
                if s.is_empty() {
                    data.get("max_page").and_then(serde_json::Value::as_u64)
                } else {
                    s.parse().ok()
                }
            })
            .unwrap_or(1);

        tracing::info!(page, max_page, batch_len, "fetched trip page");

        if page >= max_page {
            break;
        }
        page += 1;
    }

    Ok(trips)
}

/// A single `TripIt` trip with its resolved hops.
#[derive(Debug, Clone)]
pub struct Trip {
    /// The `TripIt` trip ID (numeric string from the API).
    pub id: String,
    /// Human-readable trip name from `TripIt`.
    pub display_name: String,
    /// Resolved travel hops belonging to this trip.
    pub hops: Vec<Hop>,
}

fn parse_trip_id(trip: &Value) -> Option<String> {
    trip.get("id").and_then(|v| {
        v.as_str()
            .map(std::string::ToString::to_string)
            .or_else(|| v.as_u64().map(|n| n.to_string()))
    })
}

/// Fetch all trips from `TripIt`, returning each trip with its resolved hops.
///
/// Trips are fetched in paginated listings (past + upcoming), deduplicated by
/// ID, then each trip's objects are fetched individually. Per-trip fetch errors
/// are logged and the trip is skipped — they do not fail the entire sync.
///
/// # Errors
///
/// Returns `FetchError` if paginated trip listing fails. Individual trip-object
/// fetch failures are non-fatal (logged and skipped).
pub async fn fetch_trips(
    api: &dyn TripItApi,
    geocoder: &crate::geocode::Geocoder,
) -> Result<Vec<Trip>, FetchError> {
    let mut all_trips = Vec::new();

    tracing::info!(past = true, "fetching trips");
    all_trips.extend(fetch_paginated(api, true).await?);

    tracing::info!(past = false, "fetching trips");
    all_trips.extend(fetch_paginated(api, false).await?);

    let mut seen = std::collections::HashSet::new();
    let unique_trips: Vec<&Value> = all_trips
        .iter()
        .filter(|trip| {
            let tid = parse_trip_id(trip).unwrap_or_default();
            !tid.is_empty() && seen.insert(tid)
        })
        .collect();

    tracing::info!(unique_trips = unique_trips.len(), "fetching hops");

    let mut result = Vec::new();
    let total = unique_trips.len();

    for (i, trip) in unique_trips.iter().enumerate() {
        let trip_id = parse_trip_id(trip).unwrap_or_default();
        let trip_name = trip
            .get("display_name")
            .and_then(|v| v.as_str())
            .unwrap_or(&format!("Unknown Trip {trip_id}"))
            .trim()
            .to_string();

        tracing::info!(progress = i + 1, total, trip_name, "processing trip");

        match fetch_trip_objects(api, geocoder, &trip_id).await {
            Ok(hops) => {
                if hops.is_empty() {
                    tracing::debug!(trip_id, trip_name, "skipping trip with 0 hops");
                } else {
                    tracing::debug!(count = hops.len(), "found hops");
                    result.push(Trip {
                        id: trip_id,
                        display_name: trip_name,
                        hops,
                    });
                }
            }
            Err(e) => tracing::warn!(trip_id, error = %e, "trip fetch failed"),
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geocode::Geocoder;
    use serde_json::json;

    struct MockApi;

    #[async_trait::async_trait]
    impl TripItApi for MockApi {
        async fn list_trips(
            &self,
            past: bool,
            _page: u64,
            _page_size: u64,
        ) -> Result<Value, FetchError> {
            if past {
                Ok(json!({
                    "max_page": "1",
                    "Trip": [
                        {"id": "1", "display_name": "Trip With Hops"},
                        {"id": "2", "display_name": "Empty Trip"},
                        {"id": "3", "display_name": "Another Trip With Hops"},
                    ]
                }))
            } else {
                Ok(json!({"max_page": "1"}))
            }
        }

        async fn get_trip_objects(&self, trip_id: &str) -> Result<Value, FetchError> {
            match trip_id {
                "1" => Ok(json!({
                    "AirObject": {
                        "Segment": {
                            "start_airport_code": "DUB",
                            "end_airport_code": "LHR",
                            "StartDateTime": {"date": "2024-06-01"},
                            "EndDateTime": {"date": "2024-06-01"},
                        }
                    }
                })),
                "3" => Ok(json!({
                    "AirObject": {
                        "Segment": {
                            "start_airport_code": "JFK",
                            "end_airport_code": "LAX",
                            "StartDateTime": {"date": "2024-07-01"},
                            "EndDateTime": {"date": "2024-07-01"},
                        }
                    }
                })),
                // Trip "2" and any unknown IDs return no objects
                _ => Ok(json!({})),
            }
        }
    }

    #[tokio::test]
    async fn fetch_trips_skips_trips_with_zero_hops() {
        let api = MockApi;
        let geocoder = Geocoder::default();

        let trips = fetch_trips(&api, &geocoder).await.unwrap();

        assert_eq!(trips.len(), 2, "should exclude the trip with 0 hops");
        assert_eq!(trips[0].id, "1");
        assert_eq!(trips[0].display_name, "Trip With Hops");
        assert!(!trips[0].hops.is_empty());
        assert_eq!(trips[1].id, "3");
        assert_eq!(trips[1].display_name, "Another Trip With Hops");
        assert!(!trips[1].hops.is_empty());
    }
}

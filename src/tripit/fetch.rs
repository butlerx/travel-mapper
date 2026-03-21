//! `TripIt` API client and travel-object parsers.

use super::auth::{AuthError, TripItAuth};
use crate::db::hops::{Row as Hop, TravelType};
use serde_json::Value;
use thiserror::Error;

const TRIPIT_API_BASE: &str = "https://api.tripit.com/v1";

/// Attempt to parse a JSON value as f64, returning None on failure.
fn coerce_float(val: &Value) -> Option<f64> {
    match val {
        Value::Number(n) => n.as_f64(),
        Value::String(s) if !s.is_empty() => s.parse().ok(),
        _ => None,
    }
}

#[async_trait::async_trait]
pub trait TripItApi: Send + Sync {
    async fn list_trips(&self, past: bool, page: u64, page_size: u64) -> Result<Value, FetchError>;
    async fn get_trip_objects(&self, trip_id: &str) -> Result<Value, FetchError>;
}

pub struct TripItClient {
    auth: TripItAuth,
    client: reqwest::Client,
    base_url: String,
}

impl TripItClient {
    #[must_use]
    pub fn new(auth: TripItAuth) -> Self {
        let client = reqwest::Client::new();
        Self::with_base_url(auth, client, TRIPIT_API_BASE.to_string())
    }

    #[must_use]
    pub fn with_base_url(auth: TripItAuth, client: reqwest::Client, base_url: String) -> Self {
        Self {
            auth,
            client,
            base_url,
        }
    }

    async fn get(&self, path: &str) -> Result<Value, FetchError> {
        let url = format!("{}/{path}/format/json", self.base_url);
        tracing::debug!("GET {url}");

        let max_retries: u32 = 3;
        let mut attempt = 0;

        loop {
            attempt += 1;
            let auth_header = self.auth.to_header("GET", &url)?;
            let result = self
                .client
                .get(&url)
                .header("Authorization", auth_header)
                .timeout(std::time::Duration::from_secs(30))
                .send()
                .await;

            match result {
                Ok(resp) => {
                    let status = resp.status();
                    if (status.is_server_error()
                        || status == reqwest::StatusCode::TOO_MANY_REQUESTS)
                        && attempt <= max_retries
                    {
                        let delay = std::time::Duration::from_millis(
                            500 * u64::from(2_u32.pow(attempt - 1)),
                        );
                        tracing::warn!(
                            "GET {url} returned {status}, retrying in {}ms (attempt {attempt}/{max_retries})",
                            delay.as_millis()
                        );
                        tokio::time::sleep(delay).await;
                        continue;
                    }
                    resp.error_for_status_ref()?;
                    let body = resp.bytes().await?;
                    return Ok(serde_json::from_slice(&body)?);
                }
                Err(err) => {
                    if (err.is_connect() || err.is_timeout()) && attempt <= max_retries {
                        let delay = std::time::Duration::from_millis(
                            500 * u64::from(2_u32.pow(attempt - 1)),
                        );
                        tracing::warn!(
                            "GET {url} failed: {err}, retrying in {}ms (attempt {attempt}/{max_retries})",
                            delay.as_millis()
                        );
                        tokio::time::sleep(delay).await;
                        continue;
                    }
                    return Err(FetchError::Http(err));
                }
            }
        }
    }
}

#[async_trait::async_trait]
impl TripItApi for TripItClient {
    async fn list_trips(&self, past: bool, page: u64, page_size: u64) -> Result<Value, FetchError> {
        let endpoint = if past {
            format!("list/trip/past/true/page_num/{page}/page_size/{page_size}")
        } else {
            format!("list/trip/page_num/{page}/page_size/{page_size}")
        };
        self.get(&endpoint).await
    }

    async fn get_trip_objects(&self, trip_id: &str) -> Result<Value, FetchError> {
        self.get(&format!("get/trip/id/{trip_id}/include_objects/true"))
            .await
    }
}

#[derive(Debug, Error)]
pub enum FetchError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("failed to parse API response: {0}")]
    Json(#[from] serde_json::Error),

    #[error("OAuth signing failed: {0}")]
    Auth(#[from] AuthError),
}

fn extract_coords(addr: &Value) -> (Option<f64>, Option<f64>) {
    if addr.is_null() || !addr.is_object() {
        return (None, None);
    }
    (
        coerce_float(&addr["latitude"]),
        coerce_float(&addr["longitude"]),
    )
}

fn ensure_list(val: &Value) -> Vec<&Value> {
    match val {
        Value::Array(arr) => arr.iter().collect(),
        Value::Null => vec![],
        other => vec![other],
    }
}

fn get_str(val: &Value, key: &str) -> String {
    val.get(key)
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}

fn get_date(val: &Value, datetime_key: &str) -> String {
    val.get(datetime_key)
        .and_then(|dt| dt.get("date"))
        .and_then(|d| d.as_str())
        .unwrap_or("")
        .to_string()
}

fn parse_air(obj: &Value) -> Vec<Hop> {
    ensure_list(&obj["Segment"])
        .into_iter()
        .map(|seg| Hop {
            travel_type: TravelType::Air,
            origin_name: get_str(seg, "start_airport_code"),
            origin_lat: coerce_float(&seg["start_airport_latitude"]),
            origin_lng: coerce_float(&seg["start_airport_longitude"]),
            dest_name: get_str(seg, "end_airport_code"),
            dest_lat: coerce_float(&seg["end_airport_latitude"]),
            dest_lng: coerce_float(&seg["end_airport_longitude"]),
            start_date: get_date(seg, "StartDateTime"),
            end_date: get_date(seg, "EndDateTime"),
        })
        .collect()
}

fn parse_rail(obj: &Value) -> Vec<Hop> {
    ensure_list(&obj["Segment"])
        .into_iter()
        .map(|seg| {
            let start_addr = &seg["StartStationAddress"];
            let end_addr = &seg["EndStationAddress"];
            let (slat, slng) = extract_coords(start_addr);
            let (dlat, dlng) = extract_coords(end_addr);

            let origin_name = {
                let name = get_str(seg, "start_station_name");
                if name.is_empty() {
                    get_str(start_addr, "city")
                } else {
                    name
                }
            };
            let dest_name = {
                let name = get_str(seg, "end_station_name");
                if name.is_empty() {
                    get_str(end_addr, "city")
                } else {
                    name
                }
            };

            Hop {
                travel_type: TravelType::Rail,
                origin_name,
                origin_lat: slat,
                origin_lng: slng,
                dest_name,
                dest_lat: dlat,
                dest_lng: dlng,
                start_date: get_date(seg, "StartDateTime"),
                end_date: get_date(seg, "EndDateTime"),
            }
        })
        .collect()
}

fn parse_cruise(obj: &Value) -> Vec<Hop> {
    let start_addr = &obj["StartLocationAddress"];
    let end_addr = &obj["EndLocationAddress"];
    let (slat, slng) = extract_coords(start_addr);
    let (dlat, dlng) = extract_coords(end_addr);

    let origin_name = {
        let name = get_str(obj, "start_location_name");
        if name.is_empty() {
            get_str(start_addr, "city")
        } else {
            name
        }
    };
    let dest_name = {
        let name = get_str(obj, "end_location_name");
        if name.is_empty() {
            get_str(end_addr, "city")
        } else {
            name
        }
    };

    vec![Hop {
        travel_type: TravelType::Cruise,
        origin_name,
        origin_lat: slat,
        origin_lng: slng,
        dest_name,
        dest_lat: dlat,
        dest_lng: dlng,
        start_date: get_date(obj, "StartDateTime"),
        end_date: get_date(obj, "EndDateTime"),
    }]
}

fn parse_transport(obj: &Value) -> Vec<Hop> {
    let nested = &obj["Segment"];
    let items = if nested.is_null() {
        vec![obj]
    } else {
        ensure_list(nested)
    };

    items
        .into_iter()
        .map(|seg| {
            let start_addr = &seg["StartAddress"];
            let end_addr = &seg["EndAddress"];
            let (slat, slng) = extract_coords(start_addr);
            let (dlat, dlng) = extract_coords(end_addr);

            let origin_name = {
                let name = get_str(seg, "start_location_name");
                if name.is_empty() {
                    get_str(start_addr, "city")
                } else {
                    name
                }
            };
            let dest_name = {
                let name = get_str(seg, "end_location_name");
                if name.is_empty() {
                    get_str(end_addr, "city")
                } else {
                    name
                }
            };

            Hop {
                travel_type: TravelType::Transport,
                origin_name,
                origin_lat: slat,
                origin_lng: slng,
                dest_name,
                dest_lat: dlat,
                dest_lng: dlng,
                start_date: get_date(seg, "StartDateTime"),
                end_date: get_date(seg, "EndDateTime"),
            }
        })
        .collect()
}

async fn fetch_trip_objects(api: &dyn TripItApi, trip_id: &str) -> Result<Vec<Hop>, FetchError> {
    let data = api.get_trip_objects(trip_id).await?;

    let mut hops = Vec::new();

    for obj_type in ["AirObject", "RailObject", "CruiseObject", "TransportObject"] {
        for obj in ensure_list(&data[obj_type]) {
            match obj_type {
                "AirObject" => hops.extend(parse_air(obj)),
                "RailObject" => hops.extend(parse_rail(obj)),
                "CruiseObject" => hops.extend(parse_cruise(obj)),
                "TransportObject" => hops.extend(parse_transport(obj)),
                other => tracing::warn!("Unknown object type '{}' in trip {}", other, trip_id),
            }
        }
    }

    Ok(hops)
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

        tracing::info!("page {page}/{max_page} -- {batch_len} trips");

        if page >= max_page {
            break;
        }
        page += 1;
    }

    Ok(trips)
}

/// # Errors
///
/// Returns `FetchError` if any API request or JSON parse fails.
pub async fn fetch_all_hops(api: &dyn TripItApi) -> Result<Vec<Hop>, FetchError> {
    let mut all_trips = Vec::new();

    tracing::info!("Fetching past trips...");
    all_trips.extend(fetch_paginated(api, true).await?);

    tracing::info!("Fetching upcoming trips...");
    all_trips.extend(fetch_paginated(api, false).await?);

    let mut seen = std::collections::HashSet::new();
    let unique_trips: Vec<&Value> = all_trips
        .iter()
        .filter(|trip| {
            let tid = trip
                .get("id")
                .and_then(|v| {
                    v.as_str()
                        .map(std::string::ToString::to_string)
                        .or_else(|| v.as_u64().map(|n| n.to_string()))
                })
                .unwrap_or_default();
            !tid.is_empty() && seen.insert(tid)
        })
        .collect();

    tracing::info!(
        "Found {} unique trips, fetching hops...",
        unique_trips.len()
    );

    let mut all_hops = Vec::new();
    let total = unique_trips.len();

    for (i, trip) in unique_trips.iter().enumerate() {
        let trip_id = trip
            .get("id")
            .and_then(|v| {
                v.as_str()
                    .map(std::string::ToString::to_string)
                    .or_else(|| v.as_u64().map(|n| n.to_string()))
            })
            .unwrap_or_default();
        let trip_name = trip
            .get("display_name")
            .and_then(|v| v.as_str())
            .unwrap_or(&format!("Unknown Trip {trip_id}"))
            .trim()
            .to_string();

        tracing::info!("[{}/{}] {trip_name}", i + 1, total);

        match fetch_trip_objects(api, &trip_id).await {
            Ok(trip_hops) => {
                let count = trip_hops.len();
                if count > 0 {
                    tracing::debug!("found {count} hops");
                }
                all_hops.extend(trip_hops);
            }
            Err(e) => tracing::warn!("trip {trip_id} failed: {e}"),
        }
    }

    Ok(all_hops)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn ensure_list_handles_array_null_and_single_object() {
        let arr = json!([{"id": 1}, {"id": 2}]);
        let null = Value::Null;
        let single = json!({"id": 99});

        assert_eq!(ensure_list(&arr).len(), 2);
        assert!(ensure_list(&null).is_empty());
        assert_eq!(ensure_list(&single).len(), 1);
        assert_eq!(ensure_list(&single)[0]["id"], json!(99));
    }

    #[test]
    fn get_str_and_get_date_handle_present_and_missing_values() {
        let obj = json!({
            "name": "Paris",
            "StartDateTime": {"date": "2024-01-15"},
            "bad_date": {"date": 123}
        });

        assert_eq!(get_str(&obj, "name"), "Paris");
        assert_eq!(get_str(&obj, "missing"), "");
        assert_eq!(get_date(&obj, "StartDateTime"), "2024-01-15");
        assert_eq!(get_date(&obj, "missing"), "");
        assert_eq!(get_date(&obj, "bad_date"), "");
    }

    #[test]
    fn extract_coords_handles_valid_and_missing_coordinates() {
        let addr = json!({"latitude": "48.8566", "longitude": 2.3522});
        let (lat, lng) = extract_coords(&addr);
        assert_eq!(lat, Some(48.8566));
        assert_eq!(lng, Some(2.3522));

        let missing = json!({"latitude": "", "longitude": null});
        let (lat, lng) = extract_coords(&missing);
        assert_eq!(lat, None);
        assert_eq!(lng, None);

        let not_an_object = json!("Paris");
        let (lat, lng) = extract_coords(&not_an_object);
        assert_eq!(lat, None);
        assert_eq!(lng, None);
    }

    #[test]
    fn parse_air_normal_case_with_all_fields() {
        let obj = json!({
            "Segment": [{
                "start_airport_code": "LHR",
                "start_airport_latitude": "51.4700",
                "start_airport_longitude": -0.4543,
                "end_airport_code": "JFK",
                "end_airport_latitude": 40.6413,
                "end_airport_longitude": "-73.7781",
                "StartDateTime": {"date": "2024-03-01"},
                "EndDateTime": {"date": "2024-03-01"}
            }]
        });

        let hops = parse_air(&obj);
        assert_eq!(hops.len(), 1);
        let hop = &hops[0];
        assert!(matches!(hop.travel_type, TravelType::Air));
        assert_eq!(hop.origin_name, "LHR");
        assert_eq!(hop.dest_name, "JFK");
        assert_eq!(hop.origin_lat, Some(51.47));
        assert_eq!(hop.origin_lng, Some(-0.4543));
        assert_eq!(hop.dest_lat, Some(40.6413));
        assert_eq!(hop.dest_lng, Some(-73.7781));
        assert_eq!(hop.start_date, "2024-03-01");
        assert_eq!(hop.end_date, "2024-03-01");
    }

    #[test]
    fn parse_air_handles_single_object_missing_coords_and_optional_fields() {
        let obj = json!({
            "Segment": {
                "start_airport_code": "",
                "start_airport_latitude": null,
                "end_airport_code": null,
                "end_airport_longitude": "",
                "StartDateTime": {"date": ""}
            }
        });

        let hops = parse_air(&obj);
        assert_eq!(hops.len(), 1);
        let hop = &hops[0];
        assert!(matches!(hop.travel_type, TravelType::Air));
        assert_eq!(hop.origin_name, "");
        assert_eq!(hop.dest_name, "");
        assert_eq!(hop.origin_lat, None);
        assert_eq!(hop.origin_lng, None);
        assert_eq!(hop.dest_lat, None);
        assert_eq!(hop.dest_lng, None);
        assert_eq!(hop.start_date, "");
        assert_eq!(hop.end_date, "");
    }

    #[test]
    fn parse_air_handles_empty_and_null_segments() {
        let empty = json!({"Segment": []});
        let null = json!({"Segment": null});

        assert!(parse_air(&empty).is_empty());
        assert!(parse_air(&null).is_empty());
    }

    #[test]
    fn parse_rail_normal_case_extracts_station_names_and_coords() {
        let obj = json!({
            "Segment": [{
                "start_station_name": "Gare du Nord",
                "end_station_name": "St Pancras",
                "StartStationAddress": {"city": "Paris", "latitude": 48.8809, "longitude": 2.3553},
                "EndStationAddress": {"city": "London", "latitude": "51.5319", "longitude": "-0.1263"},
                "StartDateTime": {"date": "2024-05-10"},
                "EndDateTime": {"date": "2024-05-10"}
            }]
        });

        let hops = parse_rail(&obj);
        assert_eq!(hops.len(), 1);
        let hop = &hops[0];
        assert!(matches!(hop.travel_type, TravelType::Rail));
        assert_eq!(hop.origin_name, "Gare du Nord");
        assert_eq!(hop.dest_name, "St Pancras");
        assert_eq!(hop.origin_lat, Some(48.8809));
        assert_eq!(hop.origin_lng, Some(2.3553));
        assert_eq!(hop.dest_lat, Some(51.5319));
        assert_eq!(hop.dest_lng, Some(-0.1263));
    }

    #[test]
    fn parse_rail_handles_single_segment_fallback_names_and_missing_coords() {
        let obj = json!({
            "Segment": {
                "start_station_name": "",
                "end_station_name": "",
                "StartStationAddress": {"city": "Berlin", "latitude": null, "longitude": ""},
                "EndStationAddress": {"city": "Munich"},
                "StartDateTime": {"date": "2024-06-01"},
                "EndDateTime": {"date": "2024-06-01"}
            }
        });

        let hops = parse_rail(&obj);
        assert_eq!(hops.len(), 1);
        let hop = &hops[0];
        assert!(matches!(hop.travel_type, TravelType::Rail));
        assert_eq!(hop.origin_name, "Berlin");
        assert_eq!(hop.dest_name, "Munich");
        assert_eq!(hop.origin_lat, None);
        assert_eq!(hop.origin_lng, None);
        assert_eq!(hop.dest_lat, None);
        assert_eq!(hop.dest_lng, None);
    }

    #[test]
    fn parse_rail_handles_empty_and_null_segments() {
        let empty = json!({"Segment": []});
        let null = json!({"Segment": null});

        assert!(parse_rail(&empty).is_empty());
        assert!(parse_rail(&null).is_empty());
    }

    #[test]
    fn parse_cruise_normal_case_extracts_location_names_and_coords() {
        let obj = json!({
            "start_location_name": "Port of Miami",
            "end_location_name": "Nassau Port",
            "StartLocationAddress": {"city": "Miami", "latitude": 25.7781, "longitude": -80.1794},
            "EndLocationAddress": {"city": "Nassau", "latitude": "25.0780", "longitude": "-77.3431"},
            "StartDateTime": {"date": "2024-07-01"},
            "EndDateTime": {"date": "2024-07-05"}
        });

        let hops = parse_cruise(&obj);
        assert_eq!(hops.len(), 1);
        let hop = &hops[0];
        assert!(matches!(hop.travel_type, TravelType::Cruise));
        assert_eq!(hop.origin_name, "Port of Miami");
        assert_eq!(hop.dest_name, "Nassau Port");
        assert_eq!(hop.origin_lat, Some(25.7781));
        assert_eq!(hop.origin_lng, Some(-80.1794));
        assert_eq!(hop.dest_lat, Some(25.078));
        assert_eq!(hop.dest_lng, Some(-77.3431));
        assert_eq!(hop.start_date, "2024-07-01");
        assert_eq!(hop.end_date, "2024-07-05");
    }

    #[test]
    fn parse_cruise_handles_missing_optional_fields_and_coord_fallbacks() {
        let obj = json!({
            "start_location_name": "",
            "end_location_name": null,
            "StartLocationAddress": {"city": "Venice", "latitude": null, "longitude": null},
            "EndLocationAddress": {"city": "Dubrovnik"},
            "StartDateTime": {"date": ""},
            "EndDateTime": null
        });

        let hops = parse_cruise(&obj);
        assert_eq!(hops.len(), 1);
        let hop = &hops[0];
        assert!(matches!(hop.travel_type, TravelType::Cruise));
        assert_eq!(hop.origin_name, "Venice");
        assert_eq!(hop.dest_name, "Dubrovnik");
        assert_eq!(hop.origin_lat, None);
        assert_eq!(hop.origin_lng, None);
        assert_eq!(hop.dest_lat, None);
        assert_eq!(hop.dest_lng, None);
        assert_eq!(hop.start_date, "");
        assert_eq!(hop.end_date, "");
    }

    #[test]
    fn parse_transport_handles_top_level_object_without_segment_key() {
        let obj = json!({
            "start_location_name": "Hotel Lobby",
            "end_location_name": "Airport",
            "StartAddress": {"city": "Rome", "latitude": "41.9028", "longitude": "12.4964"},
            "EndAddress": {"city": "Rome", "latitude": 41.8003, "longitude": 12.2389},
            "StartDateTime": {"date": "2024-08-01"},
            "EndDateTime": {"date": "2024-08-01"}
        });

        let hops = parse_transport(&obj);
        assert_eq!(hops.len(), 1);
        let hop = &hops[0];
        assert!(matches!(hop.travel_type, TravelType::Transport));
        assert_eq!(hop.origin_name, "Hotel Lobby");
        assert_eq!(hop.dest_name, "Airport");
        assert_eq!(hop.origin_lat, Some(41.9028));
        assert_eq!(hop.origin_lng, Some(12.4964));
        assert_eq!(hop.dest_lat, Some(41.8003));
        assert_eq!(hop.dest_lng, Some(12.2389));
    }

    #[test]
    fn parse_transport_handles_nested_segment_array_and_single_object() {
        let array_obj = json!({
            "Segment": [
                {
                    "start_location_name": "",
                    "end_location_name": "",
                    "StartAddress": {"city": "Lisbon", "latitude": null, "longitude": ""},
                    "EndAddress": {"city": "Porto"},
                    "StartDateTime": {"date": "2024-09-01"},
                    "EndDateTime": {"date": "2024-09-01"}
                },
                {
                    "start_location_name": "Porto",
                    "end_location_name": "Braga",
                    "StartAddress": {"city": "Porto", "latitude": 41.1496, "longitude": -8.6109},
                    "EndAddress": {"city": "Braga", "latitude": 41.5454, "longitude": -8.4265},
                    "StartDateTime": {"date": "2024-09-02"},
                    "EndDateTime": {"date": "2024-09-02"}
                }
            ]
        });

        let single_obj = json!({
            "Segment": {
                "start_location_name": "",
                "end_location_name": "",
                "StartAddress": {"city": "Madrid"},
                "EndAddress": {"city": "Toledo"},
                "StartDateTime": {"date": "2024-10-01"},
                "EndDateTime": {"date": "2024-10-01"}
            }
        });

        let array_hops = parse_transport(&array_obj);
        assert_eq!(array_hops.len(), 2);
        assert!(matches!(array_hops[0].travel_type, TravelType::Transport));
        assert_eq!(array_hops[0].origin_name, "Lisbon");
        assert_eq!(array_hops[0].dest_name, "Porto");
        assert_eq!(array_hops[0].origin_lat, None);
        assert_eq!(array_hops[0].origin_lng, None);
        assert!(matches!(array_hops[1].travel_type, TravelType::Transport));

        let single_hops = parse_transport(&single_obj);
        assert_eq!(single_hops.len(), 1);
        assert_eq!(single_hops[0].origin_name, "Madrid");
        assert_eq!(single_hops[0].dest_name, "Toledo");
    }

    #[test]
    fn parse_transport_handles_empty_segment_array() {
        let obj = json!({"Segment": []});
        assert!(parse_transport(&obj).is_empty());
    }

    #[test]
    fn coerce_float_handles_number_value() {
        assert_eq!(coerce_float(&json!(12.34)), Some(12.34));
    }

    #[test]
    fn coerce_float_handles_string_number() {
        assert_eq!(coerce_float(&json!("56.78")), Some(56.78));
    }

    #[test]
    fn coerce_float_handles_empty_string() {
        assert_eq!(coerce_float(&json!("")), None);
    }

    #[test]
    fn coerce_float_handles_null() {
        assert_eq!(coerce_float(&json!(null)), None);
    }

    #[test]
    fn coerce_float_handles_non_numeric_string() {
        assert_eq!(coerce_float(&json!("not-a-number")), None);
    }

    #[tokio::test]
    async fn get_retries_on_server_error() {
        use crate::tripit::auth::TripItAuth;
        use std::sync::Arc;
        use std::sync::atomic::{AtomicU32, Ordering};
        use tokio::io::AsyncWriteExt;

        let attempt_count = Arc::new(AtomicU32::new(0));
        let count = Arc::clone(&attempt_count);

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        tokio::spawn(async move {
            loop {
                let (mut stream, _) = listener.accept().await.unwrap();
                let attempt = count.fetch_add(1, Ordering::SeqCst) + 1;

                let mut buf = vec![0_u8; 4096];
                let _ = tokio::io::AsyncReadExt::read(&mut stream, &mut buf).await;

                let response = if attempt <= 2 {
                    "HTTP/1.1 500 Internal Server Error\r\nContent-Length: 0\r\n\r\n"
                } else {
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 2\r\n\r\n{}"
                };

                stream.write_all(response.as_bytes()).await.unwrap();
            }
        });

        let auth = TripItAuth::new(
            "key".to_string(),
            "secret".to_string(),
            "token".to_string(),
            "token_secret".to_string(),
        );
        let client = TripItClient::with_base_url(
            auth,
            reqwest::Client::new(),
            format!("http://127.0.0.1:{port}"),
        );

        let result = client.get("test/endpoint").await;
        assert!(
            result.is_ok(),
            "expected success after retries, got: {result:?}"
        );
        assert_eq!(attempt_count.load(Ordering::SeqCst), 3);
    }
}

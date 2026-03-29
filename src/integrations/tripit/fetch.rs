//! `TripIt` API client and travel-object parsers.

mod client;
mod parsers;
mod trips;

pub use client::{FetchError, TripItApi, TripItClient};
pub use trips::{Trip, fetch_trips};

use crate::db::hops::{
    BoatDetail, FlightDetail, RailDetail, Row as Hop, TransportDetail, TravelType,
};
use serde_json::Value;

/// Attempt to parse a JSON value as f64, returning None on failure.
pub(super) fn coerce_float(val: &Value) -> Option<f64> {
    match val {
        Value::Number(n) => n.as_f64(),
        Value::String(s) if !s.is_empty() => s.parse().ok(),
        _ => None,
    }
}

pub(super) fn extract_coords(addr: &Value) -> (f64, f64) {
    if addr.is_null() || !addr.is_object() {
        return (0.0, 0.0);
    }
    (
        coerce_float(&addr["latitude"]).unwrap_or(0.0),
        coerce_float(&addr["longitude"]).unwrap_or(0.0),
    )
}

/// Extract the ISO 3166-1 country code from a `TripIt` address object,
/// lowercased for use with Nominatim's `countrycodes` parameter.
pub(super) fn extract_country(addr: &Value) -> Option<String> {
    addr.get("country")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(str::to_lowercase)
}

pub(super) fn ensure_list(val: &Value) -> Vec<&Value> {
    match val {
        Value::Array(arr) => arr.iter().collect(),
        Value::Null => vec![],
        other => vec![other],
    }
}

pub(super) fn get_str(val: &Value, key: &str) -> String {
    val.get(key)
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}

pub(super) fn get_date(val: &Value, datetime_key: &str) -> String {
    val.get(datetime_key)
        .and_then(|dt| dt.get("date"))
        .and_then(|d| d.as_str())
        .unwrap_or("")
        .to_string()
}

pub(super) fn get_tz(val: &Value, datetime_key: &str) -> Option<String> {
    val.get(datetime_key)
        .and_then(|dt| dt.get("timezone"))
        .and_then(|t| t.as_str())
        .filter(|s| !s.is_empty())
        .map(String::from)
}

/// Strip trailing platform/track identifiers from station names.
///
/// `TripIt` sometimes appends platform numbers or single-letter codes to rail
/// station names (e.g. "Belfast 1", "Ballymote S"). This trims those suffixes
/// so we store clean station names.
pub(super) fn strip_station_suffix(name: &str) -> String {
    let trimmed = name.trim();
    if let Some(idx) = trimmed.rfind(' ') {
        let suffix = &trimmed[idx + 1..];
        // Strip if suffix is purely digits ("1", "12") or a single uppercase
        // letter ("S", "A") — both are platform/track identifiers, not part of
        // the station name.
        let is_platform = suffix.bytes().all(|b| b.is_ascii_digit())
            || (suffix.len() == 1 && suffix.bytes().all(|b| b.is_ascii_uppercase()));
        if is_platform {
            return trimmed[..idx].to_string();
        }
    }
    trimmed.to_string()
}

pub(super) fn build_address_query(addr: &Value) -> Option<String> {
    let parts: Vec<&str> = ["address", "city", "state", "country"]
        .iter()
        .filter_map(|key| {
            addr.get(key)
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
        })
        .collect();
    if parts.is_empty() {
        None
    } else {
        Some(parts.join(", "))
    }
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
        assert!((lat - 48.8566_f64).abs() < f64::EPSILON);
        assert!((lng - 2.3522_f64).abs() < f64::EPSILON);

        let missing = json!({"latitude": "", "longitude": null});
        let (lat, lng) = extract_coords(&missing);
        assert!((lat).abs() < f64::EPSILON);
        assert!((lng).abs() < f64::EPSILON);

        let not_an_object = json!("Paris");
        let (lat, lng) = extract_coords(&not_an_object);
        assert!((lat).abs() < f64::EPSILON);
        assert!((lng).abs() < f64::EPSILON);
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

    #[test]
    fn strip_station_suffix_removes_trailing_number() {
        assert_eq!(strip_station_suffix("Belfast 1"), "Belfast");
        assert_eq!(strip_station_suffix("Dublin Connolly 1"), "Dublin Connolly");
        assert_eq!(strip_station_suffix("Platform 12"), "Platform");
    }

    #[test]
    fn strip_station_suffix_removes_trailing_single_letter() {
        assert_eq!(strip_station_suffix("Ballymote S"), "Ballymote");
    }

    #[test]
    fn strip_station_suffix_preserves_normal_names() {
        assert_eq!(strip_station_suffix("Gare du Nord"), "Gare du Nord");
        assert_eq!(strip_station_suffix("St Pancras"), "St Pancras");
        assert_eq!(strip_station_suffix("Dublin Connolly"), "Dublin Connolly");
        assert_eq!(strip_station_suffix(""), "");
    }

    #[test]
    fn extract_country_returns_lowercased_code() {
        let addr = json!({"country": "MA", "city": "Marrakech"});
        assert_eq!(extract_country(&addr).as_deref(), Some("ma"));
    }

    #[test]
    fn extract_country_returns_none_for_missing_or_empty() {
        assert_eq!(extract_country(&json!({})), None);
        assert_eq!(extract_country(&json!({"country": ""})), None);
        assert_eq!(extract_country(&json!({"country": null})), None);
        assert_eq!(extract_country(&Value::Null), None);
    }
}

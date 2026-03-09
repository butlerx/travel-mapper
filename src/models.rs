//! Data models for `TripIt` travel hops.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A single origin -> destination travel hop.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TravelHop {
    pub travel_type: TravelType,
    pub origin_name: String,
    pub origin_lat: Option<f64>,
    pub origin_lng: Option<f64>,
    pub dest_name: String,
    pub dest_lat: Option<f64>,
    pub dest_lng: Option<f64>,
    pub start_date: String,
    pub end_date: String,
}

/// The type of travel for a hop.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum TravelType {
    Air,
    Rail,
    Cruise,
    Transport,
}

impl std::fmt::Display for TravelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Air => write!(f, "air"),
            Self::Rail => write!(f, "rail"),
            Self::Cruise => write!(f, "cruise"),
            Self::Transport => write!(f, "transport"),
        }
    }
}

impl TravelType {
    #[must_use]
    pub const fn emoji(&self) -> &'static str {
        match self {
            Self::Air => "✈️",
            Self::Rail => "🚆",
            Self::Cruise => "🚢",
            Self::Transport => "🚗",
        }
    }
}

/// Attempt to parse a JSON value as f64, returning None on failure.
#[must_use]
pub fn coerce_float(val: &serde_json::Value) -> Option<f64> {
    match val {
        serde_json::Value::Number(n) => n.as_f64(),
        serde_json::Value::String(s) if !s.is_empty() => s.parse().ok(),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::coerce_float;
    use serde_json::json;

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
}

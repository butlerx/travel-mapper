//! Journey → trip_id → delay matching logic.
//!
//! Matches rail journey records from the database to GTFS trip_ids using
//! fuzzy station name matching and temporal matching, then extracts delay
//! information from GTFS-RT TripUpdate messages.

use super::gtfs_rt::TripUpdate;
use crate::db::hops::{RailDetail, Row};
use thiserror::Error;

/// Matching errors.
#[derive(Debug, Error)]
pub enum MatchError {
    #[error("No static GTFS data available for operator")]
    NoStaticData,

    #[error("Could not match station name: {0}")]
    StationNotFound(String),

    #[error("No trip found for journey parameters")]
    TripNotFound,
}

/// Candidate trip match from static GTFS.
#[derive(Debug, Clone)]
pub struct TripCandidate {
    pub trip_id: String,
    pub route_name: String,
    pub origin_stop_id: String,
    pub dest_stop_id: String,
    pub departure_time: String,
    pub service_date: String,
}

/// Result of matching a journey to a trip with delay information.
#[derive(Debug, Clone)]
pub struct JourneyMatch {
    pub hop_id: i64,
    pub trip_id: String,
    pub delay_seconds: Option<i32>,
    pub origin_stop_delay: Option<i32>,
    pub dest_stop_delay: Option<i32>,
}

/// Matches a journey record to a GTFS trip ID using static schedule data.
///
/// This is a placeholder that needs implementation after static GTFS caching
/// is built. The matching strategy:
///
/// 1. Fuzzy-match `origin_name` → `stop_id` (via stops.txt)
/// 2. Fuzzy-match `dest_name` → `stop_id` (via stops.txt)
/// 3. Parse `start_date` to YYYYMMDD service date
/// 4. Extract time from `start_date` timestamp
/// 5. Query trip schedules for matching route
/// 6. Return `trip_id` if found
///
/// # Errors
///
/// Returns an error if matching fails.
pub fn match_journey_to_trip_id(
    _journey: &Row,
    _rail_detail: Option<&RailDetail>,
) -> Result<String, MatchError> {
    // Placeholder: requires static GTFS cache implementation
    Err(MatchError::NoStaticData)
}

/// Finds delay information for a matched journey from GTFS-RT trip updates.
///
/// Given a `trip_id` and list of trip updates, extracts delay at origin and
/// destination stops, plus average trip delay.
#[must_use]
pub fn extract_delays_for_trip(
    trip_id: &str,
    origin_stop_id: &str,
    dest_stop_id: &str,
    trip_updates: &[TripUpdate],
) -> Option<(Option<i32>, Option<i32>, Option<i32>)> {
    let trip = trip_updates.iter().find(|t| t.trip_id == trip_id)?;

    let origin_delay = super::gtfs_rt::get_stop_delay(trip, origin_stop_id);
    let dest_delay = super::gtfs_rt::get_stop_delay(trip, dest_stop_id);
    let avg_delay = super::gtfs_rt::calculate_average_delay(trip);

    Some((origin_delay, dest_delay, avg_delay))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn match_journey_returns_no_static_data_error() {
        use crate::db::hops::TravelType;

        let journey = Row {
            id: 1,
            travel_type: TravelType::Rail,
            origin_name: "Paris Gare du Nord".to_string(),
            origin_lat: 48.881,
            origin_lng: 2.355,
            origin_country: Some("FR".to_string()),
            dest_name: "Brussels Midi".to_string(),
            dest_lat: 50.836,
            dest_lng: 4.337,
            dest_country: Some("BE".to_string()),
            start_date: "2026-03-26T10:00:00".to_string(),
            end_date: "2026-03-26T11:22:00".to_string(),
            raw_json: None,
            origin_address_query: None,
            dest_address_query: None,
            origin_tz: None,
            dest_tz: None,
            flight_detail: None,
            rail_detail: None,
            boat_detail: None,
            transport_detail: None,
            cost_amount: None,
            cost_currency: None,
            loyalty_program: None,
            miles_earned: None,
            cached_carrier: None,
        };

        let result = match_journey_to_trip_id(&journey, None);
        assert!(matches!(result, Err(MatchError::NoStaticData)));
    }

    #[test]
    fn extract_delays_returns_none_for_missing_trip() {
        let result = extract_delays_for_trip("nonexistent", "stop1", "stop2", &[]);
        assert_eq!(result, None);
    }
}

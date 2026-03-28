//! Journey → trip_id → delay matching logic.
//!
//! Matches rail journey records from the database to GTFS trip_ids using
//! fuzzy station name matching and temporal matching, then extracts delay
//! information from GTFS-RT TripUpdate messages.

use super::gtfs_rt::TripUpdate;

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
    fn extract_delays_returns_none_for_missing_trip() {
        let result = extract_delays_for_trip("nonexistent", "stop1", "stop2", &[]);
        assert_eq!(result, None);
    }
}

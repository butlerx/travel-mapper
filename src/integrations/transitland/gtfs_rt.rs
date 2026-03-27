//! GTFS-RT protobuf parsing for TripUpdate messages.
//!
//! Decodes protobuf-encoded GTFS-RT feeds and extracts delay information from
//! TripUpdate messages. Uses the `gtfs-realtime` crate for protobuf schema.

use gtfs_realtime::FeedMessage;
use prost::Message;
use thiserror::Error;

/// Error types for GTFS-RT parsing.
#[derive(Debug, Error)]
pub enum GtfsRtError {
    #[error("Failed to decode protobuf: {0}")]
    DecodeError(#[from] prost::DecodeError),

    #[error("Invalid trip descriptor: missing required field")]
    InvalidTripDescriptor,

    #[error("No trip updates found in feed")]
    NoTripUpdates,
}

/// Parsed trip update with delay information.
#[derive(Debug, Clone)]
pub struct TripUpdate {
    /// GTFS trip ID from static schedule.
    pub trip_id: String,
    /// Service date in YYYYMMDD format.
    pub start_date: Option<String>,
    /// Route ID from static GTFS.
    pub route_id: Option<String>,
    /// Stop-specific updates with delays.
    pub stop_time_updates: Vec<StopTimeUpdate>,
}

/// Delay information for a specific stop.
#[derive(Debug, Clone)]
pub struct StopTimeUpdate {
    /// GTFS stop ID from static schedule.
    pub stop_id: String,
    /// Arrival delay in seconds (positive = late, negative = early).
    pub arrival_delay: Option<i32>,
    /// Departure delay in seconds (positive = late, negative = early).
    pub departure_delay: Option<i32>,
}

/// Decodes a GTFS-RT protobuf feed and extracts trip updates.
///
/// # Errors
///
/// Returns an error if the protobuf cannot be decoded or contains invalid data.
pub fn decode_trip_updates(protobuf_bytes: &[u8]) -> Result<Vec<TripUpdate>, GtfsRtError> {
    let feed_message = FeedMessage::decode(protobuf_bytes)?;

    let mut trip_updates = Vec::new();

    for entity in feed_message.entity {
        if let Some(trip_update_data) = entity.trip_update {
            let trip_descriptor = trip_update_data.trip;

            let trip_id = trip_descriptor.trip_id.clone().unwrap_or_default();
            if trip_id.is_empty() {
                continue;
            }

            let mut stop_time_updates = Vec::new();
            for stu in trip_update_data.stop_time_update {
                let stop_id = stu.stop_id.unwrap_or_default();
                if stop_id.is_empty() {
                    continue;
                }

                let arrival_delay = stu.arrival.and_then(|a| a.delay);
                let departure_delay = stu.departure.and_then(|d| d.delay);

                stop_time_updates.push(StopTimeUpdate {
                    stop_id,
                    arrival_delay,
                    departure_delay,
                });
            }

            trip_updates.push(TripUpdate {
                trip_id,
                start_date: trip_descriptor.start_date.clone(),
                route_id: trip_descriptor.route_id.clone(),
                stop_time_updates,
            });
        }
    }

    if trip_updates.is_empty() {
        return Err(GtfsRtError::NoTripUpdates);
    }

    Ok(trip_updates)
}

/// Finds the average delay for a trip across all stops with delay information.
///
/// Returns `None` if no stop has delay data.
#[must_use]
pub fn calculate_average_delay(trip: &TripUpdate) -> Option<i32> {
    let delays: Vec<i32> = trip
        .stop_time_updates
        .iter()
        .filter_map(|stu| stu.departure_delay.or(stu.arrival_delay))
        .collect();

    if delays.is_empty() {
        return None;
    }

    let sum: i32 = delays.iter().sum();
    Some(sum / i32::try_from(delays.len()).ok()?)
}

/// Finds delay for a specific stop in a trip update.
///
/// Prefers departure delay over arrival delay if both are available.
#[must_use]
pub fn get_stop_delay(trip: &TripUpdate, stop_id: &str) -> Option<i32> {
    trip.stop_time_updates
        .iter()
        .find(|stu| stu.stop_id == stop_id)
        .and_then(|stu| stu.departure_delay.or(stu.arrival_delay))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_empty_feed_returns_error() {
        let empty_feed = FeedMessage::default();
        let bytes = empty_feed.encode_to_vec();
        let result = decode_trip_updates(&bytes);
        assert!(matches!(result, Err(GtfsRtError::NoTripUpdates)));
    }

    #[test]
    fn calculate_average_delay_returns_none_for_no_delays() {
        let trip = TripUpdate {
            trip_id: "test-trip".to_string(),
            start_date: None,
            route_id: None,
            stop_time_updates: vec![StopTimeUpdate {
                stop_id: "stop1".to_string(),
                arrival_delay: None,
                departure_delay: None,
            }],
        };

        assert_eq!(calculate_average_delay(&trip), None);
    }

    #[test]
    fn calculate_average_delay_computes_correctly() {
        let trip = TripUpdate {
            trip_id: "test-trip".to_string(),
            start_date: None,
            route_id: None,
            stop_time_updates: vec![
                StopTimeUpdate {
                    stop_id: "stop1".to_string(),
                    arrival_delay: Some(60),
                    departure_delay: Some(120),
                },
                StopTimeUpdate {
                    stop_id: "stop2".to_string(),
                    arrival_delay: Some(180),
                    departure_delay: None,
                },
            ],
        };

        // (120 + 180) / 2 = 150
        assert_eq!(calculate_average_delay(&trip), Some(150));
    }

    #[test]
    fn get_stop_delay_returns_none_for_missing_stop() {
        let trip = TripUpdate {
            trip_id: "test-trip".to_string(),
            start_date: None,
            route_id: None,
            stop_time_updates: vec![],
        };

        assert_eq!(get_stop_delay(&trip, "nonexistent"), None);
    }

    #[test]
    fn get_stop_delay_prefers_departure_over_arrival() {
        let trip = TripUpdate {
            trip_id: "test-trip".to_string(),
            start_date: None,
            route_id: None,
            stop_time_updates: vec![StopTimeUpdate {
                stop_id: "stop1".to_string(),
                arrival_delay: Some(60),
                departure_delay: Some(120),
            }],
        };

        assert_eq!(get_stop_delay(&trip, "stop1"), Some(120));
    }
}

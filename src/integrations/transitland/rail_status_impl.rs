use super::{cache::GtfsCache, client::TransitlandClient, feed_discovery::RailOperator};
use crate::integrations::rail_status::{
    RailStatus, RailStatusApi, RailStatusError, RailStatusQuery,
};
use sqlx::SqlitePool;

const OVAPI_TRIP_UPDATES_URL: &str = "https://gtfs.openov.nl/gtfs-rt/tripUpdates.pb";

/// Transitland-backed rail status client using GTFS-RT feeds.
pub struct TransitlandRailClient {
    client: TransitlandClient,
    cache: GtfsCache,
    pool: SqlitePool,
}

impl TransitlandRailClient {
    /// # Errors
    ///
    /// Returns an error if the underlying HTTP client cannot be initialized.
    pub fn new(api_key: String, pool: SqlitePool) -> Result<Self, reqwest::Error> {
        let client = TransitlandClient::new(api_key)?;
        let cache = GtfsCache::new(pool.clone(), client.clone());
        Ok(Self {
            client,
            cache,
            pool,
        })
    }

    async fn get_status_for_known_operator(
        &self,
        operator: RailOperator,
        query: &RailStatusQuery<'_>,
    ) -> Result<Option<RailStatus>, RailStatusError> {
        let departure_time = parse_departure_time(query.start_date).ok_or_else(|| {
            RailStatusError::Parse("invalid start_date: expected ISO-8601 datetime".to_string())
        })?;
        let service_date = parse_service_date(query.start_date).ok_or_else(|| {
            RailStatusError::Parse("invalid start_date: expected ISO-8601 datetime".to_string())
        })?;

        let feed_id = self
            .cache
            .ensure_feed_cached(&operator)
            .await
            .map_err(|e| RailStatusError::Parse(e.to_string()))?;

        let origin_matches = self
            .cache
            .find_stop_ids(feed_id, query.origin_name, 5)
            .await
            .map_err(|e| RailStatusError::Parse(e.to_string()))?;
        if origin_matches.is_empty() {
            return Ok(None);
        }

        let dest_matches = self
            .cache
            .find_stop_ids(feed_id, query.dest_name, 5)
            .await
            .map_err(|e| RailStatusError::Parse(e.to_string()))?;
        if dest_matches.is_empty() {
            return Ok(None);
        }

        let Some((trip_id, origin_stop_id, dest_stop_id)) = self
            .find_trip_candidate(
                feed_id,
                &origin_matches,
                &dest_matches,
                &departure_time,
                &service_date,
            )
            .await?
        else {
            return Ok(None);
        };

        let protobuf_bytes = self
            .client
            .fetch_trip_updates(operator.onestop_id())
            .await
            .map_err(RailStatusError::Http)?;
        let trip_updates = match super::gtfs_rt::decode_trip_updates(&protobuf_bytes) {
            Ok(updates) => updates,
            Err(super::gtfs_rt::GtfsRtError::NoTripUpdates) => return Ok(None),
            Err(err) => return Err(RailStatusError::Parse(err.to_string())),
        };

        let Some((origin_delay_seconds, dest_delay_seconds, avg_delay_seconds)) =
            super::matcher::extract_delays_for_trip(
                &trip_id,
                &origin_stop_id,
                &dest_stop_id,
                &trip_updates,
            )
        else {
            return Ok(None);
        };

        let matched_trip = trip_updates.iter().find(|u| u.trip_id == trip_id);
        let raw_json = serialize_trip_update(matched_trip)?;

        let dep_delay_minutes = origin_delay_seconds.map(delay_seconds_to_minutes);
        let arr_delay_minutes = dest_delay_seconds.map(delay_seconds_to_minutes);
        let status_delay_seconds = origin_delay_seconds
            .or(dest_delay_seconds)
            .or(avg_delay_seconds);

        Ok(Some(RailStatus {
            status: delay_to_status(status_delay_seconds),
            dep_delay_minutes,
            arr_delay_minutes,
            dep_platform: String::new(),
            arr_platform: String::new(),
            raw_json,
        }))
    }

    async fn find_trip_candidate(
        &self,
        feed_id: i64,
        origin_matches: &[super::cache::StopMatch],
        dest_matches: &[super::cache::StopMatch],
        departure_time: &str,
        service_date: &str,
    ) -> Result<Option<(String, String, String)>, RailStatusError> {
        for origin in origin_matches {
            for dest in dest_matches {
                let trip_matches = self
                    .cache
                    .find_trip_ids(
                        feed_id,
                        &origin.stop_id,
                        &dest.stop_id,
                        departure_time,
                        service_date,
                    )
                    .await
                    .map_err(|e| RailStatusError::Parse(e.to_string()))?;

                if let Some(trip) = trip_matches.first() {
                    return Ok(Some((
                        trip.trip_id.clone(),
                        origin.stop_id.clone(),
                        dest.stop_id.clone(),
                    )));
                }
            }
        }

        Ok(None)
    }

    async fn get_status_for_ovapi(
        &self,
        query: &RailStatusQuery<'_>,
    ) -> Result<Option<RailStatus>, RailStatusError> {
        let protobuf_bytes = reqwest::get(OVAPI_TRIP_UPDATES_URL)
            .await
            .map_err(RailStatusError::Http)?
            .bytes()
            .await
            .map_err(RailStatusError::Http)?;

        let trip_updates = match super::gtfs_rt::decode_trip_updates(&protobuf_bytes) {
            Ok(updates) => updates,
            Err(super::gtfs_rt::GtfsRtError::NoTripUpdates) => return Ok(None),
            Err(err) => return Err(RailStatusError::Parse(err.to_string())),
        };

        let service_date = parse_service_date(query.start_date);
        let matched_trip = trip_updates.iter().find(|trip| {
            matches_train_number(trip, query.train_number)
                && service_date
                    .as_ref()
                    .is_none_or(|date| trip.start_date.as_ref() == Some(date))
        });

        let Some(trip) = matched_trip else {
            return Ok(None);
        };

        let dep_delay_seconds = trip
            .stop_time_updates
            .first()
            .and_then(|s| s.departure_delay.or(s.arrival_delay));
        let arr_delay_seconds = trip
            .stop_time_updates
            .last()
            .and_then(|s| s.arrival_delay.or(s.departure_delay));
        let avg_delay_seconds = super::gtfs_rt::calculate_average_delay(trip);

        let dep_delay_minutes = dep_delay_seconds.map(delay_seconds_to_minutes);
        let arr_delay_minutes = arr_delay_seconds.map(delay_seconds_to_minutes);
        let status_delay_seconds = dep_delay_seconds
            .or(arr_delay_seconds)
            .or(avg_delay_seconds);

        Ok(Some(RailStatus {
            status: delay_to_status(status_delay_seconds),
            dep_delay_minutes,
            arr_delay_minutes,
            dep_platform: String::new(),
            arr_platform: String::new(),
            raw_json: serialize_trip_update(Some(trip))?,
        }))
    }
}

#[async_trait::async_trait]
impl RailStatusApi for TransitlandRailClient {
    fn provider_name(&self) -> &'static str {
        "transitland"
    }

    async fn get_rail_status(
        &self,
        query: &RailStatusQuery<'_>,
    ) -> Result<Option<RailStatus>, RailStatusError> {
        let _ = &self.pool;
        let country = query
            .origin_country
            .unwrap_or_default()
            .to_ascii_lowercase();

        if country == "nl" {
            return self.get_status_for_ovapi(query).await;
        }

        let Some(operator) = operator_for_country(&country) else {
            return Ok(None);
        };

        if operator == RailOperator::TerFrance
            && let Some(status) = self
                .get_status_for_known_operator(RailOperator::SncfTransilien, query)
                .await?
        {
            return Ok(Some(status));
        }

        self.get_status_for_known_operator(operator, query).await
    }
}

#[must_use]
fn operator_for_country(country: &str) -> Option<RailOperator> {
    match country {
        "fr" => Some(RailOperator::TerFrance),
        _ => None,
    }
}

#[must_use]
fn parse_departure_time(start_date: &str) -> Option<String> {
    let time_part = start_date
        .split('T')
        .nth(1)
        .or_else(|| start_date.split_whitespace().nth(1))?;
    let hhmmss = &time_part[..time_part.len().min(8)];

    if hhmmss.len() == 8
        && hhmmss.as_bytes()[2] == b':'
        && hhmmss.as_bytes()[5] == b':'
        && hhmmss.chars().enumerate().all(|(idx, c)| {
            if idx == 2 || idx == 5 {
                c == ':'
            } else {
                c.is_ascii_digit()
            }
        })
    {
        Some(hhmmss.to_string())
    } else {
        None
    }
}

#[must_use]
fn parse_service_date(start_date: &str) -> Option<String> {
    let date_part = start_date
        .split('T')
        .next()
        .or_else(|| start_date.split_whitespace().next())?;
    if date_part.len() < 10 {
        return None;
    }
    let yyyy_mm_dd = &date_part[..10];
    let mut output = String::with_capacity(8);
    for ch in yyyy_mm_dd.chars() {
        if ch.is_ascii_digit() {
            output.push(ch);
        }
    }
    if output.len() == 8 {
        Some(output)
    } else {
        None
    }
}

#[must_use]
fn delay_seconds_to_minutes(delay_seconds: i32) -> i64 {
    i64::from(delay_seconds) / 60
}

#[must_use]
fn delay_to_status(delay_seconds: Option<i32>) -> String {
    match delay_seconds {
        Some(delay) if delay > 0 => "delayed".to_string(),
        _ => "on_time".to_string(),
    }
}

#[must_use]
fn matches_train_number(trip: &super::gtfs_rt::TripUpdate, train_number: &str) -> bool {
    let train_number_trimmed = train_number.trim();
    if train_number_trimmed.is_empty() {
        return false;
    }

    let haystack = [
        trip.trip_id.as_str(),
        trip.route_id.as_deref().unwrap_or(""),
    ];
    haystack.iter().any(|value| {
        value
            .to_ascii_lowercase()
            .contains(&train_number_trimmed.to_ascii_lowercase())
    })
}

fn serialize_trip_update(
    trip: Option<&super::gtfs_rt::TripUpdate>,
) -> Result<String, RailStatusError> {
    let payload = trip.map_or_else(
        || serde_json::json!({}),
        |item| {
            serde_json::json!({
                "trip_id": item.trip_id,
                "start_date": item.start_date,
                "route_id": item.route_id,
                "stop_time_updates": item
                    .stop_time_updates
                    .iter()
                    .map(|update| {
                        serde_json::json!({
                            "stop_id": update.stop_id,
                            "arrival_delay": update.arrival_delay,
                            "departure_delay": update.departure_delay,
                        })
                    })
                    .collect::<Vec<_>>(),
            })
        },
    );

    serde_json::to_string(&payload).map_err(|e| RailStatusError::Parse(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::tests::test_pool;

    #[tokio::test]
    async fn test_provider_name() {
        let pool = test_pool().await;
        let client = TransitlandRailClient::new("test-api-key".to_string(), pool).unwrap();
        assert_eq!(client.provider_name(), "transitland");
    }

    #[test]
    fn test_operator_for_country() {
        assert_eq!(operator_for_country("fr"), Some(RailOperator::TerFrance));
        assert_eq!(operator_for_country("unknown"), None);
    }

    #[test]
    fn test_parse_departure_time() {
        assert_eq!(
            parse_departure_time("2026-03-26T10:00:00"),
            Some("10:00:00".to_string())
        );
    }

    #[test]
    fn test_parse_service_date() {
        assert_eq!(
            parse_service_date("2026-03-26T10:00:00"),
            Some("20260326".to_string())
        );
    }

    #[test]
    fn test_delay_to_status() {
        assert_eq!(delay_seconds_to_minutes(180), 3);
        assert_eq!(delay_seconds_to_minutes(0), 0);
        assert_eq!(delay_seconds_to_minutes(-120), -2);

        assert_eq!(delay_to_status(Some(120)), "delayed");
        assert_eq!(delay_to_status(Some(0)), "on_time");
        assert_eq!(delay_to_status(Some(-60)), "on_time");
        assert_eq!(delay_to_status(None), "on_time");
    }

    #[test]
    fn test_matches_train_number() {
        let trip = super::super::gtfs_rt::TripUpdate {
            trip_id: "NS-1234-A".to_string(),
            start_date: Some("20260326".to_string()),
            route_id: Some("SPRINTER-1234".to_string()),
            stop_time_updates: vec![],
        };

        assert!(matches_train_number(&trip, "1234"));
        assert!(!matches_train_number(&trip, "9999"));
    }
}

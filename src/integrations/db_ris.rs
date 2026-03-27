//! Deutsche Bahn RIS Journeys API client for German rail status enrichment.
//!
//! Queries the DB RIS `/find` endpoint by train number and date, then fuzzy-matches
//! origin/destination station names against the journey events to extract delay and
//! platform data. Station EVA numbers from responses are cached in SQLite for
//! future lookups.

use super::rail_status::{RailStatus, RailStatusApi, RailStatusError, RailStatusQuery};
use crate::db;
use governor::{Quota, RateLimiter, clock::DefaultClock, state::InMemoryState, state::NotKeyed};
use serde_json::Value;
use sqlx::SqlitePool;
use std::num::NonZeroU32;

const DB_RIS_API_BASE: &str =
    "https://apis.deutschebahn.com/db-api-marketplace/apis/ris-journeys-netz/v2";

const TRANSPORT_TYPES: &str = "HIGH_SPEED_TRAIN,INTERCITY_TRAIN,INTER_REGIONAL_TRAIN,REGIONAL_TRAIN,CITY_TRAIN,SUBWAY,TRAM,BUS";

const MAX_RETRIES: u32 = 3;
const REQUEST_TIMEOUT_SECS: u64 = 30;

/// Minimum Jaro-Winkler similarity for a station name to be considered a match.
const FUZZY_MATCH_THRESHOLD: f64 = 0.85;

/// HTTP client for the Deutsche Bahn RIS Journeys API.
pub struct DbRisClient {
    api_key: String,
    client_id: String,
    client: reqwest::Client,
    base_url: String,
    pool: SqlitePool,
    rate_limiter: RateLimiter<NotKeyed, InMemoryState, DefaultClock>,
}

impl DbRisClient {
    #[must_use]
    pub fn new(api_key: String, client_id: String, pool: SqlitePool) -> Self {
        let client = reqwest::Client::new();
        Self::with_base_url(
            api_key,
            client_id,
            client,
            pool,
            DB_RIS_API_BASE.to_string(),
        )
    }

    /// # Panics
    ///
    /// Panics if the rate limiter quota cannot be constructed (should never happen
    /// since the quota value is a compile-time constant).
    #[must_use]
    pub fn with_base_url(
        api_key: String,
        client_id: String,
        client: reqwest::Client,
        pool: SqlitePool,
        base_url: String,
    ) -> Self {
        // 60 requests per minute on the free tier.
        let quota = Quota::per_minute(NonZeroU32::new(60).expect("60 > 0"));
        Self {
            api_key,
            client_id,
            client,
            base_url,
            pool,
            rate_limiter: RateLimiter::direct(quota),
        }
    }

    /// Send a GET request with DB RIS auth headers, retrying on transient errors.
    async fn get(&self, url: &str) -> Result<Value, RailStatusError> {
        if self.rate_limiter.check().is_err() {
            return Err(RailStatusError::RateLimited);
        }

        tracing::debug!(url, "DB RIS GET request");

        let mut attempt = 0;

        loop {
            attempt += 1;
            let result = self
                .client
                .get(url)
                .header("DB-Api-Key", &self.api_key)
                .header("DB-Client-Id", &self.client_id)
                .timeout(std::time::Duration::from_secs(REQUEST_TIMEOUT_SECS))
                .send()
                .await;

            match result {
                Ok(resp) => {
                    let status = resp.status();
                    if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                        return Err(RailStatusError::RateLimited);
                    }
                    if status.is_server_error() && attempt <= MAX_RETRIES {
                        let delay = std::time::Duration::from_millis(
                            500 * u64::from(2_u32.pow(attempt - 1)),
                        );
                        tracing::warn!(
                            url,
                            %status,
                            delay_ms = u64::try_from(delay.as_millis()).unwrap_or(u64::MAX),
                            attempt,
                            MAX_RETRIES,
                            "retrying after server error",
                        );
                        tokio::time::sleep(delay).await;
                        continue;
                    }
                    resp.error_for_status_ref().map_err(RailStatusError::Http)?;
                    let body = resp.bytes().await.map_err(RailStatusError::Http)?;
                    return serde_json::from_slice(&body)
                        .map_err(|e| RailStatusError::Parse(e.to_string()));
                }
                Err(err) => {
                    if (err.is_connect() || err.is_timeout()) && attempt <= MAX_RETRIES {
                        let delay = std::time::Duration::from_millis(
                            500 * u64::from(2_u32.pow(attempt - 1)),
                        );
                        tracing::warn!(
                            url,
                            error = %err,
                            delay_ms = u64::try_from(delay.as_millis()).unwrap_or(u64::MAX),
                            attempt,
                            MAX_RETRIES,
                            "retrying after connection error",
                        );
                        tokio::time::sleep(delay).await;
                        continue;
                    }
                    return Err(RailStatusError::Http(err));
                }
            }
        }
    }

    /// Cache EVA station entries from a journey's events into `SQLite`.
    async fn cache_eva_stations(&self, events: &[Value]) {
        for event in events {
            let Some(station) = event.get("station") else {
                continue;
            };
            let Some(eva_number) = station.get("evaNumber").and_then(Value::as_i64) else {
                continue;
            };
            let Some(name) = station.get("name").and_then(Value::as_str) else {
                continue;
            };
            let lat = station
                .get("position")
                .and_then(|p| p.get("latitude"))
                .and_then(Value::as_f64);
            let lng = station
                .get("position")
                .and_then(|p| p.get("longitude"))
                .and_then(Value::as_f64);

            if let Err(err) = (db::station_eva_cache::Upsert {
                eva_number,
                name,
                lat,
                lng,
            })
            .execute(&self.pool)
            .await
            {
                tracing::debug!(eva_number, name, error = %err, "failed to cache EVA station");
            }
        }
    }
}

/// Normalise a station name for fuzzy comparison: transliterate unicode to ASCII
/// and lowercase.
fn normalise_station_name(name: &str) -> String {
    deunicode::deunicode(name).to_lowercase()
}

/// Find the event index whose station name best matches `target_name` above the
/// fuzzy threshold. Returns the event `Value` reference.
fn find_station_event<'a>(events: &'a [Value], target_name: &str) -> Option<&'a Value> {
    let normalised_target = normalise_station_name(target_name);

    let mut best_score = 0.0_f64;
    let mut best_event: Option<&Value> = None;

    for event in events {
        let station_name = event
            .get("station")
            .and_then(|s| s.get("name"))
            .and_then(Value::as_str)
            .unwrap_or_default();

        let normalised = normalise_station_name(station_name);
        let score = strsim::jaro_winkler(&normalised, &normalised_target);

        if score > best_score {
            best_score = score;
            best_event = Some(event);
        }
    }

    if best_score >= FUZZY_MATCH_THRESHOLD {
        best_event
    } else {
        None
    }
}

/// Extract the platform string from an event, preferring actual over scheduled.
fn extract_platform(event: &Value) -> String {
    event
        .get("platform")
        .and_then(|p| {
            p.get("actual")
                .and_then(Value::as_str)
                .or_else(|| p.get("scheduled").and_then(Value::as_str))
        })
        .unwrap_or_default()
        .to_string()
}

/// Derive a high-level status string from event data.
fn derive_status(origin_event: Option<&Value>, dest_event: Option<&Value>) -> String {
    let cancelled = origin_event
        .and_then(|e| e.get("cancelled"))
        .and_then(Value::as_bool)
        .unwrap_or(false)
        || dest_event
            .and_then(|e| e.get("cancelled"))
            .and_then(Value::as_bool)
            .unwrap_or(false);

    if cancelled {
        return "cancelled".to_string();
    }

    let dep_delay = origin_event
        .and_then(|e| e.get("departureDelay"))
        .and_then(Value::as_i64);
    let arr_delay = dest_event
        .and_then(|e| e.get("arrivalDelay"))
        .and_then(Value::as_i64);

    let max_delay = dep_delay.into_iter().chain(arr_delay).max();
    match max_delay {
        Some(d) if d > 0 => "delayed".to_string(),
        _ => "on_time".to_string(),
    }
}

#[async_trait::async_trait]
impl RailStatusApi for DbRisClient {
    fn provider_name(&self) -> &'static str {
        "db_ris"
    }

    async fn get_rail_status(
        &self,
        query: &RailStatusQuery<'_>,
    ) -> Result<Option<RailStatus>, RailStatusError> {
        // Extract YYYY-MM-DD from the start_date (which may include a time component).
        let date = query
            .start_date
            .split_whitespace()
            .next()
            .unwrap_or(query.start_date);

        let url = format!(
            "{}/find?journeyNumber={}&transportTypes={TRANSPORT_TYPES}&date={date}",
            self.base_url, query.train_number,
        );

        let response = self.get(&url).await?;

        let journeys = response
            .get("journeys")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();

        if journeys.is_empty() {
            return Ok(None);
        }

        let journey = &journeys[0];

        let events = journey
            .get("events")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();

        if events.is_empty() {
            return Ok(None);
        }

        self.cache_eva_stations(&events).await;

        let origin_event = find_station_event(&events, query.origin_name);
        let dest_event = find_station_event(&events, query.dest_name);

        if origin_event.is_none() && dest_event.is_none() {
            tracing::debug!(
                train_number = query.train_number,
                origin = query.origin_name,
                dest = query.dest_name,
                "no station match in DB RIS journey events",
            );
            return Ok(None);
        }

        let status = derive_status(origin_event, dest_event);

        let dep_delay_minutes = origin_event
            .and_then(|e| e.get("departureDelay"))
            .and_then(Value::as_i64);
        let arr_delay_minutes = dest_event
            .and_then(|e| e.get("arrivalDelay"))
            .and_then(Value::as_i64);

        let dep_platform = origin_event.map_or_else(String::new, extract_platform);
        let arr_platform = dest_event.map_or_else(String::new, extract_platform);

        let raw_json =
            serde_json::to_string(journey).map_err(|e| RailStatusError::Parse(e.to_string()))?;

        Ok(Some(RailStatus {
            status,
            dep_delay_minutes,
            arr_delay_minutes,
            dep_platform,
            arr_platform,
            raw_json,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::tests::test_pool;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU32, Ordering};
    use tokio::io::AsyncWriteExt;

    fn sample_journey_json() -> String {
        r#"{
            "journeys": [{
                "journeyID": "20260327_ICE_101",
                "events": [
                    {
                        "station": {
                            "name": "Frankfurt(Main)Hbf",
                            "evaNumber": 8000105,
                            "position": {"latitude": 50.1071, "longitude": 8.6636}
                        },
                        "departureDelay": 5,
                        "arrivalDelay": null,
                        "platform": {"scheduled": "3", "actual": "5"},
                        "cancelled": false
                    },
                    {
                        "station": {
                            "name": "München Hbf",
                            "evaNumber": 8000261,
                            "position": {"latitude": 48.1402, "longitude": 11.5600}
                        },
                        "departureDelay": null,
                        "arrivalDelay": 3,
                        "platform": {"scheduled": "12", "actual": null},
                        "cancelled": false
                    }
                ]
            }]
        }"#
        .to_string()
    }

    fn sample_query() -> RailStatusQuery<'static> {
        RailStatusQuery {
            carrier: "DB",
            train_number: "101",
            origin_name: "Frankfurt Hbf",
            dest_name: "Muenchen Hbf",
            origin_country: Some("de"),
            dest_country: Some("de"),
            start_date: "2026-03-27",
            end_date: "2026-03-27",
            origin_lat: 50.1071,
            origin_lng: 8.6636,
            dest_lat: 48.1402,
            dest_lng: 11.5600,
        }
    }

    #[tokio::test]
    async fn get_rail_status_returns_delayed() {
        let pool = test_pool().await;
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut buf = vec![0_u8; 4096];
            let _ = tokio::io::AsyncReadExt::read(&mut stream, &mut buf).await;

            let body = sample_journey_json();
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
                body.len()
            );
            stream.write_all(response.as_bytes()).await.unwrap();
        });

        let client = DbRisClient::with_base_url(
            "test_key".to_string(),
            "test_id".to_string(),
            reqwest::Client::new(),
            pool.clone(),
            format!("http://127.0.0.1:{port}"),
        );

        let query = sample_query();
        let result = client.get_rail_status(&query).await.unwrap();
        let status = result.expect("expected a rail status record");

        assert_eq!(status.status, "delayed");
        assert_eq!(status.dep_delay_minutes, Some(5));
        assert_eq!(status.arr_delay_minutes, Some(3));
        assert_eq!(status.dep_platform, "5");
        assert_eq!(status.arr_platform, "12");

        let raw: Value = serde_json::from_str(&status.raw_json).unwrap();
        assert!(raw.get("journeyID").is_some());

        // Verify EVA cache was populated.
        let cached = db::station_eva_cache::GetByName {
            name: "Frankfurt(Main)Hbf",
        }
        .execute(&pool)
        .await
        .unwrap();
        assert!(cached.is_some(), "EVA cache should be populated");
        assert_eq!(cached.unwrap().eva_number, 8_000_105);
    }

    #[tokio::test]
    async fn get_rail_status_empty_journeys_returns_none() {
        let pool = test_pool().await;
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut buf = vec![0_u8; 4096];
            let _ = tokio::io::AsyncReadExt::read(&mut stream, &mut buf).await;

            let body = r#"{"journeys":[]}"#;
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
                body.len()
            );
            stream.write_all(response.as_bytes()).await.unwrap();
        });

        let client = DbRisClient::with_base_url(
            "test_key".to_string(),
            "test_id".to_string(),
            reqwest::Client::new(),
            pool,
            format!("http://127.0.0.1:{port}"),
        );

        let query = sample_query();
        let result = client.get_rail_status(&query).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn get_rail_status_rate_limit_returns_error() {
        let pool = test_pool().await;
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut buf = vec![0_u8; 4096];
            let _ = tokio::io::AsyncReadExt::read(&mut stream, &mut buf).await;

            let response = "HTTP/1.1 429 Too Many Requests\r\nContent-Length: 0\r\n\r\n";
            stream.write_all(response.as_bytes()).await.unwrap();
        });

        let client = DbRisClient::with_base_url(
            "test_key".to_string(),
            "test_id".to_string(),
            reqwest::Client::new(),
            pool,
            format!("http://127.0.0.1:{port}"),
        );

        let query = sample_query();
        let result = client.get_rail_status(&query).await;
        assert!(
            matches!(result, Err(RailStatusError::RateLimited)),
            "expected RateLimited error, got: {result:?}"
        );
    }

    #[tokio::test]
    async fn get_rail_status_retries_on_server_error() {
        let pool = test_pool().await;
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
                    "HTTP/1.1 500 Internal Server Error\r\nContent-Length: 0\r\n\r\n".to_string()
                } else {
                    let body = sample_journey_json();
                    format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
                        body.len()
                    )
                };

                stream.write_all(response.as_bytes()).await.unwrap();
            }
        });

        let client = DbRisClient::with_base_url(
            "test_key".to_string(),
            "test_id".to_string(),
            reqwest::Client::new(),
            pool,
            format!("http://127.0.0.1:{port}"),
        );

        let query = sample_query();
        let result = client.get_rail_status(&query).await;
        assert!(
            result.is_ok(),
            "expected success after retries, got: {result:?}"
        );
        assert_eq!(attempt_count.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn get_rail_status_no_station_match_returns_none() {
        let pool = test_pool().await;
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut buf = vec![0_u8; 4096];
            let _ = tokio::io::AsyncReadExt::read(&mut stream, &mut buf).await;

            // Journey events have completely different stations.
            let body = r#"{"journeys":[{"journeyID":"test","events":[{"station":{"name":"Berlin Hbf","evaNumber":8011160},"departureDelay":0,"platform":{"scheduled":"1"},"cancelled":false}]}]}"#;
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
                body.len()
            );
            stream.write_all(response.as_bytes()).await.unwrap();
        });

        let client = DbRisClient::with_base_url(
            "test_key".to_string(),
            "test_id".to_string(),
            reqwest::Client::new(),
            pool,
            format!("http://127.0.0.1:{port}"),
        );

        // Query for Frankfurt → München but response only has Berlin.
        let query = sample_query();
        let result = client.get_rail_status(&query).await.unwrap();
        assert!(result.is_none(), "expected None for unmatched stations");
    }

    #[test]
    fn fuzzy_match_umlaut_station_names() {
        // "Muenchen Hbf" (ASCII) should match "München Hbf" (unicode) via deunicode.
        let events: Vec<Value> = serde_json::from_str(
            r#"[{"station":{"name":"München Hbf","evaNumber":8000261},"departureDelay":0,"platform":{"scheduled":"12"},"cancelled":false}]"#,
        ).unwrap();

        let matched = find_station_event(&events, "Muenchen Hbf");
        assert!(matched.is_some(), "should fuzzy-match umlaut station name");
    }

    #[test]
    fn fuzzy_match_parenthetical_station_names() {
        // "Frankfurt Hbf" should match "Frankfurt(Main)Hbf" with sufficient similarity.
        let events: Vec<Value> = serde_json::from_str(
            r#"[{"station":{"name":"Frankfurt(Main)Hbf","evaNumber":8000105},"departureDelay":5,"platform":{"scheduled":"3","actual":"5"},"cancelled":false}]"#,
        ).unwrap();

        let matched = find_station_event(&events, "Frankfurt Hbf");
        assert!(
            matched.is_some(),
            "should fuzzy-match parenthetical station name"
        );
    }

    #[test]
    fn derive_status_cancelled() {
        let event: Value =
            serde_json::from_str(r#"{"cancelled":true,"departureDelay":null}"#).unwrap();
        assert_eq!(derive_status(Some(&event), None), "cancelled");
    }

    #[test]
    fn derive_status_on_time() {
        let event: Value =
            serde_json::from_str(r#"{"cancelled":false,"departureDelay":0}"#).unwrap();
        assert_eq!(derive_status(Some(&event), Some(&event)), "on_time");
    }

    #[test]
    fn derive_status_delayed() {
        let origin: Value =
            serde_json::from_str(r#"{"cancelled":false,"departureDelay":10}"#).unwrap();
        let dest: Value = serde_json::from_str(r#"{"cancelled":false,"arrivalDelay":5}"#).unwrap();
        assert_eq!(derive_status(Some(&origin), Some(&dest)), "delayed");
    }

    #[test]
    fn extract_platform_prefers_actual() {
        let event: Value =
            serde_json::from_str(r#"{"platform":{"scheduled":"3","actual":"5"}}"#).unwrap();
        assert_eq!(extract_platform(&event), "5");
    }

    #[test]
    fn extract_platform_falls_back_to_scheduled() {
        let event: Value =
            serde_json::from_str(r#"{"platform":{"scheduled":"3","actual":null}}"#).unwrap();
        assert_eq!(extract_platform(&event), "3");
    }

    #[test]
    fn extract_platform_missing_returns_empty() {
        let event: Value = serde_json::from_str(r"{}").unwrap();
        assert_eq!(extract_platform(&event), "");
    }

    #[test]
    fn normalise_station_name_handles_umlauts() {
        assert_eq!(normalise_station_name("München Hbf"), "munchen hbf");
        assert_eq!(normalise_station_name("Düsseldorf"), "dusseldorf");
        assert_eq!(normalise_station_name("Zürich HB"), "zurich hb");
    }
}

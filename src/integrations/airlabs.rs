//! AirLabs flight status API client.
//!
//! Uses the `/schedules` endpoint with client-side date filtering to get
//! accurate status for historical flights (the `/flight` endpoint only returns
//! the closest match without date discrimination).

use super::flight_status::{FlightStatus, FlightStatusApi, FlightStatusError};
use serde_json::Value;

const AIRLABS_API_BASE: &str = "https://airlabs.co/api/v9";

/// HTTP client for the `AirLabs` flight status API.
pub struct AirLabsClient {
    api_key: String,
    client: reqwest::Client,
    base_url: String,
}

impl AirLabsClient {
    /// Creates a new client with the given API key, using the default `AirLabs` base URL.
    #[must_use]
    pub fn new(api_key: String) -> Self {
        let client = reqwest::Client::new();
        Self::with_base_url(api_key, client, AIRLABS_API_BASE.to_string())
    }

    /// Creates a new client with a custom base URL and HTTP client (useful for testing).
    #[must_use]
    pub fn with_base_url(api_key: String, client: reqwest::Client, base_url: String) -> Self {
        Self {
            api_key,
            client,
            base_url,
        }
    }

    async fn get(&self, url: &str) -> Result<Value, FlightStatusError> {
        tracing::debug!(url, "GET request");

        let max_retries: u32 = 3;
        let mut attempt = 0;

        loop {
            attempt += 1;
            let result = self
                .client
                .get(url)
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
                            url,
                            %status,
                            delay_ms = u64::try_from(delay.as_millis()).unwrap_or(u64::MAX),
                            attempt,
                            max_retries,
                            "retrying after server error",
                        );
                        tokio::time::sleep(delay).await;
                        continue;
                    }
                    if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                        return Err(FlightStatusError::RateLimited);
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
                            url,
                            error = %err,
                            delay_ms = u64::try_from(delay.as_millis()).unwrap_or(u64::MAX),
                            attempt,
                            max_retries,
                            "retrying after connection error",
                        );
                        tokio::time::sleep(delay).await;
                        continue;
                    }
                    return Err(FlightStatusError::Http(err));
                }
            }
        }
    }
}

/// Find the schedule entry whose `dep_time` or `dep_time_utc` matches the
/// requested date. Returns `None` when no entry matches.
fn find_matching_schedule<'a>(schedules: &'a [Value], flight_date: &str) -> Option<&'a Value> {
    schedules.iter().find(|entry| {
        let dep_local = entry
            .get("dep_time")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let dep_utc = entry
            .get("dep_time_utc")
            .and_then(Value::as_str)
            .unwrap_or_default();
        dep_local.starts_with(flight_date) || dep_utc.starts_with(flight_date)
    })
}

/// Extract a [`FlightStatus`] from a single `AirLabs` schedule/flight JSON object.
fn parse_flight_entry(entry: &Value) -> Result<FlightStatus, FlightStatusError> {
    let status = entry
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();

    let dep_delay_minutes = entry.get("dep_delayed").and_then(Value::as_i64);
    let arr_delay_minutes = entry.get("arr_delayed").and_then(Value::as_i64);

    let dep_gate = entry
        .get("dep_gate")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let dep_terminal = entry
        .get("dep_terminal")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let arr_gate = entry
        .get("arr_gate")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let arr_terminal = entry
        .get("arr_terminal")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();

    let raw_json = serde_json::to_string(entry)?;

    Ok(FlightStatus {
        status,
        dep_delay_minutes,
        arr_delay_minutes,
        dep_gate,
        dep_terminal,
        arr_gate,
        arr_terminal,
        raw_json,
    })
}

#[async_trait::async_trait]
impl FlightStatusApi for AirLabsClient {
    async fn get_flight_status(
        &self,
        flight_iata: &str,
        flight_date: &str,
    ) -> Result<Option<FlightStatus>, FlightStatusError> {
        let url = format!(
            "{}/schedules?flight_iata={flight_iata}&api_key={}",
            self.base_url, self.api_key,
        );
        let response = self.get(&url).await?;

        let schedules = response
            .get("response")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();

        if schedules.is_empty() {
            return Ok(None);
        }

        let Some(entry) = find_matching_schedule(&schedules, flight_date) else {
            return Ok(None);
        };

        parse_flight_entry(entry).map(Some)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU32, Ordering};
    use tokio::io::AsyncWriteExt;

    #[tokio::test]
    async fn get_flight_status_returns_landed() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut buf = vec![0_u8; 4096];
            let _ = tokio::io::AsyncReadExt::read(&mut stream, &mut buf).await;

            let body = r#"{"response":[{"flight_iata":"AA100","dep_time":"2026-03-25 08:00","dep_time_utc":"2026-03-25 13:00","status":"landed","dep_delayed":15,"arr_delayed":7,"dep_gate":"A1","dep_terminal":"5","arr_gate":"B3","arr_terminal":"2"}]}"#;
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
                body.len()
            );

            stream.write_all(response.as_bytes()).await.unwrap();
        });

        let client = AirLabsClient::with_base_url(
            "test_key".to_string(),
            reqwest::Client::new(),
            format!("http://127.0.0.1:{port}"),
        );

        let result = client
            .get_flight_status("AA100", "2026-03-25")
            .await
            .unwrap();
        let status = result.expect("expected a flight status record");

        assert_eq!(status.status, "landed");
        assert_eq!(status.dep_delay_minutes, Some(15));
        assert_eq!(status.arr_delay_minutes, Some(7));
        assert_eq!(status.dep_gate, "A1");
        assert_eq!(status.dep_terminal, "5");
        assert_eq!(status.arr_gate, "B3");
        assert_eq!(status.arr_terminal, "2");
        let raw_json: Value = serde_json::from_str(&status.raw_json).unwrap();
        assert_eq!(raw_json["status"], "landed");
        assert_eq!(raw_json["dep_delayed"], 15);
    }

    #[tokio::test]
    async fn get_flight_status_empty_response_returns_none() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut buf = vec![0_u8; 4096];
            let _ = tokio::io::AsyncReadExt::read(&mut stream, &mut buf).await;

            let body = r#"{"response":[]}"#;
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
                body.len()
            );

            stream.write_all(response.as_bytes()).await.unwrap();
        });

        let client = AirLabsClient::with_base_url(
            "test_key".to_string(),
            reqwest::Client::new(),
            format!("http://127.0.0.1:{port}"),
        );

        let result = client
            .get_flight_status("AA100", "2026-03-25")
            .await
            .unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn get_flight_status_no_matching_date_returns_none() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut buf = vec![0_u8; 4096];
            let _ = tokio::io::AsyncReadExt::read(&mut stream, &mut buf).await;

            // Schedules exist but none match the requested date.
            let body = r#"{"response":[{"flight_iata":"AA100","dep_time":"2026-03-26 08:00","dep_time_utc":"2026-03-26 13:00","status":"scheduled"}]}"#;
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
                body.len()
            );

            stream.write_all(response.as_bytes()).await.unwrap();
        });

        let client = AirLabsClient::with_base_url(
            "test_key".to_string(),
            reqwest::Client::new(),
            format!("http://127.0.0.1:{port}"),
        );

        let result = client
            .get_flight_status("AA100", "2026-03-25")
            .await
            .unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn get_flight_status_retries_on_server_error() {
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
                    let body = r#"{"response":[{"flight_iata":"AA100","dep_time":"2026-03-25 08:00","dep_time_utc":"2026-03-25 13:00","status":"landed","dep_delayed":1,"arr_delayed":2,"dep_gate":"A1","dep_terminal":"1","arr_gate":"B1","arr_terminal":"2"}]}"#;
                    format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
                        body.len()
                    )
                };

                stream.write_all(response.as_bytes()).await.unwrap();
            }
        });

        let client = AirLabsClient::with_base_url(
            "test_key".to_string(),
            reqwest::Client::new(),
            format!("http://127.0.0.1:{port}"),
        );

        let result = client.get_flight_status("AA100", "2026-03-25").await;
        assert!(
            result.is_ok(),
            "expected success after retries, got: {result:?}"
        );
        assert_eq!(attempt_count.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn find_matching_schedule_matches_local_time() {
        let schedules: Vec<Value> = serde_json::from_str(
            r#"[{"dep_time":"2026-03-25 08:00","dep_time_utc":"2026-03-25 13:00","status":"landed"}]"#,
        )
        .unwrap();
        assert!(find_matching_schedule(&schedules, "2026-03-25").is_some());
        assert!(find_matching_schedule(&schedules, "2026-03-26").is_none());
    }

    #[test]
    fn find_matching_schedule_matches_utc_time() {
        let schedules: Vec<Value> = serde_json::from_str(
            r#"[{"dep_time":"2026-03-24 23:30","dep_time_utc":"2026-03-25 04:30","status":"landed"}]"#,
        )
        .unwrap();
        // Matches on UTC date even though local date is different.
        assert!(find_matching_schedule(&schedules, "2026-03-25").is_some());
    }

    #[test]
    fn parse_flight_entry_handles_nulls() {
        let entry: Value = serde_json::from_str(
            r#"{"status":"scheduled","dep_delayed":null,"arr_delayed":null,"dep_gate":null,"dep_terminal":null,"arr_gate":null,"arr_terminal":null}"#,
        )
        .unwrap();
        let status = parse_flight_entry(&entry).unwrap();
        assert_eq!(status.status, "scheduled");
        assert_eq!(status.dep_delay_minutes, None);
        assert_eq!(status.arr_delay_minutes, None);
        assert_eq!(status.dep_gate, "");
        assert_eq!(status.dep_terminal, "");
        assert_eq!(status.arr_gate, "");
        assert_eq!(status.arr_terminal, "");
    }
}

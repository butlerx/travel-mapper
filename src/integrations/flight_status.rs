//! AviationStack flight status API client.

use serde_json::Value;
use thiserror::Error;

const AVIATIONSTACK_API_BASE: &str = "https://api.aviationstack.com/v1";

#[derive(Debug, Error)]
pub enum FlightStatusError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("failed to parse API response: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Clone)]
pub struct FlightStatus {
    pub flight_status: String,
    pub dep_delay_minutes: Option<i64>,
    pub arr_delay_minutes: Option<i64>,
    pub dep_gate: String,
    pub dep_terminal: String,
    pub arr_gate: String,
    pub arr_terminal: String,
    pub raw_json: String,
}

#[async_trait::async_trait]
pub trait FlightStatusApi: Send + Sync {
    /// # Errors
    ///
    /// Returns an error if the request fails, the response status is non-success,
    /// or the JSON body cannot be parsed.
    async fn get_flight_status(
        &self,
        flight_iata: &str,
        flight_date: &str,
    ) -> Result<Option<FlightStatus>, FlightStatusError>;
}

pub struct AviationStackClient {
    api_key: String,
    client: reqwest::Client,
    base_url: String,
}

impl AviationStackClient {
    #[must_use]
    pub fn new(api_key: String) -> Self {
        let client = reqwest::Client::new();
        Self::with_base_url(api_key, client, AVIATIONSTACK_API_BASE.to_string())
    }

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

#[async_trait::async_trait]
impl FlightStatusApi for AviationStackClient {
    async fn get_flight_status(
        &self,
        flight_iata: &str,
        flight_date: &str,
    ) -> Result<Option<FlightStatus>, FlightStatusError> {
        let url = format!(
            "{}/flights?access_key={}&flight_iata={flight_iata}&flight_date={flight_date}",
            self.base_url, self.api_key,
        );
        let response = self.get(&url).await?;

        let first = response
            .get("data")
            .and_then(Value::as_array)
            .and_then(|data| data.first());

        let Some(first_entry) = first else {
            return Ok(None);
        };

        let departure = first_entry.get("departure");
        let arrival = first_entry.get("arrival");

        let status = first_entry
            .get("flight_status")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        let dep_delay_minutes = departure
            .and_then(|dep| dep.get("delay"))
            .and_then(Value::as_i64);
        let arr_delay_minutes = arrival
            .and_then(|arr| arr.get("delay"))
            .and_then(Value::as_i64);
        let dep_gate = departure
            .and_then(|dep| dep.get("gate"))
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        let dep_terminal = departure
            .and_then(|dep| dep.get("terminal"))
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        let arr_gate = arrival
            .and_then(|arr| arr.get("gate"))
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        let arr_terminal = arrival
            .and_then(|arr| arr.get("terminal"))
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        let raw_json = serde_json::to_string(first_entry)?;

        Ok(Some(FlightStatus {
            flight_status: status,
            dep_delay_minutes,
            arr_delay_minutes,
            dep_gate,
            dep_terminal,
            arr_gate,
            arr_terminal,
            raw_json,
        }))
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

            let body = r#"{"data":[{"flight_status":"landed","departure":{"delay":15,"gate":"A1","terminal":"5"},"arrival":{"delay":7,"gate":"B3","terminal":"2"}}]}"#;
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
                body.len()
            );

            stream.write_all(response.as_bytes()).await.unwrap();
        });

        let client = AviationStackClient::with_base_url(
            "test_key".to_string(),
            reqwest::Client::new(),
            format!("http://127.0.0.1:{port}"),
        );

        let result = client
            .get_flight_status("AA100", "2026-03-25")
            .await
            .unwrap();
        let status = result.expect("expected a flight status record");

        assert_eq!(status.flight_status, "landed");
        assert_eq!(status.dep_delay_minutes, Some(15));
        assert_eq!(status.arr_delay_minutes, Some(7));
        assert_eq!(status.dep_gate, "A1");
        assert_eq!(status.dep_terminal, "5");
        assert_eq!(status.arr_gate, "B3");
        assert_eq!(status.arr_terminal, "2");
        let raw_json: Value = serde_json::from_str(&status.raw_json).unwrap();
        let expected = serde_json::json!({
            "flight_status": "landed",
            "departure": {
                "delay": 15,
                "gate": "A1",
                "terminal": "5"
            },
            "arrival": {
                "delay": 7,
                "gate": "B3",
                "terminal": "2"
            }
        });
        assert_eq!(raw_json, expected);
    }

    #[tokio::test]
    async fn get_flight_status_empty_data_returns_none() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut buf = vec![0_u8; 4096];
            let _ = tokio::io::AsyncReadExt::read(&mut stream, &mut buf).await;

            let body = r#"{"data":[]}"#;
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
                body.len()
            );

            stream.write_all(response.as_bytes()).await.unwrap();
        });

        let client = AviationStackClient::with_base_url(
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
                    let body = r#"{"data":[{"flight_status":"landed","departure":{"delay":1,"gate":"A1","terminal":"1"},"arrival":{"delay":2,"gate":"B1","terminal":"2"}}]}"#;
                    format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
                        body.len()
                    )
                };

                stream.write_all(response.as_bytes()).await.unwrap();
            }
        });

        let client = AviationStackClient::with_base_url(
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
}

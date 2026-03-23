//! `TripIt` API HTTP client and error types.

use crate::integrations::tripit::auth::{AuthError, TripItAuth};
use serde_json::Value;
use thiserror::Error;

const TRIPIT_API_BASE: &str = "https://api.tripit.com/v1";

/// Trait for making `TripIt` API calls — abstracted for testing.
#[async_trait::async_trait]
pub trait TripItApi: Send + Sync {
    async fn list_trips(&self, past: bool, page: u64, page_size: u64) -> Result<Value, FetchError>;
    async fn get_trip_objects(&self, trip_id: &str) -> Result<Value, FetchError>;
}

/// HTTP client for the `TripIt` v1 REST API.
pub struct TripItClient {
    auth: TripItAuth,
    client: reqwest::Client,
    base_url: String,
}

impl TripItClient {
    /// Create a new client using the default `TripIt` API base URL.
    #[must_use]
    pub fn new(auth: TripItAuth) -> Self {
        let client = reqwest::Client::new();
        Self::with_base_url(auth, client, TRIPIT_API_BASE.to_string())
    }

    /// Create a new client with a custom base URL — primarily for testing.
    #[must_use]
    pub fn with_base_url(auth: TripItAuth, client: reqwest::Client, base_url: String) -> Self {
        Self {
            auth,
            client,
            base_url,
        }
    }

    pub(super) async fn get(&self, path: &str) -> Result<Value, FetchError> {
        let url = format!("{}/{path}/format/json", self.base_url);
        tracing::debug!(url, "GET request");

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

/// Errors that can occur when fetching data from the `TripIt` API.
#[derive(Debug, Error)]
pub enum FetchError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("failed to parse API response: {0}")]
    Json(#[from] serde_json::Error),

    #[error("OAuth signing failed: {0}")]
    Auth(#[from] AuthError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integrations::tripit::auth::TripItAuth;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU32, Ordering};
    use tokio::io::AsyncWriteExt;

    #[tokio::test]
    async fn get_retries_on_server_error() {
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

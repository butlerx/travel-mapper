//! Flight status API trait and shared types.

use serde_json;
use thiserror::Error;

/// Errors returned by flight status API providers.
#[derive(Debug, Error)]
pub enum FlightStatusError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("failed to parse API response: {0}")]
    Json(#[from] serde_json::Error),
}

/// Live or historical status data for a single flight.
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

/// Trait for querying real-time or historical flight status from external providers.
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

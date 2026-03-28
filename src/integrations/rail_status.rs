//! Rail status API trait and shared types.

use thiserror::Error;

/// Errors from rail status API providers.
#[derive(Debug, Error)]
pub enum RailStatusError {
    /// HTTP transport or connection failure.
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    /// Response body could not be parsed into the expected format.
    #[error("failed to parse API response: {0}")]
    Parse(String),

    /// Provider-specific rate limit exceeded — caller should stop retrying.
    #[error("rate limit exceeded")]
    RateLimited,
}

/// Input query for rail status lookup — providers use whichever fields they need.
#[derive(Debug, Clone)]
#[allow(dead_code)] // providers use different field subsets
pub struct RailStatusQuery<'a> {
    pub carrier: &'a str,
    pub train_number: &'a str,
    pub origin_name: &'a str,
    pub dest_name: &'a str,
    pub origin_country: Option<&'a str>,
    pub dest_country: Option<&'a str>,
    pub start_date: &'a str,
    pub end_date: &'a str,
    pub origin_lat: f64,
    pub origin_lng: f64,
    pub dest_lat: f64,
    pub dest_lng: f64,
}

/// Rail status result from a provider.
#[derive(Debug, Clone)]
pub struct RailStatus {
    /// High-level status: `"on_time"`, `"delayed"`, `"cancelled"`, `"departed"`, `"arrived"`.
    pub status: String,
    /// Departure delay in minutes (positive = late, negative = early).
    pub dep_delay_minutes: Option<i64>,
    /// Arrival delay in minutes (positive = late, negative = early).
    pub arr_delay_minutes: Option<i64>,
    /// Scheduled or actual departure platform.
    pub dep_platform: String,
    /// Scheduled or actual arrival platform.
    pub arr_platform: String,
    /// Full provider response serialised as JSON for debugging.
    pub raw_json: String,
}

/// Provider-agnostic interface for rail status lookups.
#[async_trait::async_trait]
pub trait RailStatusApi: Send + Sync {
    /// Provider name written to the `status_enrichments.provider` column.
    #[allow(dead_code)]
    fn provider_name(&self) -> &'static str;

    /// Look up rail status for a journey.
    ///
    /// Returns `Ok(None)` when the provider has no data for this journey (not an
    /// error — just no match).
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails, the response cannot be parsed,
    /// or the provider rate limit is exceeded.
    async fn get_rail_status(
        &self,
        query: &RailStatusQuery<'_>,
    ) -> Result<Option<RailStatus>, RailStatusError>;
}

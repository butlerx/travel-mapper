//! HTTP client for Transitland REST API v2.
//!
//! Base URL: `https://transit.land/api/v2/rest/`
//! Authentication: API key via query parameter or header.
//! Rate limits: 10,000 queries/month (free), 200,000/month (professional $200/month).

use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Transitland API v2 client with authentication.
#[derive(Clone)]
pub struct TransitlandClient {
    client: Client,
    api_key: String,
    base_url: String,
}

impl TransitlandClient {
    /// Creates a new Transitland API client.
    ///
    /// # Arguments
    ///
    /// * `api_key` - Transitland API key for authentication
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client cannot be initialized.
    pub fn new(api_key: String) -> Result<Self, reqwest::Error> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("travel-mapper/0.1.0")
            .build()?;

        Ok(Self {
            client,
            api_key,
            base_url: "https://transit.land/api/v2/rest".to_string(),
        })
    }

    /// Fetches feeds matching the given query parameters.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails or the response cannot be parsed.
    pub async fn search_feeds(
        &self,
        params: &FeedSearchParams,
    ) -> Result<FeedSearchResponse, reqwest::Error> {
        let url = format!("{}/feeds", self.base_url);
        let mut query_params = vec![("apikey", self.api_key.as_str())];

        if let Some(ref spec) = params.spec {
            query_params.push(("spec", spec.as_str()));
        }
        if let Some(ref search) = params.search {
            query_params.push(("search", search.as_str()));
        }
        if let Some(ref onestop_id) = params.onestop_id {
            query_params.push(("onestop_id", onestop_id.as_str()));
        }

        self.client
            .get(&url)
            .query(&query_params)
            .send()
            .await?
            .json()
            .await
    }

    /// Downloads the latest GTFS-RT protobuf data for a specific feed.
    ///
    /// # Arguments
    ///
    /// * `feed_id` - Transitland feed onestop ID (e.g., `f-u0-sncf~transilien~rer`)
    /// * `rt_type` - Type of realtime data: `trip_updates`, `vehicle_positions`, or `alerts`
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails.
    pub async fn download_rt_feed(
        &self,
        feed_id: &str,
        rt_type: RtFeedType,
    ) -> Result<Response, reqwest::Error> {
        let url = format!(
            "{}/feeds/{}/download_latest_rt/{}.pb",
            self.base_url,
            feed_id,
            rt_type.as_str()
        );

        self.client
            .get(&url)
            .query(&[("apikey", self.api_key.as_str())])
            .send()
            .await
    }

    /// Downloads and returns GTFS-RT protobuf bytes for trip updates.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails or response cannot be read.
    pub async fn fetch_trip_updates(&self, feed_id: &str) -> Result<Vec<u8>, reqwest::Error> {
        let response = self
            .download_rt_feed(feed_id, RtFeedType::TripUpdates)
            .await?;
        response.bytes().await.map(|b| b.to_vec())
    }
}

/// GTFS-RT feed type for download endpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RtFeedType {
    TripUpdates,
    VehiclePositions,
    Alerts,
}

impl RtFeedType {
    const fn as_str(self) -> &'static str {
        match self {
            Self::TripUpdates => "trip_updates",
            Self::VehiclePositions => "vehicle_positions",
            Self::Alerts => "alerts",
        }
    }
}

/// Parameters for feed search query.
#[derive(Debug, Default, Clone, Serialize)]
pub struct FeedSearchParams {
    /// Filter by feed specification type (e.g., `gtfs`, `gtfs-rt`).
    pub spec: Option<String>,
    /// Search query for operator name or feed metadata.
    pub search: Option<String>,
    /// Exact onestop ID to retrieve a specific feed.
    pub onestop_id: Option<String>,
}

/// Response from feed search endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct FeedSearchResponse {
    pub feeds: Vec<Feed>,
    pub meta: Option<ResponseMeta>,
}

/// Individual feed metadata from Transitland.
#[derive(Debug, Clone, Deserialize)]
pub struct Feed {
    pub id: i64,
    pub onestop_id: String,
    pub name: Option<String>,
    pub spec: String,
    pub urls: FeedUrls,
}

/// Feed URLs for GTFS static and realtime data.
#[derive(Debug, Clone, Deserialize)]
pub struct FeedUrls {
    pub static_current: Option<String>,
    pub realtime_trip_updates: Option<String>,
    pub realtime_vehicle_positions: Option<String>,
    pub realtime_alerts: Option<String>,
}

/// Pagination metadata from API responses.
#[derive(Debug, Clone, Deserialize)]
pub struct ResponseMeta {
    pub next: Option<String>,
    pub prev: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rt_feed_type_as_str() {
        assert_eq!(RtFeedType::TripUpdates.as_str(), "trip_updates");
        assert_eq!(RtFeedType::VehiclePositions.as_str(), "vehicle_positions");
        assert_eq!(RtFeedType::Alerts.as_str(), "alerts");
    }

    #[tokio::test]
    async fn new_client_succeeds() {
        let result = TransitlandClient::new("test-key".to_string());
        assert!(result.is_ok());
    }
}

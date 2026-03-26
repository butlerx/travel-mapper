//! Application state, router setup, and request handling.

pub(crate) mod components;
pub(crate) mod error;
/// Axum extractors for request authentication and authorization.
pub(crate) mod extractors;
/// Tower middleware for HTTP request tracing and diagnostics.
pub(crate) mod middleware;
pub(crate) mod pages;
pub(crate) mod routes;
pub(crate) mod session;
mod state;

pub use state::{AppState, create_router};

/// User-facing application name shown in the navbar, HTML titles, and PWA manifest.
pub(crate) const APP_NAME: &str = "Travel Mapper";

/// Abbreviated name for PWA home-screen icons and space-constrained UI.
pub(crate) const APP_SHORT_NAME: &str = env!("CARGO_PKG_NAME");

/// One-line tagline used in the PWA manifest description.
pub(crate) const APP_DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

/// Primary theme colour applied to the PWA chrome and `<meta name="theme-color">`.
pub(crate) const THEME_COLOR: &str = "#0a0e1a";

#[cfg(test)]
pub(crate) mod test_helpers {
    use crate::{
        db,
        integrations::tripit::{FetchError, TripItApi},
        server::AppState,
    };
    use axum::{body::to_bytes, response::Response};
    use serde_json::{Value, json};
    use sha2::{Digest, Sha256};
    use sqlx::SqlitePool;
    use std::{fmt::Write, sync::Arc};
    use uuid::Uuid;

    pub struct MockTripItApiWithData;

    #[async_trait::async_trait]
    impl TripItApi for MockTripItApiWithData {
        async fn list_trips(
            &self,
            past: bool,
            _page: u64,
            _page_size: u64,
        ) -> Result<Value, FetchError> {
            if past {
                Ok(json!({
                    "Trip": [
                        {"id": "100", "display_name": "Paris Trip"},
                        {"id": "200", "display_name": "London Trip"}
                    ],
                    "max_page": "1"
                }))
            } else {
                Ok(json!({"Trip": [], "max_page": "1"}))
            }
        }

        async fn get_trip_objects(&self, trip_id: &str) -> Result<Value, FetchError> {
            match trip_id {
                "100" => Ok(json!({
                    "AirObject": [{
                        "Segment": [{
                            "start_airport_code": "CDG",
                            "start_airport_latitude": "49.0097",
                            "start_airport_longitude": "2.5479",
                            "end_airport_code": "LHR",
                            "end_airport_latitude": "51.4700",
                            "end_airport_longitude": "-0.4543",
                            "StartDateTime": {"date": "2024-03-01"},
                            "EndDateTime": {"date": "2024-03-01"}
                        }]
                    }]
                })),
                "200" => Ok(json!({
                    "RailObject": [{
                        "Segment": [{
                            "start_station_name": "Kings Cross",
                            "end_station_name": "Edinburgh Waverley",
                            "StartStationAddress": {"latitude": "51.5320", "longitude": "-0.1240"},
                            "EndStationAddress": {"latitude": "55.9519", "longitude": "-3.1890"},
                            "StartDateTime": {"date": "2024-04-15"},
                            "EndDateTime": {"date": "2024-04-15"}
                        }]
                    }]
                })),
                _ => Ok(json!({})),
            }
        }
    }

    pub async fn test_pool() -> SqlitePool {
        let db_name = Uuid::new_v4();
        let url = format!("sqlite:file:{db_name}?mode=memory&cache=shared");
        db::create_pool(&url)
            .await
            .expect("failed to create test pool")
    }

    pub fn test_app_state(pool: SqlitePool) -> AppState {
        AppState {
            leptos_options: leptos::prelude::LeptosOptions::builder()
                .output_name("travel-mapper")
                .build(),
            db: pool,
            encryption_key: [7; 32],
            tripit_consumer_key: "consumer-key".to_string(),
            tripit_consumer_secret: "consumer-secret".to_string(),
            tripit_override: None,
            registration_enabled: true,
            aviationstack_api_key: None,
        }
    }

    pub async fn auth_cookie_for_user(pool: &SqlitePool, username: &str) -> String {
        let user_id = db::users::Create {
            username,
            password_hash: "hash",
        }
        .execute(pool)
        .await
        .expect("failed to create user");
        db::sessions::Create {
            token: &format!("session-{username}"),
            user_id,
            expires_at: "2999-01-01 00:00:00",
        }
        .execute(pool)
        .await
        .expect("failed to create session");
        format!("session_id=session-{username}")
    }

    pub async fn api_key_for_user(pool: &SqlitePool, username: &str, key: &str) {
        let user = db::users::GetByUsername { username }
            .execute(pool)
            .await
            .expect("user lookup failed")
            .expect("user missing");
        let hash = Sha256::digest(key.as_bytes());
        let hex = hash.iter().fold(String::new(), |mut output, byte| {
            let _ = write!(output, "{byte:02x}");
            output
        });
        db::api_keys::Create {
            user_id: user.id,
            key_hash: &hex,
            label: "test",
        }
        .execute(pool)
        .await
        .expect("failed to create api key");
    }

    pub fn sample_hop(
        travel_type: db::hops::TravelType,
        origin: &str,
        dest: &str,
        start: &str,
        end: &str,
    ) -> db::hops::Row {
        db::hops::Row {
            id: 0,
            travel_type,
            origin_name: origin.to_string(),
            origin_lat: 1.0,
            origin_lng: 2.0,
            origin_country: None,
            dest_name: dest.to_string(),
            dest_lat: 3.0,
            dest_lng: 4.0,
            dest_country: None,
            start_date: start.to_string(),
            end_date: end.to_string(),
            raw_json: None,
            origin_address_query: None,
            dest_address_query: None,
            origin_tz: None,
            dest_tz: None,
            flight_detail: None,
            rail_detail: None,
            boat_detail: None,
            transport_detail: None,
            cached_carrier: None,
            cost_amount: None,
            cost_currency: None,
        }
    }

    pub async fn body_text(response: Response) -> String {
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("failed to read response body");
        String::from_utf8(body.to_vec()).expect("response body should be UTF-8")
    }

    pub fn mock_tripit_api_with_data() -> Arc<dyn TripItApi> {
        Arc::new(MockTripItApiWithData)
    }
}

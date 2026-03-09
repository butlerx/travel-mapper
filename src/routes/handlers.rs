use super::response::{ResponseFormat, build_csv_response, build_html_response, negotiate_format};
use crate::{
    auth::AuthUser,
    db,
    models::TravelHop,
    server::{AppState, SyncResult, sync_all},
};
use aide::{axum::IntoApiResponse, transform::TransformOperation};
use axum::{
    Json,
    extract::{Query, State},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Redirect, Response},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;

/// Error response returned by most API endpoints on failure.
#[derive(Debug, Serialize, JsonSchema)]
pub struct ErrorResponse {
    /// Human-readable error message.
    pub error: String,
}

/// Health check response.
#[derive(Debug, Serialize, JsonSchema)]
pub struct HealthResponse {
    /// Server status, always `"ok"`.
    pub status: String,
    /// Timestamp of the most recent sync, or null if never synced.
    pub last_sync: Option<String>,
}

/// Response when a sync job is enqueued.
#[derive(Debug, Serialize, JsonSchema)]
pub struct SyncQueuedResponse {
    /// Status message.
    pub status: String,
    /// ID of the enqueued sync job.
    pub job_id: i64,
}

#[derive(Deserialize, JsonSchema)]
pub struct HopQuery {
    /// Filter hops by travel type (air, rail, cruise, transport).
    #[serde(rename = "type")]
    travel_type: Option<String>,
}

fn is_form_request(headers: &HeaderMap) -> bool {
    headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|ct| ct.starts_with("application/x-www-form-urlencoded"))
}

/// Trigger a `TripIt` sync for the authenticated user.
///
/// Enqueues a background sync job. If a `tripit_override` is configured (testing),
/// runs the sync inline instead.
pub async fn sync_handler(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
) -> Response {
    let is_form = is_form_request(&headers);

    if let Some(override_api) = &state.tripit_override {
        let result = sync_all(override_api.as_ref(), &state.db, auth.user_id).await;
        return match result {
            Ok(r) => (StatusCode::OK, Json(json!(r))).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("sync failed: {err}") })),
            )
                .into_response(),
        };
    }

    match db::has_pending_or_running_sync_job(&state.db, auth.user_id).await {
        Ok(true) => {
            return if is_form {
                Redirect::to("/dashboard?error=Sync+already+queued").into_response()
            } else {
                (
                    StatusCode::CONFLICT,
                    Json(json!({ "error": "sync already queued or running" })),
                )
                    .into_response()
            };
        }
        Ok(false) => {}
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("database error: {err}") })),
            )
                .into_response();
        }
    }

    match db::enqueue_sync_job(&state.db, auth.user_id).await {
        Ok(job_id) => {
            tracing::info!(user_id = auth.user_id, job_id, "sync job enqueued");
            if is_form {
                Redirect::to("/dashboard").into_response()
            } else {
                (
                    StatusCode::ACCEPTED,
                    Json(json!({ "status": "sync queued", "job_id": job_id })),
                )
                    .into_response()
            }
        }
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("failed to enqueue sync: {err}") })),
        )
            .into_response(),
    }
}

/// `OpenAPI` docs for [`sync_handler`].
pub fn sync_handler_docs(op: TransformOperation) -> TransformOperation {
    op.description("Trigger a TripIt sync for the authenticated user.")
        .response::<200, Json<SyncResult>>()
        .response::<202, Json<SyncQueuedResponse>>()
        .response::<409, Json<ErrorResponse>>()
        .response::<500, Json<ErrorResponse>>()
        .tag("sync")
}

/// Check server health and last sync timestamp.
pub async fn health_handler(State(state): State<AppState>) -> impl IntoApiResponse {
    let last_sync =
        sqlx::query_scalar::<_, Option<String>>("SELECT MAX(last_sync_at) FROM sync_state")
            .fetch_one(&state.db)
            .await
            .ok()
            .flatten();

    let body = json!({
        "status": "ok",
        "last_sync": last_sync,
    });

    (StatusCode::OK, Json(body)).into_response()
}

pub fn health_handler_docs(op: TransformOperation) -> TransformOperation {
    op.description("Check server health and last sync timestamp.")
        .response::<200, Json<HealthResponse>>()
        .tag("health")
}

/// List travel hops for the authenticated user.
///
/// Returns hops as JSON, CSV, or an HTML table based on the `Accept` header.
pub async fn hops_handler(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(query): Query<HopQuery>,
    headers: HeaderMap,
) -> Response {
    let hops = match db::get_all_hops(&state.db, auth.user_id, query.travel_type.as_deref()).await {
        Ok(hops) => hops,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("failed to fetch hops: {err}") })),
            )
                .into_response();
        }
    };

    match negotiate_format(&headers) {
        ResponseFormat::Json => (StatusCode::OK, Json(json!(hops))).into_response(),
        ResponseFormat::Csv => build_csv_response(&hops),
        ResponseFormat::Html => build_html_response(&hops),
    }
}

pub fn hops_handler_docs(op: TransformOperation) -> TransformOperation {
    op.description("List travel hops for the authenticated user.")
        .response::<200, Json<Vec<TravelHop>>>()
        .response::<500, Json<ErrorResponse>>()
        .tag("hops")
}

#[cfg(test)]
mod tests {
    use crate::{
        auth::encrypt_token,
        db,
        models::{TravelHop, TravelType},
        server::{AppState, create_router},
        tripit::{FetchError, TripItApi},
    };
    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode, header},
    };
    use serde_json::{Value, json};
    use sha2::{Digest, Sha256};
    use sqlx::SqlitePool;
    use std::{fmt::Write, sync::Arc};
    use tower::ServiceExt;
    use uuid::Uuid;

    struct MockTripItApiWithData;

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

    async fn test_pool() -> SqlitePool {
        let db_name = Uuid::new_v4();
        let url = format!("sqlite:file:{db_name}?mode=memory&cache=shared");
        db::create_pool(&url)
            .await
            .expect("failed to create test pool")
    }

    fn test_app_state(pool: SqlitePool) -> AppState {
        AppState {
            leptos_options: leptos::prelude::LeptosOptions::builder()
                .output_name("travel-mapper")
                .build(),
            db: pool,
            encryption_key: [7; 32],
            tripit_consumer_key: "consumer-key".to_string(),
            tripit_consumer_secret: "consumer-secret".to_string(),
            tripit_override: None,
        }
    }

    async fn auth_cookie_for_user(pool: &SqlitePool, username: &str) -> String {
        let user_id = db::create_user(pool, username, "hash")
            .await
            .expect("failed to create user");
        db::create_session(
            pool,
            &format!("session-{username}"),
            user_id,
            "2999-01-01 00:00:00",
        )
        .await
        .expect("failed to create session");
        format!("session_id=session-{username}")
    }

    async fn api_key_for_user(pool: &SqlitePool, username: &str, key: &str) {
        let user = db::get_user_by_username(pool, username)
            .await
            .expect("user lookup failed")
            .expect("user missing");
        let hash = Sha256::digest(key.as_bytes());
        let hex = hash.iter().fold(String::new(), |mut output, byte| {
            let _ = write!(output, "{byte:02x}");
            output
        });
        db::create_api_key(pool, user.id, &hex, "test")
            .await
            .expect("failed to create api key");
    }

    fn sample_hop(
        travel_type: TravelType,
        origin: &str,
        dest: &str,
        start: &str,
        end: &str,
    ) -> TravelHop {
        TravelHop {
            travel_type,
            origin_name: origin.to_string(),
            origin_lat: Some(1.0),
            origin_lng: Some(2.0),
            dest_name: dest.to_string(),
            dest_lat: Some(3.0),
            dest_lng: Some(4.0),
            start_date: start.to_string(),
            end_date: end.to_string(),
        }
    }

    async fn body_text(response: axum::response::Response) -> String {
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("failed to read response body");
        String::from_utf8(body.to_vec()).expect("response body should be UTF-8")
    }

    #[tokio::test]
    async fn register_new_user_returns_created() {
        let pool = test_pool().await;
        let app = create_router(test_app_state(pool));
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/auth/register")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(r#"{"username":"alice","password":"secret"}"#))
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    #[tokio::test]
    async fn register_duplicate_username_returns_conflict() {
        let pool = test_pool().await;
        let app = create_router(test_app_state(pool));

        let request = || {
            Request::builder()
                .method("POST")
                .uri("/auth/register")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"username":"alice","password":"secret"}"#))
                .expect("failed to build request")
        };

        let first = app
            .clone()
            .oneshot(request())
            .await
            .expect("first request failed");
        assert_eq!(first.status(), StatusCode::CREATED);

        let second = app.oneshot(request()).await.expect("second request failed");
        assert_eq!(second.status(), StatusCode::CONFLICT);
    }

    #[tokio::test]
    async fn login_correct_password_returns_cookie() {
        let pool = test_pool().await;
        let app = create_router(test_app_state(pool.clone()));

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/auth/register")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(r#"{"username":"alice","password":"secret"}"#))
                    .expect("failed to build request"),
            )
            .await
            .expect("register request failed");

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/auth/login")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(r#"{"username":"alice","password":"secret"}"#))
                    .expect("failed to build request"),
            )
            .await
            .expect("login request failed");

        assert_eq!(response.status(), StatusCode::OK);
        let set_cookie = response
            .headers()
            .get(header::SET_COOKIE)
            .and_then(|value| value.to_str().ok())
            .expect("set-cookie header should exist");
        assert!(set_cookie.contains("session_id="));
    }

    #[tokio::test]
    async fn login_wrong_password_returns_unauthorized() {
        let pool = test_pool().await;
        let app = create_router(test_app_state(pool));

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/auth/register")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(r#"{"username":"alice","password":"secret"}"#))
                    .expect("failed to build request"),
            )
            .await
            .expect("register request failed");

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/auth/login")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(r#"{"username":"alice","password":"wrong"}"#))
                    .expect("failed to build request"),
            )
            .await
            .expect("login request failed");

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn access_hops_without_auth_returns_unauthorized() {
        let pool = test_pool().await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/hops")
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn access_hops_with_session_cookie_returns_ok() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let user = db::get_user_by_username(&pool, "alice")
            .await
            .expect("lookup failed")
            .expect("missing user");
        db::insert_hops(
            &pool,
            "trip-1",
            user.id,
            &[sample_hop(
                TravelType::Air,
                "LHR",
                "JFK",
                "2024-01-01",
                "2024-01-01",
            )],
        )
        .await
        .expect("insert failed");

        let app = create_router(test_app_state(pool));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/hops")
                    .header(header::COOKIE, cookie)
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn create_api_key_returns_key() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/auth/api-keys")
                    .header(header::CONTENT_TYPE, "application/json")
                    .header(header::COOKIE, cookie)
                    .body(Body::from(r#"{"label":"test"}"#))
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("failed to read response body");
        let parsed: Value = serde_json::from_slice(&body).expect("json body");
        assert!(parsed["key"].as_str().is_some());
    }

    #[tokio::test]
    async fn access_hops_with_api_key_returns_ok() {
        let pool = test_pool().await;
        let _ = auth_cookie_for_user(&pool, "alice").await;
        let user = db::get_user_by_username(&pool, "alice")
            .await
            .expect("lookup failed")
            .expect("missing user");
        db::insert_hops(
            &pool,
            "trip-1",
            user.id,
            &[sample_hop(
                TravelType::Rail,
                "Paris",
                "London",
                "2024-01-01",
                "2024-01-01",
            )],
        )
        .await
        .expect("insert failed");
        api_key_for_user(&pool, "alice", "my-api-key").await;

        let app = create_router(test_app_state(pool));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/hops")
                    .header(header::AUTHORIZATION, "Bearer my-api-key")
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn get_health_returns_ok_with_null_last_sync() {
        let pool = test_pool().await;
        let state = test_app_state(pool);

        let app = create_router(state);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("failed to read response body");
        let body_text = String::from_utf8(body.to_vec()).expect("response body should be UTF-8");

        assert!(body_text.contains("\"status\":\"ok\""));
        assert!(body_text.contains("\"last_sync\":null"));
    }

    #[tokio::test]
    async fn get_hops_json_returns_inserted_hops() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let user = db::get_user_by_username(&pool, "alice")
            .await
            .expect("lookup failed")
            .expect("missing user");
        let hops = vec![
            sample_hop(
                TravelType::Rail,
                "Paris",
                "London",
                "2024-01-01",
                "2024-01-01",
            ),
            sample_hop(TravelType::Air, "LHR", "JFK", "2024-02-01", "2024-02-01"),
        ];
        db::insert_hops(&pool, "trip-1", user.id, &hops)
            .await
            .expect("insert failed");

        let app = create_router(test_app_state(pool));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/hops")
                    .header(header::COOKIE, cookie)
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("failed to read response body");
        let parsed: Vec<TravelHop> =
            serde_json::from_slice(&body).expect("body should be valid JSON array");

        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].travel_type, TravelType::Rail);
    }

    #[tokio::test]
    async fn get_hops_json_filters_by_type() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let user = db::get_user_by_username(&pool, "alice")
            .await
            .expect("lookup failed")
            .expect("missing user");
        let hops = vec![
            sample_hop(TravelType::Air, "LHR", "JFK", "2024-02-01", "2024-02-01"),
            sample_hop(
                TravelType::Rail,
                "Paris",
                "London",
                "2024-01-01",
                "2024-01-01",
            ),
        ];
        db::insert_hops(&pool, "trip-1", user.id, &hops)
            .await
            .expect("insert failed");

        let app = create_router(test_app_state(pool));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/hops?type=rail")
                    .header(header::COOKIE, cookie)
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("failed to read response body");
        let parsed: Vec<TravelHop> = serde_json::from_slice(&body).expect("valid json");
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].travel_type, TravelType::Rail);
    }

    #[tokio::test]
    async fn post_sync_fetches_and_stores_hops() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let mut state = test_app_state(pool.clone());
        state.tripit_override = Some(Arc::new(MockTripItApiWithData));

        let app = create_router(state);
        let sync_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/sync")
                    .header(header::COOKIE, cookie.clone())
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");
        assert_eq!(sync_response.status(), StatusCode::OK);

        let sync_body = to_bytes(sync_response.into_body(), usize::MAX)
            .await
            .expect("failed to read response body");
        let sync_json: Value =
            serde_json::from_slice(&sync_body).expect("body should be valid JSON");
        assert_eq!(sync_json["trips_fetched"], json!(2));

        let hops_response = app
            .oneshot(
                Request::builder()
                    .uri("/hops")
                    .header(header::COOKIE, cookie)
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");
        assert_eq!(hops_response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn post_sync_updates_sync_state() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let user = db::get_user_by_username(&pool, "alice")
            .await
            .expect("lookup failed")
            .expect("missing user");

        let mut state = test_app_state(pool.clone());
        state.tripit_override = Some(Arc::new(MockTripItApiWithData));

        let app = create_router(state);
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/sync")
                    .header(header::COOKIE, cookie)
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);

        let sync_state = db::get_or_create_sync_state(&pool, user.id)
            .await
            .expect("failed to fetch sync state");
        assert!(sync_state.last_sync_at.is_some());
        assert_eq!(sync_state.sync_status, "idle");
    }

    #[tokio::test]
    async fn get_hops_with_accept_csv_returns_csv() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let user = db::get_user_by_username(&pool, "alice")
            .await
            .expect("lookup failed")
            .expect("missing user");
        db::insert_hops(
            &pool,
            "trip-1",
            user.id,
            &[sample_hop(
                TravelType::Rail,
                "Paris",
                "London",
                "2024-01-01",
                "2024-01-01",
            )],
        )
        .await
        .expect("insert failed");

        let app = create_router(test_app_state(pool));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/hops")
                    .header(header::COOKIE, cookie)
                    .header(header::ACCEPT, "text/csv")
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn get_hops_with_accept_html_returns_html_table() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let user = db::get_user_by_username(&pool, "alice")
            .await
            .expect("lookup failed")
            .expect("missing user");
        db::insert_hops(
            &pool,
            "trip-1",
            user.id,
            &[sample_hop(
                TravelType::Air,
                "LHR",
                "JFK",
                "2024-02-01",
                "2024-02-01",
            )],
        )
        .await
        .expect("insert failed");

        let app = create_router(test_app_state(pool));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/hops")
                    .header(header::COOKIE, cookie)
                    .header(header::ACCEPT, "text/html")
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn landing_page_contains_hero_content() {
        let pool = test_pool().await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/")
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_text(response).await;
        assert!(body.contains("Travel Export"));
        assert!(body.contains("Get Started"));
        assert!(body.contains("href=\"/register\""));
        assert!(body.contains("href=\"/login\""));
    }

    #[tokio::test]
    async fn register_page_renders_form() {
        let pool = test_pool().await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/register")
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_text(response).await;
        assert!(body.contains("Register"));
        assert!(body.contains("action=\"/auth/register\""));
        assert!(body.contains("method=\"post\""));
        assert!(body.contains("id=\"username\""));
        assert!(body.contains("name=\"username\""));
        assert!(body.contains("id=\"password\""));
        assert!(body.contains("name=\"password\""));
        assert!(body.contains("Create Account"));
        assert!(body.contains("href=\"/login\""));
    }

    #[tokio::test]
    async fn register_page_with_error_renders_alert() {
        let pool = test_pool().await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/register?error=Username+taken")
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_text(response).await;
        assert!(body.contains("Username taken"));
        assert!(body.contains("alert-error"));
    }

    #[tokio::test]
    async fn login_page_renders_form() {
        let pool = test_pool().await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/login")
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_text(response).await;
        assert!(body.contains("Log In"));
        assert!(body.contains("action=\"/auth/login\""));
        assert!(body.contains("method=\"post\""));
        assert!(body.contains("id=\"username\""));
        assert!(body.contains("name=\"username\""));
        assert!(body.contains("id=\"password\""));
        assert!(body.contains("name=\"password\""));
        assert!(body.contains("href=\"/register\""));
    }

    #[tokio::test]
    async fn login_page_with_error_renders_alert() {
        let pool = test_pool().await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/login?error=Invalid+credentials")
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_text(response).await;
        assert!(body.contains("Invalid credentials"));
        assert!(body.contains("alert-error"));
    }

    #[tokio::test]
    async fn dashboard_without_hops_renders_empty_state_and_nav() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/dashboard")
                    .header(header::COOKIE, cookie)
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_text(response).await;
        assert!(body.contains("No hops yet"));
        assert!(body.contains("href=\"/settings\""));
        assert!(body.contains("<nav"));
        assert!(body.contains("Dashboard"));
        assert!(body.contains("Settings"));
        assert!(body.contains("action=\"/auth/logout\""));
    }

    #[tokio::test]
    async fn dashboard_with_hops_renders_map_controls_and_script() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let user = db::get_user_by_username(&pool, "alice")
            .await
            .expect("lookup failed")
            .expect("missing user");
        db::insert_hops(
            &pool,
            "trip-1",
            user.id,
            &[sample_hop(
                TravelType::Air,
                "LHR",
                "JFK",
                "2024-02-01",
                "2024-02-01",
            )],
        )
        .await
        .expect("insert failed");

        let app = create_router(test_app_state(pool));
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/dashboard")
                    .header(header::COOKIE, cookie)
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_text(response).await;
        assert!(body.contains("id=\"map\""));
        assert!(body.contains("id=\"filter-type\""));
        assert!(body.contains("id=\"filter-year\""));
        assert!(body.contains("map-legend"));
        assert!(body.contains("window.allHops="));
        assert!(body.contains("/static/map.js"));
    }

    #[tokio::test]
    async fn dashboard_with_error_renders_alert() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/dashboard?error=Sync+failed")
                    .header(header::COOKIE, cookie)
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_text(response).await;
        assert!(body.contains("Sync failed"));
        assert!(body.contains("alert-error"));
    }

    #[tokio::test]
    async fn settings_page_without_tripit_renders_sections() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/settings")
                    .header(header::COOKIE, cookie)
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_text(response).await;
        assert!(body.contains("Settings"));
        assert!(body.contains("TripIt Connection"));
        assert!(body.contains("Not Connected"));
        assert!(body.contains("Connect TripIt"));
        assert!(body.contains("/auth/tripit/connect"));
        assert!(body.contains("Sync Status"));
        assert!(body.contains("API Keys"));
    }

    #[tokio::test]
    async fn settings_page_with_tripit_and_sync_state_renders_connected_and_sync_now() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let user = db::get_user_by_username(&pool, "alice")
            .await
            .expect("lookup failed")
            .expect("missing user");
        let key = [7_u8; 32];
        let (access_token_enc, nonce_token) =
            encrypt_token("token", &key).expect("failed to encrypt access token");
        let (access_token_secret_enc, nonce_secret) =
            encrypt_token("secret", &key).expect("failed to encrypt access token secret");

        db::upsert_tripit_credentials(
            &pool,
            user.id,
            &access_token_enc,
            &access_token_secret_enc,
            &nonce_token,
            &nonce_secret,
        )
        .await
        .expect("failed to upsert tripit credentials");

        let mut sync_state = db::get_or_create_sync_state(&pool, user.id)
            .await
            .expect("failed to fetch sync state");
        sync_state.sync_status = "idle".to_string();
        sync_state.last_sync_at = Some("2026-01-02 03:04:05".to_string());
        sync_state.trips_fetched = 3;
        sync_state.hops_fetched = 12;
        db::update_sync_state(&pool, user.id, &sync_state)
            .await
            .expect("failed to update sync state");

        let app = create_router(test_app_state(pool));
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/settings")
                    .header(header::COOKIE, cookie)
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_text(response).await;
        assert!(body.contains("Connected"));
        assert!(body.contains("Sync Now"));
    }

    #[tokio::test]
    async fn settings_page_with_error_renders_alert() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/settings?error=Something+broke")
                    .header(header::COOKIE, cookie)
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_text(response).await;
        assert!(body.contains("Something broke"));
        assert!(body.contains("alert-error"));
    }

    #[tokio::test]
    async fn settings_page_tripit_connected_feedback_renders_success_alert() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/settings?tripit=connected")
                    .header(header::COOKIE, cookie)
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_text(response).await;
        assert!(body.contains("TripIt account connected successfully"));
    }

    #[tokio::test]
    async fn not_found_page_renders_expected_content() {
        let pool = test_pool().await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/nonexistent-route")
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let body = body_text(response).await;
        assert!(body.contains("404"));
        assert!(body.contains("Page Not Found"));
        assert!(body.contains("href=\"/\""));
    }

    #[tokio::test]
    async fn dashboard_without_auth_and_html_accept_renders_401_page() {
        let pool = test_pool().await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/dashboard")
                    .header(header::ACCEPT, "text/html")
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        let body = body_text(response).await;
        assert!(body.contains("401"));
        assert!(body.contains("Unauthorized"));
        assert!(body.contains("href=\"/login\""));
    }

    #[tokio::test]
    async fn settings_navbar_marks_settings_link_as_current_page() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/settings")
                    .header(header::COOKIE, cookie)
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_text(response).await;
        assert!(body.contains("<nav"));
        assert!(body.contains("href=\"/settings\""));
        assert!(body.contains("aria-current=\"page\""));
    }

    #[tokio::test]
    async fn get_hops_with_accept_html_contains_table_headers_and_hop_data() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let user = db::get_user_by_username(&pool, "alice")
            .await
            .expect("lookup failed")
            .expect("missing user");
        db::insert_hops(
            &pool,
            "trip-2",
            user.id,
            &[sample_hop(
                TravelType::Rail,
                "Paris",
                "London",
                "2024-01-01",
                "2024-01-01",
            )],
        )
        .await
        .expect("insert failed");

        let app = create_router(test_app_state(pool));
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/hops")
                    .header(header::COOKIE, cookie)
                    .header(header::ACCEPT, "text/html")
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_text(response).await;
        assert!(body.contains("<table>"));
        assert!(body.contains("Travel Hops"));
        assert!(body.contains("Type"));
        assert!(body.contains("Origin"));
        assert!(body.contains("Destination"));
        assert!(body.contains("Paris"));
        assert!(body.contains("London"));
    }

    #[tokio::test]
    async fn static_css_route_serves_text_css_content_type() {
        let pool = test_pool().await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/static/style.css")
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);
        let content_type = response
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .expect("content-type should exist");
        assert!(content_type.contains("text/css"));
    }

    #[tokio::test]
    async fn static_js_route_serves_javascript_content_type() {
        let pool = test_pool().await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/static/map.js")
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);
        let content_type = response
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .expect("content-type should exist");
        assert!(content_type.contains("application/javascript"));
    }
}

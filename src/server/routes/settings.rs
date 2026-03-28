use super::{ErrorResponse, ResponseFormat, negotiate_format};
use crate::{
    db,
    server::{AppState, extractors::AuthUser, pages},
};
use aide::transform::TransformOperation;
use axum::{
    Json,
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Read-only account settings state returned by the JSON API.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct SettingsResponse {
    /// Whether the user has connected a `TripIt` account.
    pub tripit_connected: bool,
    /// Current sync state, if available.
    pub sync: Option<SyncStatus>,
    pub email: String,
    /// Whether the user's email address has been verified.
    pub email_verified: bool,
    pub first_name: String,
    pub last_name: String,
}

/// Snapshot of the user's `TripIt` sync progress.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct SyncStatus {
    /// Current sync status (e.g. `"idle"`, `"running"`).
    pub status: String,
    /// Timestamp of the last completed sync.
    pub last_sync_at: Option<String>,
    /// Number of `TripIt` trips fetched.
    pub trips_fetched: i64,
    /// Number of individual journey hops fetched.
    pub journeys_fetched: i64,
}

pub async fn handler(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(feedback): Query<pages::settings::SettingsFeedback>,
    headers: HeaderMap,
) -> Response {
    let format = negotiate_format(&headers);

    let has_tripit = db::credentials::Has {
        user_id: auth.user_id,
    }
    .execute(&state.db)
    .await
    .unwrap_or(false);

    let sync_state = db::sync_state::GetOrCreate {
        user_id: auth.user_id,
    }
    .execute(&state.db)
    .await
    .ok();

    let user = db::users::GetById { id: auth.user_id }
        .execute(&state.db)
        .await
        .ok()
        .flatten();

    let (email, email_verified, first_name, last_name) = user
        .map(|u| (u.email, u.email_verified, u.first_name, u.last_name))
        .unwrap_or_default();

    match format {
        ResponseFormat::Html => pages::settings::render_page(
            has_tripit,
            sync_state.as_ref(),
            feedback,
            pages::settings::UserProfileData {
                email,
                email_verified,
                first_name,
                last_name,
            },
            state.vapid_public_key.clone(),
        ),
        ResponseFormat::Json => {
            let response = SettingsResponse {
                tripit_connected: has_tripit,
                sync: sync_state.map(|s| SyncStatus {
                    status: s.sync_status,
                    last_sync_at: s.last_sync_at,
                    trips_fetched: s.trips_fetched,
                    journeys_fetched: s.hops_fetched,
                }),
                email,
                email_verified,
                first_name,
                last_name,
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        ResponseFormat::Csv => (
            StatusCode::NOT_ACCEPTABLE,
            Json(ErrorResponse {
                error: "CSV format is not supported for settings".to_owned(),
            }),
        )
            .into_response(),
    }
}

/// `OpenAPI` metadata for the settings endpoint.
pub fn handler_docs(op: TransformOperation) -> TransformOperation {
    op.description("Account settings state for the authenticated user.")
        .response::<200, Json<SettingsResponse>>()
        .response::<401, Json<ErrorResponse>>()
        .response::<406, Json<ErrorResponse>>()
        .response::<500, Json<ErrorResponse>>()
        .tag("settings")
}

#[cfg(test)]
mod tests {
    use crate::{
        auth::encrypt_token,
        db,
        server::{create_router, test_helpers::*},
    };
    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode, header},
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn settings_json_returns_defaults() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/settings")
                    .header(header::COOKIE, cookie)
                    .header(header::ACCEPT, "application/json")
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("failed to read body");
        let json: serde_json::Value = serde_json::from_slice(&body).expect("valid json response");
        assert_eq!(json["tripit_connected"], false);
        assert_eq!(json["email"], "");
        assert_eq!(json["first_name"], "");
        assert_eq!(json["last_name"], "");
        assert!(json["sync"].is_object());
        assert_eq!(json["sync"]["status"], "idle");
    }

    #[tokio::test]
    async fn settings_json_with_tripit_connected() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let user = db::users::GetByUsername { username: "alice" }
            .execute(&pool)
            .await
            .expect("lookup failed")
            .expect("missing user");
        let key = [7_u8; 32];
        let (access_token_enc, nonce_token) =
            encrypt_token("token", &key).expect("failed to encrypt access token");
        let (access_token_secret_enc, nonce_secret) =
            encrypt_token("secret", &key).expect("failed to encrypt access token secret");

        db::credentials::Upsert {
            user_id: user.id,
            access_token_enc: &access_token_enc,
            access_token_secret_enc: &access_token_secret_enc,
            nonce_token: &nonce_token,
            nonce_secret: &nonce_secret,
        }
        .execute(&pool)
        .await
        .expect("failed to upsert tripit credentials");

        let mut sync_state = db::sync_state::GetOrCreate { user_id: user.id }
            .execute(&pool)
            .await
            .expect("failed to fetch sync state");
        sync_state.sync_status = "idle".to_string();
        sync_state.last_sync_at = Some("2026-01-02 03:04:05".to_string());
        sync_state.trips_fetched = 3;
        sync_state.hops_fetched = 12;
        db::sync_state::Update {
            user_id: user.id,
            state: &sync_state,
        }
        .execute(&pool)
        .await
        .expect("failed to update sync state");

        let app = create_router(test_app_state(pool));
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/settings")
                    .header(header::COOKIE, cookie)
                    .header(header::ACCEPT, "application/json")
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("failed to read body");
        let json: serde_json::Value = serde_json::from_slice(&body).expect("valid json response");
        assert_eq!(json["tripit_connected"], true);
        assert_eq!(json["sync"]["status"], "idle");
        assert_eq!(json["sync"]["last_sync_at"], "2026-01-02 03:04:05");
        assert_eq!(json["sync"]["trips_fetched"], 3);
        assert_eq!(json["sync"]["journeys_fetched"], 12);
    }

    #[tokio::test]
    async fn settings_json_ignores_feedback_query_params() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/settings?error=Something+broke&tripit=connected&csv=42")
                    .header(header::COOKIE, cookie)
                    .header(header::ACCEPT, "application/json")
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("failed to read body");
        let json: serde_json::Value = serde_json::from_slice(&body).expect("valid json response");
        // JSON response should contain only tripit_connected and sync — no feedback fields
        assert!(json.get("error").is_none());
        assert!(json.get("tripit").is_none());
        assert!(json.get("csv").is_none());
        assert_eq!(json["tripit_connected"], false);
    }

    #[tokio::test]
    async fn settings_html_returns_rendered_page() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/settings")
                    .header(header::COOKIE, cookie)
                    .header(header::ACCEPT, "text/html")
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_text(response).await;
        assert!(body.contains("Settings"));
        assert!(body.contains("TripIt Connection"));
    }

    #[tokio::test]
    async fn settings_csv_returns_not_acceptable() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/settings")
                    .header(header::COOKIE, cookie)
                    .header(header::ACCEPT, "text/csv")
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::NOT_ACCEPTABLE);
    }

    #[tokio::test]
    async fn settings_without_auth_returns_unauthorized() {
        let pool = test_pool().await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/settings")
                    .header(header::ACCEPT, "application/json")
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}

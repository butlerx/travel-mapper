use super::ErrorResponse;
use crate::{
    db,
    server::{AppState, middleware::AuthUser, session::sha256_hex},
};
use aide::transform::TransformOperation;
use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use rand::RngCore;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Deserialize, JsonSchema)]
pub struct ApiKeyRequest {
    pub label: Option<String>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ApiKeyResponse {
    pub id: i64,
    pub key: String,
    pub label: String,
}

/// Create a new API key for programmatic access.
pub async fn create_api_key_handler(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<ApiKeyRequest>,
) -> Response {
    let mut key_bytes = [0_u8; 32];
    rand::thread_rng().fill_bytes(&mut key_bytes);
    let key = URL_SAFE_NO_PAD.encode(key_bytes);
    let key_hash = sha256_hex(&key);
    let label = body.label.unwrap_or_default();

    match db::create_api_key(&state.db, auth.user_id, &key_hash, &label).await {
        Ok(id) => (
            StatusCode::OK,
            Json(json!({ "id": id, "key": key, "label": label })),
        )
            .into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("failed to create api key: {err}") })),
        )
            .into_response(),
    }
}

pub fn create_api_key_handler_docs(op: TransformOperation) -> TransformOperation {
    op.description("Create a new API key for programmatic access.")
        .response::<200, Json<ApiKeyResponse>>()
        .response::<500, Json<ErrorResponse>>()
        .tag("auth")
}

#[cfg(test)]
mod tests {
    use crate::server::create_router;
    use crate::server::test_helpers::helpers::*;
    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode, header},
    };
    use serde_json::Value;
    use tower::ServiceExt;

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
}

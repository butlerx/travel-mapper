use super::{ErrorResponse, MultiFormatResponse, multi_format_docs, negotiate_format};
use crate::{
    db,
    server::{AppState, error::AppError, extractors::AuthUser, session::sha256_hex},
};
use aide::transform::TransformOperation;
use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::Response,
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use rand::RngCore;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Request body for creating an API key.
#[derive(Deserialize, JsonSchema)]
pub struct ApiKeyRequest {
    pub label: Option<String>,
}

/// JSON response after creating an API key.
#[derive(Debug, Default, Serialize, JsonSchema)]
pub struct ApiKeyResponse {
    pub id: i64,
    pub key: String,
    pub label: String,
}

impl MultiFormatResponse for ApiKeyResponse {
    const HTML_TITLE: &'static str = "API Key Created";
    const CSV_HEADERS: &'static [&'static str] = &["id", "key", "label"];

    fn csv_row(&self) -> Vec<String> {
        vec![self.id.to_string(), self.key.clone(), self.label.clone()]
    }
}

pub async fn handler(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    Json(body): Json<ApiKeyRequest>,
) -> Response {
    let mut key_bytes = [0_u8; 32];
    rand::thread_rng().fill_bytes(&mut key_bytes);
    let key = URL_SAFE_NO_PAD.encode(key_bytes);
    let key_hash = sha256_hex(&key);
    let label = body.label.unwrap_or_default();

    match (db::api_keys::Create {
        user_id: auth.user_id,
        key_hash: &key_hash,
        label: &label,
    })
    .execute(&state.db)
    .await
    {
        Ok(id) => {
            let response = ApiKeyResponse { id, key, label };
            let format = negotiate_format(&headers);
            ApiKeyResponse::single_format_response(&response, format, StatusCode::OK)
        }
        Err(err) => {
            let format = negotiate_format(&headers);
            AppError::from(err).into_format_response(format)
        }
    }
}

/// `OpenAPI` metadata for the create API key endpoint.
pub fn handler_docs(op: TransformOperation) -> TransformOperation {
    multi_format_docs!(
        op.description("Create a new API key for programmatic access."),
        200 => ApiKeyResponse,
        401 | 500 => ErrorResponse,
    )
    .tag("auth")
}

#[cfg(test)]
mod tests {
    use crate::server::create_router;
    use crate::server::test_helpers::*;
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

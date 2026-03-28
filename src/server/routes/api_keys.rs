use super::{ErrorResponse, MultiFormatResponse, multi_format_docs, negotiate_format};
use crate::{
    db,
    server::{AppState, error::AppError, extractors::AuthUser, session::sha256_hex},
};
use aide::transform::TransformOperation;
use axum::{
    body::Bytes,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Redirect, Response},
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use rand::RngCore;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Request body for creating an API key.
#[derive(Deserialize, Default, JsonSchema)]
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
    body: Bytes,
) -> Response {
    let parsed =
        match crate::server::extractors::FormOrJson::<ApiKeyRequest>::parse(&headers, &body) {
            Ok(v) => v,
            Err(err) => {
                let format = negotiate_format(&headers);
                return err.into_format_response(format);
            }
        };

    let mut key_bytes = [0_u8; 32];
    rand::thread_rng().fill_bytes(&mut key_bytes);
    let key = URL_SAFE_NO_PAD.encode(key_bytes);
    let key_hash = sha256_hex(&key);
    let label = parsed.label.unwrap_or_default();

    match (db::api_keys::Create {
        user_id: auth.user_id,
        key_hash: &key_hash,
        label: &label,
    })
    .execute(&state.db)
    .await
    {
        Ok(id) => {
            let format = negotiate_format(&headers);
            if format == super::ResponseFormat::Html {
                Redirect::to(&format!("/settings?new_api_key={key}")).into_response()
            } else {
                let response = ApiKeyResponse { id, key, label };
                ApiKeyResponse::single_format_response(&response, format, StatusCode::CREATED)
            }
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
        201 => ApiKeyResponse,
        401 | 500 => ErrorResponse,
    )
    .tag("auth")
}

pub async fn delete_handler(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Response {
    match (db::api_keys::Delete {
        id,
        user_id: auth.user_id,
    })
    .execute(&state.db)
    .await
    {
        Ok(true) => {
            let format = negotiate_format(&headers);
            if format == super::ResponseFormat::Html {
                Redirect::to("/settings").into_response()
            } else {
                StatusCode::NO_CONTENT.into_response()
            }
        }
        Ok(false) => {
            let format = negotiate_format(&headers);
            ErrorResponse::into_format_response("not found", format, StatusCode::NOT_FOUND)
        }
        Err(err) => {
            let format = negotiate_format(&headers);
            AppError::from(err).into_format_response(format)
        }
    }
}

/// `OpenAPI` metadata for the delete API key endpoint.
pub fn delete_handler_docs(op: TransformOperation) -> TransformOperation {
    multi_format_docs!(
        op.description("Revoke an API key."),
        404 | 401 | 500 => ErrorResponse,
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

        assert_eq!(response.status(), StatusCode::CREATED);
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("failed to read response body");
        let parsed: Value = serde_json::from_slice(&body).expect("json body");
        assert!(parsed["key"].as_str().is_some());
    }

    #[tokio::test]
    async fn delete_api_key_returns_no_content() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let app = create_router(test_app_state(pool.clone()));

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/auth/api-keys")
                    .header(header::CONTENT_TYPE, "application/json")
                    .header(header::COOKIE, &cookie)
                    .body(Body::from(r#"{"label":"temp"}"#))
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::CREATED);
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("failed to read response body");
        let parsed: Value = serde_json::from_slice(&body).expect("json body");
        let id = parsed["id"].as_i64().expect("id should be i64");

        let app2 = create_router(test_app_state(pool));
        let response = app2
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!("/auth/api-keys/{id}"))
                    .header(header::COOKIE, cookie)
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn delete_nonexistent_api_key_returns_not_found() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri("/auth/api-keys/99999")
                    .header(header::COOKIE, cookie)
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}

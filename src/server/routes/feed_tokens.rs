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

/// Request body for creating a calendar feed token.
#[derive(Deserialize, Default, JsonSchema)]
pub struct FeedTokenRequest {
    pub label: Option<String>,
}

/// JSON response after creating a calendar feed token.
#[derive(Debug, Default, Serialize, JsonSchema)]
pub struct FeedTokenResponse {
    pub id: i64,
    pub token: String,
    pub label: String,
}

impl MultiFormatResponse for FeedTokenResponse {
    const HTML_TITLE: &'static str = "Feed Token Created";
    const CSV_HEADERS: &'static [&'static str] = &["id", "token", "label"];

    fn csv_row(&self) -> Vec<String> {
        vec![self.id.to_string(), self.token.clone(), self.label.clone()]
    }
}

pub async fn create_handler(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let parsed =
        match crate::server::extractors::FormOrJson::<FeedTokenRequest>::parse(&headers, &body) {
            Ok(v) => v,
            Err(err) => {
                let format = negotiate_format(&headers);
                return err.into_format_response(format);
            }
        };

    let mut token_bytes = [0_u8; 32];
    rand::thread_rng().fill_bytes(&mut token_bytes);
    let token = URL_SAFE_NO_PAD.encode(token_bytes);
    let token_hash = sha256_hex(&token);
    let label = parsed.label.unwrap_or_default();

    match (db::feed_tokens::Create {
        user_id: auth.user_id,
        token_hash: &token_hash,
        label: &label,
    })
    .execute(&state.db)
    .await
    {
        Ok(id) => {
            let format = negotiate_format(&headers);
            if format == super::ResponseFormat::Html {
                Redirect::to("/settings").into_response()
            } else {
                let response = FeedTokenResponse {
                    id,
                    token: token_hash,
                    label,
                };
                FeedTokenResponse::single_format_response(&response, format, StatusCode::CREATED)
            }
        }
        Err(err) => {
            let format = negotiate_format(&headers);
            AppError::from(err).into_format_response(format)
        }
    }
}

/// `OpenAPI` metadata for the create feed token endpoint.
pub fn create_handler_docs(op: TransformOperation) -> TransformOperation {
    multi_format_docs!(
        op.description("Create a new calendar feed token."),
        201 => FeedTokenResponse,
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
    match (db::feed_tokens::Delete {
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

/// `OpenAPI` metadata for the delete feed token endpoint.
pub fn delete_handler_docs(op: TransformOperation) -> TransformOperation {
    multi_format_docs!(
        op.description("Revoke a calendar feed token."),
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
    async fn create_feed_token_returns_token() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/auth/feed-tokens")
                    .header(header::CONTENT_TYPE, "application/json")
                    .header(header::COOKIE, cookie)
                    .body(Body::from(r#"{"label":"my calendar"}"#))
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::CREATED);
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("failed to read response body");
        let parsed: Value = serde_json::from_slice(&body).expect("json body");
        assert!(parsed["token"].as_str().is_some());
        assert_eq!(parsed["label"], "my calendar");
    }

    #[tokio::test]
    async fn delete_feed_token_returns_no_content() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let app = create_router(test_app_state(pool.clone()));

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/auth/feed-tokens")
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
                    .uri(format!("/auth/feed-tokens/{id}"))
                    .header(header::COOKIE, cookie)
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn delete_nonexistent_feed_token_returns_not_found() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri("/auth/feed-tokens/99999")
                    .header(header::COOKIE, cookie)
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}

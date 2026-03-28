use super::{
    ErrorResponse, MultiFormatResponse, StatusResponse, multi_format_docs, negotiate_format,
};
use crate::{
    db,
    server::{AppState, error::AppError, extractors::AuthUser},
};
use aide::transform::TransformOperation;
use axum::{
    Json,
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Redirect, Response},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Push subscription encryption keys for Web Push notifications.
#[derive(Debug, Deserialize, Default, JsonSchema)]
pub struct SubscriptionKeys {
    pub p256dh: String,
    pub auth: String,
}

/// Request body for subscribing to Web Push notifications.
#[derive(Debug, Deserialize, Default, JsonSchema)]
pub struct SubscribeRequest {
    pub endpoint: String,
    pub keys: SubscriptionKeys,
}

impl MultiFormatResponse for VapidPublicKeyResponse {
    const HTML_TITLE: &'static str = "VAPID Public Key";
    const CSV_HEADERS: &'static [&'static str] = &["key"];

    fn csv_row(&self) -> Vec<String> {
        vec![self.key.clone()]
    }
}

/// Request body for unsubscribing from Web Push notifications.
#[derive(Debug, Deserialize, Default, JsonSchema)]
pub struct UnsubscribeRequest {
    pub endpoint: String,
}

/// JSON response containing the server's VAPID public key.
#[derive(Debug, Serialize, Default, JsonSchema)]
pub struct VapidPublicKeyResponse {
    pub key: String,
}

pub async fn subscribe_handler(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let parsed =
        match crate::server::extractors::FormOrJson::<SubscribeRequest>::parse(&headers, &body) {
            Ok(v) => v,
            Err(err) => {
                let format = negotiate_format(&headers);
                return err.into_format_response(format);
            }
        };

    match (db::push_subscriptions::Create {
        user_id: auth.user_id,
        endpoint: &parsed.endpoint,
        p256dh: &parsed.keys.p256dh,
        auth: &parsed.keys.auth,
    })
    .execute(&state.db)
    .await
    {
        Ok(_) => {
            let format = negotiate_format(&headers);
            if format == super::ResponseFormat::Html {
                Redirect::to("/settings").into_response()
            } else {
                let response = StatusResponse {
                    status: "subscribed".to_string(),
                };
                StatusResponse::single_format_response(&response, format, StatusCode::CREATED)
            }
        }
        Err(err) => {
            let format = negotiate_format(&headers);
            AppError::from(err).into_format_response(format)
        }
    }
}

/// `OpenAPI` metadata for the subscribe to push notifications endpoint.
pub fn subscribe_handler_docs(op: TransformOperation) -> TransformOperation {
    multi_format_docs!(
        op.description("Store or update a Web Push subscription for the authenticated user."),
        201 => StatusResponse,
        401 | 500 => ErrorResponse,
    )
    .tag("auth")
}

pub async fn unsubscribe_handler(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let parsed =
        match crate::server::extractors::FormOrJson::<UnsubscribeRequest>::parse(&headers, &body) {
            Ok(v) => v,
            Err(err) => {
                let format = negotiate_format(&headers);
                return err.into_format_response(format);
            }
        };

    match (db::push_subscriptions::DeleteByUserAndEndpoint {
        user_id: auth.user_id,
        endpoint: &parsed.endpoint,
    })
    .execute(&state.db)
    .await
    {
        Ok(_) => {
            let format = negotiate_format(&headers);
            if format == super::ResponseFormat::Html {
                Redirect::to("/settings").into_response()
            } else {
                StatusCode::NO_CONTENT.into_response()
            }
        }
        Err(err) => {
            let format = negotiate_format(&headers);
            AppError::from(err).into_format_response(format)
        }
    }
}

/// `OpenAPI` metadata for the unsubscribe from push notifications endpoint.
pub fn unsubscribe_handler_docs(op: TransformOperation) -> TransformOperation {
    multi_format_docs!(
        op.description("Delete a Web Push subscription for the authenticated user."),
        401 | 500 => ErrorResponse,
    )
    .tag("auth")
}

pub async fn vapid_key_handler(State(state): State<AppState>, _auth: AuthUser) -> Response {
    match state.vapid_public_key {
        Some(key) => (StatusCode::OK, Json(VapidPublicKeyResponse { key })).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

/// `OpenAPI` metadata for the VAPID public key endpoint.
pub fn vapid_key_handler_docs(op: TransformOperation) -> TransformOperation {
    multi_format_docs!(
        op.description("Get the configured VAPID public key used for browser push subscriptions."),
        200 => VapidPublicKeyResponse,
        401 | 404 | 500 => ErrorResponse,
    )
    .tag("auth")
}

#[cfg(test)]
mod tests {
    use crate::server::{create_router, test_helpers::*};
    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode, header},
    };
    use serde_json::Value;
    use tower::ServiceExt;

    #[tokio::test]
    async fn subscribe_returns_created() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/auth/push-subscribe")
                    .header(header::CONTENT_TYPE, "application/json")
                    .header(header::COOKIE, cookie)
                    .body(Body::from(
                        r#"{"endpoint":"https://example.com/sub","keys":{"p256dh":"abc","auth":"xyz"}}"#,
                    ))
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::CREATED);
    }

    #[tokio::test]
    async fn unsubscribe_returns_no_content() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let app = create_router(test_app_state(pool.clone()));

        let subscribe_response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/auth/push-subscribe")
                    .header(header::CONTENT_TYPE, "application/json")
                    .header(header::COOKIE, &cookie)
                    .body(Body::from(
                        r#"{"endpoint":"https://example.com/sub","keys":{"p256dh":"abc","auth":"xyz"}}"#,
                    ))
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");
        assert_eq!(subscribe_response.status(), StatusCode::CREATED);

        let app2 = create_router(test_app_state(pool));
        let response = app2
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri("/auth/push-subscribe")
                    .header(header::CONTENT_TYPE, "application/json")
                    .header(header::COOKIE, cookie)
                    .body(Body::from(r#"{"endpoint":"https://example.com/sub"}"#))
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn vapid_key_returns_key() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let mut state = test_app_state(pool);
        state.vapid_public_key = Some("public-key-value".to_string());
        let app = create_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/auth/vapid-public-key")
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
        let parsed: Value = serde_json::from_slice(&body).expect("json body");
        assert_eq!(parsed["key"], "public-key-value");
    }

    #[tokio::test]
    async fn vapid_key_returns_not_found_when_not_configured() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/auth/vapid-public-key")
                    .header(header::COOKIE, cookie)
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}

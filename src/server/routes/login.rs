use super::{ErrorResponse, MultiFormatResponse, multi_format_docs, negotiate_format};
use crate::server::{
    AppState,
    session::{create_user_session, is_form_request, session_cookie, verify_credentials},
};
use aide::transform::TransformOperation;
use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::CookieJar;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Credentials for logging in.
#[derive(Deserialize, JsonSchema)]
pub struct LoginRequest {
    /// Account username.
    pub username: String,
    /// Account password.
    pub password: String,
}

/// Successful authentication response.
#[derive(Debug, Default, Serialize, JsonSchema)]
pub struct AuthResponse {
    /// Unique user identifier.
    pub id: i64,
    /// Username of the authenticated user.
    pub username: String,
}

impl MultiFormatResponse for AuthResponse {
    const HTML_TITLE: &'static str = "Authentication";
    const CSV_HEADERS: &'static [&'static str] = &["id", "username"];

    fn csv_row(&self) -> Vec<String> {
        vec![self.id.to_string(), self.username.clone()]
    }
}

/// Log in with username and password.
///
/// Accepts JSON or form-encoded body. On success, sets a session cookie.
pub async fn login_handler(
    State(state): State<AppState>,
    jar: CookieJar,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> (CookieJar, Response) {
    let parsed: Result<LoginRequest, String> = if is_form_request(&headers) {
        serde_urlencoded::from_bytes(&body).map_err(|e| e.to_string())
    } else {
        serde_json::from_slice(&body).map_err(|e| e.to_string())
    };

    let body = match parsed {
        Ok(b) => b,
        Err(err) => {
            return if is_form_request(&headers) {
                (
                    jar,
                    Redirect::to("/login?error=Invalid+form+data").into_response(),
                )
            } else {
                let format = negotiate_format(&headers);
                (
                    jar,
                    ErrorResponse::into_format_response(
                        format!("invalid request body: {err}"),
                        format,
                        StatusCode::BAD_REQUEST,
                    ),
                )
            };
        }
    };

    let is_form = is_form_request(&headers);
    let user = match verify_credentials(&state.db, &body.username, &body.password).await {
        Ok(user) => user,
        Err((status, msg)) => {
            return (
                jar,
                if is_form && status == StatusCode::UNAUTHORIZED {
                    Redirect::to("/login?error=Invalid+credentials").into_response()
                } else {
                    let format = negotiate_format(&headers);
                    ErrorResponse::into_format_response(msg, format, status)
                },
            );
        }
    };

    let token = match create_user_session(&state.db, user.id).await {
        Ok((t, _)) => t,
        Err((status, msg)) => {
            let format = negotiate_format(&headers);
            return (
                jar,
                ErrorResponse::into_format_response(msg, format, status),
            );
        }
    };

    let updated_jar = jar.add(session_cookie(&token));
    (
        updated_jar,
        if is_form {
            Redirect::to("/dashboard").into_response()
        } else {
            let format = negotiate_format(&headers);
            let response = AuthResponse {
                id: user.id,
                username: user.username,
            };
            AuthResponse::single_format_response(&response, format, StatusCode::OK)
        },
    )
}

pub fn login_handler_docs(op: TransformOperation) -> TransformOperation {
    multi_format_docs!(
        op.description("Log in with username and password. Accepts JSON or form-encoded body.")
            .input::<Json<LoginRequest>>()
            .with(|mut op| {
                if let Some(aide::openapi::ReferenceOr::Item(body)) =
                    &mut op.inner_mut().request_body
                    && let Some(json_media) = body.content.get("application/json").cloned()
                {
                    body.content
                        .insert("application/x-www-form-urlencoded".to_string(), json_media);
                }
                op
            }),
        200 => AuthResponse,
        400 | 401 | 500 => ErrorResponse,
    )
    .tag("auth")
}

#[cfg(test)]
mod tests {
    use crate::server::create_router;
    use crate::server::test_helpers::helpers::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode, header},
    };
    use tower::ServiceExt;

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
}

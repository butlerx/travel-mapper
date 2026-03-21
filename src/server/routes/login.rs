use crate::server::{
    AppState,
    routes::ErrorResponse,
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
use serde_json::json;

#[derive(Deserialize, JsonSchema)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct AuthResponse {
    pub id: i64,
    pub username: String,
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
                (
                    jar,
                    (
                        StatusCode::BAD_REQUEST,
                        Json(json!({ "error": format!("invalid request body: {err}") })),
                    )
                        .into_response(),
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
                    (status, Json(json!({ "error": msg }))).into_response()
                },
            );
        }
    };

    let token = match create_user_session(&state.db, user.id).await {
        Ok((t, _)) => t,
        Err((status, msg)) => {
            return (jar, (status, Json(json!({ "error": msg }))).into_response());
        }
    };

    let updated_jar = jar.add(session_cookie(&token));
    (
        updated_jar,
        if is_form {
            Redirect::to("/dashboard").into_response()
        } else {
            (
                StatusCode::OK,
                Json(json!({ "id": user.id, "username": user.username })),
            )
                .into_response()
        },
    )
}

pub fn login_handler_docs(op: TransformOperation) -> TransformOperation {
    op.description("Log in with username and password.")
        .response::<200, Json<AuthResponse>>()
        .response::<400, Json<ErrorResponse>>()
        .response::<401, Json<ErrorResponse>>()
        .response::<500, Json<ErrorResponse>>()
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

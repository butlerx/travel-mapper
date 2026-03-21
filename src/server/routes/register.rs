use crate::{
    auth::hash_password,
    db,
    server::{
        AppState,
        routes::ErrorResponse,
        session::{create_user_session, is_form_request, session_cookie},
    },
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
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize, JsonSchema)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
}

/// Register a new user account.
///
/// Accepts JSON or form-encoded body. On success, sets a session cookie.
pub async fn register_handler(
    State(state): State<AppState>,
    jar: CookieJar,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> (CookieJar, Response) {
    let parsed: Result<RegisterRequest, String> = if is_form_request(&headers) {
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
                    Redirect::to("/register?error=Invalid+form+data").into_response(),
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

    let hash = match hash_password(&body.password) {
        Ok(hash) => hash,
        Err(err) => {
            return (
                jar,
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": format!("failed to hash password: {err}") })),
                )
                    .into_response(),
            );
        }
    };

    match db::create_user(&state.db, &body.username, &hash).await {
        Ok(id) => {
            let token = match create_user_session(&state.db, id).await {
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
                        StatusCode::CREATED,
                        Json(json!({ "id": id, "username": body.username })),
                    )
                        .into_response()
                },
            )
        }
        Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => (
            jar,
            if is_form {
                Redirect::to("/register?error=Username+already+exists").into_response()
            } else {
                (
                    StatusCode::CONFLICT,
                    Json(json!({ "error": "username already exists" })),
                )
                    .into_response()
            },
        ),
        Err(err) => (
            jar,
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("failed to create user: {err}") })),
            )
                .into_response(),
        ),
    }
}

pub fn register_handler_docs(op: TransformOperation) -> TransformOperation {
    op.description("Register a new user account.")
        .response::<201, Json<crate::server::routes::AuthResponse>>()
        .response::<400, Json<ErrorResponse>>()
        .response::<409, Json<ErrorResponse>>()
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
}

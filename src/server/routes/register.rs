use super::{
    AuthResponse, ErrorResponse, MultiFormatResponse, multi_format_docs, negotiate_format,
};
use crate::{
    auth::hash_password,
    db,
    server::{
        AppState,
        error::AppError,
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

/// Credentials for creating a new account.
#[derive(Deserialize, JsonSchema)]
pub struct RegisterRequest {
    /// Desired username (must be unique).
    pub username: String,
    /// Password for the new account.
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
    let parsed: Result<RegisterRequest, AppError> = if is_form_request(&headers) {
        serde_urlencoded::from_bytes(&body).map_err(AppError::from)
    } else {
        serde_json::from_slice(&body).map_err(AppError::from)
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
                let format = negotiate_format(&headers);
                (jar, err.into_format_response(format))
            };
        }
    };

    let is_form = is_form_request(&headers);

    let hash = match hash_password(&body.password) {
        Ok(hash) => hash,
        Err(err) => {
            let err = AppError::from(err);
            return if is_form_request(&headers) {
                (
                    jar,
                    Redirect::to("/register?error=Registration+failed").into_response(),
                )
            } else {
                let format = negotiate_format(&headers);
                (jar, err.into_format_response(format))
            };
        }
    };

    create_and_authenticate(state, jar, &headers, is_form, &body.username, &hash).await
}

async fn create_and_authenticate(
    state: AppState,
    jar: CookieJar,
    headers: &HeaderMap,
    is_form: bool,
    username: &str,
    hash: &str,
) -> (CookieJar, Response) {
    match (db::users::Create {
        username,
        password_hash: hash,
    })
    .execute(&state.db)
    .await
    {
        Ok(id) => {
            let token = match create_user_session(&state.db, id).await {
                Ok((t, _)) => t,
                Err(err) => {
                    let format = negotiate_format(headers);
                    return (jar, err.into_format_response(format));
                }
            };

            let updated_jar = jar.add(session_cookie(&token));
            (
                updated_jar,
                if is_form {
                    Redirect::to("/dashboard").into_response()
                } else {
                    let format = negotiate_format(headers);
                    let response = AuthResponse {
                        id,
                        username: username.to_owned(),
                    };
                    AuthResponse::single_format_response(&response, format, StatusCode::CREATED)
                },
            )
        }
        Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => {
            let err = AppError::UsernameExists;
            (
                jar,
                if is_form {
                    Redirect::to("/register?error=Username+already+exists").into_response()
                } else {
                    let format = negotiate_format(headers);
                    err.into_format_response(format)
                },
            )
        }
        Err(err) => {
            let err = AppError::from(err);
            let format = negotiate_format(headers);
            (jar, err.into_format_response(format))
        }
    }
}

pub fn register_handler_docs(op: TransformOperation) -> TransformOperation {
    multi_format_docs!(
        op.description("Register a new user account. Accepts JSON or form-encoded body.")
            .input::<Json<RegisterRequest>>()
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
        201 => AuthResponse,
        400 | 409 | 500 => ErrorResponse,
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

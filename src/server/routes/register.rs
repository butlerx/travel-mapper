use super::{
    AuthResponse, ErrorResponse, MultiFormatResponse, multi_format_docs, negotiate_format,
};
use crate::{
    auth::hash_password,
    db,
    server::{
        AppState,
        error::AppError,
        extractors::FormOrJson,
        session::{create_user_session, is_form_request, session_cookie, sha256_hex},
    },
};
use aide::transform::TransformOperation;
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::CookieJar;
use schemars::JsonSchema;
use serde::Deserialize;
use uuid::Uuid;

struct RegistrationInput<'a> {
    username: &'a str,
    password_hash: &'a str,
    email: &'a str,
    first_name: &'a str,
    last_name: &'a str,
}

/// Credentials for creating a new account.
#[derive(Default, Deserialize, JsonSchema)]
pub struct RegisterRequest {
    /// Desired username (must be unique).
    pub username: String,
    /// Password for the new account.
    pub password: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
}

/// Register a new user account.
///
/// Accepts JSON or form-encoded body. On success, sets a session cookie.
pub async fn handler(
    State(state): State<AppState>,
    jar: CookieJar,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> (CookieJar, Response) {
    if !state.registration_enabled {
        return if is_form_request(&headers) {
            (
                jar,
                Redirect::to("/login?error=Registration+is+disabled").into_response(),
            )
        } else {
            let format = negotiate_format(&headers);
            (
                jar,
                ErrorResponse::into_format_response(
                    "Registration is disabled",
                    format,
                    StatusCode::FORBIDDEN,
                ),
            )
        };
    }

    let parsed = match FormOrJson::<RegisterRequest>::parse(&headers, &body) {
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

    let hash = match hash_password(&parsed.password) {
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

    create_and_authenticate(
        state,
        jar,
        &headers,
        is_form,
        RegistrationInput {
            username: &parsed.username,
            password_hash: &hash,
            email: &parsed.email,
            first_name: &parsed.first_name,
            last_name: &parsed.last_name,
        },
    )
    .await
}

async fn create_and_authenticate(
    state: AppState,
    jar: CookieJar,
    headers: &HeaderMap,
    is_form: bool,
    input: RegistrationInput<'_>,
) -> (CookieJar, Response) {
    match (db::users::Create {
        username: input.username,
        password_hash: input.password_hash,
        email: input.email,
        first_name: input.first_name,
        last_name: input.last_name,
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

            if !input.email.is_empty() {
                let raw_token = Uuid::new_v4().to_string();
                let token_hash = sha256_hex(&raw_token);
                let pool = state.db.clone();
                let smtp = state.smtp_config.clone();
                let email_owned = input.email.to_owned();
                let raw_token_owned = raw_token;
                tokio::spawn(async move {
                    let expires_at =
                        sqlx::query_scalar::<_, Option<String>>("SELECT datetime('now', '+1 day')")
                            .fetch_one(&pool)
                            .await
                            .ok()
                            .flatten()
                            .unwrap_or_else(|| "2099-01-01 00:00:00".to_string());

                    if let Err(err) = (db::email_verifications::Create {
                        user_id: id,
                        token_hash: &token_hash,
                        expires_at: &expires_at,
                    })
                    .execute(&pool)
                    .await
                    {
                        tracing::error!(user_id = id, error = %err, "failed to store verification token");
                        return;
                    }

                    let base_url = "";
                    if let Err(err) = crate::server::email::send_verification_email(
                        smtp.as_ref(),
                        &email_owned,
                        &raw_token_owned,
                        base_url,
                    )
                    .await
                    {
                        tracing::warn!(user_id = id, error = %err, "failed to send verification email");
                    }
                });
            }

            let updated_jar = jar.add(session_cookie(&token));
            (
                updated_jar,
                if is_form {
                    Redirect::to("/dashboard").into_response()
                } else {
                    let format = negotiate_format(headers);
                    let response = AuthResponse {
                        id,
                        username: input.username.to_owned(),
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

pub fn handler_docs(op: TransformOperation) -> TransformOperation {
    multi_format_docs!(
        op.description("Register a new user account. Accepts JSON or form-encoded body.")
            .input::<FormOrJson<RegisterRequest>>(),
        201 => AuthResponse,
        400 | 409 | 500 => ErrorResponse,
    )
    .tag("auth")
}

#[cfg(test)]
mod tests {
    use crate::server::create_router;
    use crate::server::test_helpers::*;
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
                            .body(Body::from(r#"{"username":"alice","password":"secret","email":"alice@test.com","first_name":"Alice","last_name":"Test"}"#))
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
                .body(Body::from(r#"{"username":"alice","password":"secret","email":"alice@test.com","first_name":"Alice","last_name":"Test"}"#))
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

    #[tokio::test]
    async fn register_returns_forbidden_when_disabled() {
        let pool = test_pool().await;
        let mut state = test_app_state(pool);
        state.registration_enabled = false;
        let app = create_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/auth/register")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(r#"{"username":"alice","password":"secret","email":"alice@test.com","first_name":"Alice","last_name":"Test"}"#))
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }
}

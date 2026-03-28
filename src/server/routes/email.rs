use super::{ErrorResponse, MultiFormatResponse, StatusResponse, negotiate_format};
use crate::{
    db,
    server::{
        AppState,
        error::AppError,
        extractors::{AuthUser, FormOrJson},
        session::{is_form_request, sha256_hex},
    },
};
use aide::transform::TransformOperation;
use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Redirect, Response},
};
use schemars::JsonSchema;
use serde::Deserialize;
use uuid::Uuid;

/// Payload for updating the user's email address.
#[derive(Default, Deserialize, JsonSchema)]
pub struct UpdateEmailRequest {
    /// New email address for the account.
    pub email: String,
}

/// Update the authenticated user's email address.
///
/// Resets email verification status and sends a new verification email
/// if SMTP is configured.
pub async fn update_handler(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let parsed = match FormOrJson::<UpdateEmailRequest>::parse(&headers, &body) {
        Ok(v) => v,
        Err(err) => {
            let format = negotiate_format(&headers);
            return err.into_format_response(format);
        }
    };

    let email = parsed.email.trim();
    if email.is_empty() {
        return if is_form_request(&headers) {
            Redirect::to("/settings?error=Email+address+is+required").into_response()
        } else {
            let format = negotiate_format(&headers);
            ErrorResponse::into_format_response(
                "Email address is required",
                format,
                StatusCode::BAD_REQUEST,
            )
        };
    }

    if let Err(err) = (db::users::UpdateEmail {
        user_id: auth.user_id,
        email,
    })
    .execute(&state.db)
    .await
    {
        if matches!(&err, sqlx::Error::Database(db_err) if db_err.is_unique_violation()) {
            return if is_form_request(&headers) {
                Redirect::to("/settings?error=Email+address+already+in+use").into_response()
            } else {
                let format = negotiate_format(&headers);
                ErrorResponse::into_format_response(
                    "Email address already in use",
                    format,
                    StatusCode::CONFLICT,
                )
            };
        }
        let format = negotiate_format(&headers);
        return AppError::from(err).into_format_response(format);
    }

    let _ = (db::email_verifications::DeleteByUserId {
        user_id: auth.user_id,
    })
    .execute(&state.db)
    .await;

    spawn_verification_email(&state, auth.user_id, email);

    if is_form_request(&headers) {
        Redirect::to("/settings?email=updated").into_response()
    } else {
        let format = negotiate_format(&headers);
        let resp = StatusResponse {
            status: "ok".to_owned(),
        };
        StatusResponse::single_format_response(&resp, format, StatusCode::OK)
    }
}

/// `OpenAPI` metadata for the update email endpoint.
pub fn update_handler_docs(op: TransformOperation) -> TransformOperation {
    op.description("Update the authenticated user's email address.")
        .input::<FormOrJson<UpdateEmailRequest>>()
        .response::<200, axum::Json<StatusResponse>>()
        .response::<400, axum::Json<ErrorResponse>>()
        .response::<401, axum::Json<ErrorResponse>>()
        .response::<409, axum::Json<ErrorResponse>>()
        .response::<500, axum::Json<ErrorResponse>>()
        .tag("auth")
}

/// Resend the email verification link to the authenticated user's current email.
///
/// Deletes any existing tokens and generates a fresh one.
pub async fn resend_handler(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
) -> Response {
    let user = match (db::users::GetById { id: auth.user_id })
        .execute(&state.db)
        .await
    {
        Ok(Some(u)) => u,
        Ok(None) => {
            return if is_form_request(&headers) {
                Redirect::to("/settings?error=User+not+found").into_response()
            } else {
                let format = negotiate_format(&headers);
                ErrorResponse::into_format_response("User not found", format, StatusCode::NOT_FOUND)
            };
        }
        Err(err) => {
            let format = negotiate_format(&headers);
            return AppError::from(err).into_format_response(format);
        }
    };

    if user.email.is_empty() {
        return if is_form_request(&headers) {
            Redirect::to("/settings?error=No+email+address+set").into_response()
        } else {
            let format = negotiate_format(&headers);
            ErrorResponse::into_format_response(
                "No email address set",
                format,
                StatusCode::BAD_REQUEST,
            )
        };
    }

    if user.email_verified {
        return if is_form_request(&headers) {
            Redirect::to("/settings?error=Email+already+verified").into_response()
        } else {
            let format = negotiate_format(&headers);
            ErrorResponse::into_format_response(
                "Email already verified",
                format,
                StatusCode::BAD_REQUEST,
            )
        };
    }

    let _ = (db::email_verifications::DeleteByUserId {
        user_id: auth.user_id,
    })
    .execute(&state.db)
    .await;

    spawn_verification_email(&state, auth.user_id, &user.email);

    if is_form_request(&headers) {
        Redirect::to("/settings?email=resent").into_response()
    } else {
        let format = negotiate_format(&headers);
        let resp = StatusResponse {
            status: "ok".to_owned(),
        };
        StatusResponse::single_format_response(&resp, format, StatusCode::OK)
    }
}

/// `OpenAPI` metadata for the resend verification email endpoint.
pub fn resend_handler_docs(op: TransformOperation) -> TransformOperation {
    op.description("Resend the email verification link.")
        .response::<200, axum::Json<StatusResponse>>()
        .response::<400, axum::Json<ErrorResponse>>()
        .response::<401, axum::Json<ErrorResponse>>()
        .response::<404, axum::Json<ErrorResponse>>()
        .response::<500, axum::Json<ErrorResponse>>()
        .tag("auth")
}

/// Spawn a background task that creates a verification token and sends the
/// verification email.
fn spawn_verification_email(state: &AppState, user_id: i64, email: &str) {
    let raw_token = Uuid::new_v4().to_string();
    let token_hash = sha256_hex(&raw_token);
    let pool = state.db.clone();
    let smtp = state.smtp_config.clone();
    let email_owned = email.to_owned();

    tokio::spawn(async move {
        let expires_at =
            sqlx::query_scalar::<_, Option<String>>("SELECT datetime('now', '+1 day')")
                .fetch_one(&pool)
                .await
                .ok()
                .flatten()
                .unwrap_or_else(|| "2099-01-01 00:00:00".to_string());

        if let Err(err) = (db::email_verifications::Create {
            user_id,
            token_hash: &token_hash,
            expires_at: &expires_at,
        })
        .execute(&pool)
        .await
        {
            tracing::error!(user_id, error = %err, "failed to store verification token");
            return;
        }

        let base_url = "";
        if let Err(err) = crate::server::email::send_verification_email(
            smtp.as_ref(),
            &email_owned,
            &raw_token,
            base_url,
        )
        .await
        {
            tracing::warn!(user_id, error = %err, "failed to send verification email");
        }
    });
}

#[cfg(test)]
mod tests {
    use crate::server::create_router;
    use crate::server::test_helpers::*;
    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode, header},
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn update_email_sets_new_address() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/auth/email")
                    .header(header::CONTENT_TYPE, "application/json")
                    .header(header::COOKIE, cookie)
                    .body(Body::from(r#"{"email":"alice@example.com"}"#))
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("failed to read body");
        let json: serde_json::Value = serde_json::from_slice(&body).expect("valid json");
        assert_eq!(json["status"], "ok");
    }

    #[tokio::test]
    async fn update_email_rejects_empty() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/auth/email")
                    .header(header::CONTENT_TYPE, "application/json")
                    .header(header::COOKIE, cookie)
                    .body(Body::from(r#"{"email":"  "}"#))
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn resend_without_email_returns_bad_request() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/auth/resend-verification")
                    .header(header::COOKIE, cookie)
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn update_email_form_redirects_to_settings() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/auth/email")
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .header(header::COOKIE, cookie)
                    .body(Body::from("email=alice%40example.com"))
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert!(
            response
                .headers()
                .get(header::LOCATION)
                .and_then(|v| v.to_str().ok())
                .unwrap_or("")
                .contains("/settings")
        );
    }
}

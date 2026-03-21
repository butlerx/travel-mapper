//! Shared session and authentication utilities.

use crate::db;
use axum::http::{HeaderMap, StatusCode, header};
use axum_extra::extract::cookie::{Cookie, SameSite};
use sha2::{Digest, Sha256};
use uuid::Uuid;

#[must_use]
pub fn sha256_hex(value: &str) -> String {
    let digest = Sha256::digest(value.as_bytes());
    digest
        .iter()
        .fold(String::with_capacity(digest.len() * 2), |mut acc, byte| {
            use std::fmt::Write;
            let _ = write!(acc, "{byte:02x}");
            acc
        })
}

#[must_use]
pub fn session_cookie(token: &str) -> Cookie<'static> {
    Cookie::build(("session_id", token.to_string()))
        .http_only(true)
        .same_site(SameSite::Lax)
        .path("/")
        .build()
}

#[must_use]
pub fn clear_session_cookie() -> Cookie<'static> {
    Cookie::build(("session_id", String::new()))
        .http_only(true)
        .same_site(SameSite::Lax)
        .path("/")
        .build()
}

pub fn is_form_request(headers: &HeaderMap) -> bool {
    headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|ct| ct.starts_with("application/x-www-form-urlencoded"))
}

/// Create a new session token and expiry for the given user.
///
/// # Errors
///
/// Returns a status code and message if the session cannot be created.
pub async fn create_user_session(
    db: &sqlx::SqlitePool,
    user_id: i64,
) -> Result<(String, String), (StatusCode, String)> {
    let token = Uuid::new_v4().to_string();
    let expires_at =
        match sqlx::query_scalar::<_, Option<String>>("SELECT datetime('now', '+7 days')")
            .fetch_one(db)
            .await
        {
            Ok(Some(value)) => value,
            Ok(None) => {
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to generate session expiry".to_string(),
                ));
            }
            Err(err) => {
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("failed to generate session expiry: {err}"),
                ));
            }
        };

    db::sessions::Create {
        token: &token,
        user_id,
        expires_at: &expires_at,
    }
    .execute(db)
    .await
    .map_err(|err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("failed to create session: {err}"),
        )
    })?;

    Ok((token, expires_at))
}

/// Look up a user by username and verify the password.
///
/// # Errors
///
/// Returns a status code and message if credentials are invalid or a database error occurs.
pub async fn verify_credentials(
    db: &sqlx::SqlitePool,
    username: &str,
    password: &str,
) -> Result<db::users::Row, (StatusCode, String)> {
    use crate::auth::verify_password;

    let user = db::users::GetByUsername { username }
        .execute(db)
        .await
        .map_err(|err| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed to lookup user: {err}"),
            )
        })?
        .ok_or((StatusCode::UNAUTHORIZED, "invalid credentials".to_string()))?;

    let verified = verify_password(password, &user.password_hash).map_err(|err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("failed to verify password: {err}"),
        )
    })?;

    if verified {
        Ok(user)
    } else {
        Err((StatusCode::UNAUTHORIZED, "invalid credentials".to_string()))
    }
}

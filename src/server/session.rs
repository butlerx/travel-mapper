//! Shared session and authentication utilities.

use crate::{auth::verify_password, db, server::error::AppError};
use axum::http::{HeaderMap, header};
use axum_extra::extract::cookie::{Cookie, SameSite};
use sha2::{Digest, Sha256};
use uuid::Uuid;

#[must_use]
pub(crate) fn sha256_hex(value: &str) -> String {
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
pub(crate) fn session_cookie(token: &str) -> Cookie<'static> {
    Cookie::build(("session_id", token.to_string()))
        .http_only(true)
        .same_site(SameSite::Lax)
        .path("/")
        .build()
}

#[must_use]
pub(crate) fn clear_session_cookie() -> Cookie<'static> {
    Cookie::build(("session_id", String::new()))
        .http_only(true)
        .same_site(SameSite::Lax)
        .path("/")
        .build()
}

pub(crate) fn is_form_request(headers: &HeaderMap) -> bool {
    headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|ct| ct.starts_with("application/x-www-form-urlencoded"))
}

/// Create a new session token and expiry for the given user.
///
/// # Errors
///
/// Returns [`AppError::Db`] if the session cannot be created.
pub(crate) async fn create_user_session(
    db: &sqlx::SqlitePool,
    user_id: i64,
) -> Result<(String, String), AppError> {
    let token = Uuid::new_v4().to_string();
    let expires_at = sqlx::query_scalar::<_, Option<String>>("SELECT datetime('now', '+7 days')")
        .fetch_one(db)
        .await?
        .ok_or_else(|| sqlx::Error::ColumnDecode {
            index: "datetime".to_string(),
            source: Box::new(std::fmt::Error),
        })?;

    db::sessions::Create {
        token: &token,
        user_id,
        expires_at: &expires_at,
    }
    .execute(db)
    .await?;

    Ok((token, expires_at))
}

/// Look up a user by username and verify the password.
///
/// # Errors
///
/// Returns [`AppError::InvalidCredentials`] if the user is not found or the
/// password does not match, or [`AppError::Db`] / [`AppError::PasswordHash`]
/// on infrastructure failures.
pub(crate) async fn verify_credentials(
    db: &sqlx::SqlitePool,
    username: &str,
    password: &str,
) -> Result<db::users::Row, AppError> {
    let user = db::users::GetByUsername { username }
        .execute(db)
        .await?
        .ok_or(AppError::InvalidCredentials)?;

    let verified = verify_password(password, &user.password_hash)?;

    if verified {
        Ok(user)
    } else {
        Err(AppError::InvalidCredentials)
    }
}

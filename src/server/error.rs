//! Unified error type for route handlers — maps domain errors to HTTP responses.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

use super::routes::{ErrorResponse, ResponseFormat};

/// Application-wide error type — each variant maps to an HTTP status code.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("invalid JSON: {0}")]
    InvalidJson(#[from] serde_json::Error),

    #[error("invalid form data: {0}")]
    InvalidForm(#[from] serde_urlencoded::de::Error),

    #[error("{0}")]
    MissingField(&'static str),

    #[error(transparent)]
    InvalidGenericCsv(#[from] crate::integrations::generic_csv::ImportError),

    #[error("failed to read request body: {0}")]
    BodyRead(#[from] axum::Error),

    #[error("invalid credentials")]
    InvalidCredentials,

    #[error("{0}")]
    Forbidden(&'static str),

    #[error("username already exists")]
    UsernameExists,

    #[error("{0}")]
    Conflict(&'static str),

    #[error("password hashing failed: {0}")]
    PasswordHash(argon2::password_hash::Error),

    #[error(transparent)]
    Crypto(#[from] crate::auth::CryptoError),

    #[error(transparent)]
    Sync(#[from] crate::worker::SyncError),

    #[error(transparent)]
    TripItAuth(#[from] crate::integrations::tripit::AuthError),

    #[error(transparent)]
    Db(#[from] sqlx::Error),
}

impl AppError {
    /// Return the HTTP status code for this error variant.
    #[must_use]
    pub const fn status(&self) -> StatusCode {
        match self {
            Self::InvalidJson(_)
            | Self::InvalidForm(_)
            | Self::MissingField(_)
            | Self::InvalidGenericCsv(_)
            | Self::BodyRead(_) => StatusCode::BAD_REQUEST,
            Self::InvalidCredentials => StatusCode::UNAUTHORIZED,
            Self::Forbidden(_) => StatusCode::FORBIDDEN,
            Self::UsernameExists | Self::Conflict(_) => StatusCode::CONFLICT,
            Self::PasswordHash(_)
            | Self::Crypto(_)
            | Self::Sync(_)
            | Self::TripItAuth(_)
            | Self::Db(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// Convert this error into an HTTP response in the given format.
    #[must_use]
    pub fn into_format_response(self, format: ResponseFormat) -> Response {
        let status = self.status();
        let msg = match &self {
            Self::Db(_)
            | Self::PasswordHash(_)
            | Self::Crypto(_)
            | Self::Sync(_)
            | Self::TripItAuth(_) => {
                tracing::error!(error = %self, "internal error");
                "internal server error".to_owned()
            }
            _ => self.to_string(),
        };
        ErrorResponse::into_format_response(msg, format, status)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        self.into_format_response(ResponseFormat::Json)
    }
}

impl From<argon2::password_hash::Error> for AppError {
    fn from(err: argon2::password_hash::Error) -> Self {
        Self::PasswordHash(err)
    }
}

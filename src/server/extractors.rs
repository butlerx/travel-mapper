//! Axum extractors for request authentication and body parsing.

/// Bearer-token and session-cookie authentication.
mod auth_user;
/// Content-type-aware JSON / form body parsing.
mod form_or_json;

pub use auth_user::AuthUser;
pub use form_or_json::FormOrJson;

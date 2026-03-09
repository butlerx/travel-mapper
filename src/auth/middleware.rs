use crate::{db, routes::unauthorized_page, server::AppState};
use aide::OperationInput;
use axum::{
    extract::FromRequestParts,
    http::{StatusCode, header, request::Parts},
    response::{IntoResponse, Response},
};
use axum_extra::extract::CookieJar;
use serde_json::json;
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Copy)]
pub struct AuthUser {
    pub user_id: i64,
}

impl OperationInput for AuthUser {}

fn sha256_hex(value: &str) -> String {
    let digest = Sha256::digest(value.as_bytes());
    digest
        .iter()
        .fold(String::with_capacity(digest.len() * 2), |mut acc, byte| {
            use std::fmt::Write;
            let _ = write!(acc, "{byte:02x}");
            acc
        })
}

fn unauthorized_response(parts: &Parts) -> Response {
    let wants_html = parts
        .headers
        .get(header::ACCEPT)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.contains("text/html"));

    if wants_html {
        unauthorized_page()
    } else {
        (
            StatusCode::UNAUTHORIZED,
            axum::Json(json!({ "error": "unauthorized" })),
        )
            .into_response()
    }
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        if let Some(auth_header) = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "))
        {
            let key_hash = sha256_hex(auth_header);
            if let Ok(Some(user_id)) = db::get_user_id_by_api_key_hash(&state.db, &key_hash).await {
                return Ok(Self { user_id });
            }
            return Err(unauthorized_response(parts));
        }

        let jar = CookieJar::from_headers(&parts.headers);
        if let Some(cookie) = jar.get("session_id")
            && let Ok(Some(session)) = db::get_session(&state.db, cookie.value()).await
        {
            let now = sqlx::query_scalar::<_, Option<String>>("SELECT datetime('now')")
                .fetch_one(&state.db)
                .await
                .ok()
                .flatten();

            if let Some(now_value) = now
                && session.expires_at > now_value
            {
                return Ok(Self {
                    user_id: session.user_id,
                });
            }
        }

        Err(unauthorized_response(parts))
    }
}

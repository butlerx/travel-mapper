use crate::{
    auth::crypto::{CryptoError, encrypt_token},
    db,
    server::{AppState, middleware::AuthUser, routes::ErrorResponse},
};
use aide::transform::TransformOperation;
use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;

use super::StatusResponse;

#[derive(Deserialize, JsonSchema)]
pub struct TripItCredentialsRequest {
    pub access_token: String,
    pub access_token_secret: String,
}

/// Store `TripIt` OAuth access tokens (encrypted at rest).
pub async fn store_tripit_credentials_handler(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<TripItCredentialsRequest>,
) -> Response {
    let token_encrypted = encrypt_token(&body.access_token, &state.encryption_key);
    let secret_encrypted = encrypt_token(&body.access_token_secret, &state.encryption_key);

    let (access_token_enc, nonce_token, access_token_secret_enc, nonce_secret) =
        match (token_encrypted, secret_encrypted) {
            (Ok((token_ct, token_nonce)), Ok((secret_ct, secret_nonce))) => {
                (token_ct, token_nonce, secret_ct, secret_nonce)
            }
            (Err(err), _) | (_, Err(err)) => {
                let message = match err {
                    CryptoError::Encrypt => "encryption failed".to_string(),
                    other => format!("failed to encrypt credentials: {other}"),
                };
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": message })),
                )
                    .into_response();
            }
        };

    match (db::credentials::Upsert {
        user_id: auth.user_id,
        access_token_enc: &access_token_enc,
        access_token_secret_enc: &access_token_secret_enc,
        nonce_token: &nonce_token,
        nonce_secret: &nonce_secret,
    })
    .execute(&state.db)
    .await
    {
        Ok(()) => (StatusCode::OK, Json(json!({ "status": "ok" }))).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("failed to store credentials: {err}") })),
        )
            .into_response(),
    }
}

pub fn store_tripit_credentials_handler_docs(op: TransformOperation) -> TransformOperation {
    op.description("Store TripIt OAuth access tokens (encrypted at rest).")
        .response::<200, Json<StatusResponse>>()
        .response::<500, Json<ErrorResponse>>()
        .tag("tripit")
}

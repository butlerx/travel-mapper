use super::{
    ErrorResponse, MultiFormatResponse, StatusResponse, multi_format_docs, negotiate_format,
};
use crate::{
    auth::{CryptoError, encrypt_token},
    db,
    server::{AppState, middleware::AuthUser},
};
use aide::transform::TransformOperation;
use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::Response,
};
use schemars::JsonSchema;
use serde::Deserialize;

/// `TripIt` OAuth credentials to store.
#[derive(Deserialize, JsonSchema)]
pub struct TripItCredentialsRequest {
    /// OAuth access token obtained from `TripIt`.
    pub access_token: String,
    /// OAuth access token secret obtained from `TripIt`.
    pub access_token_secret: String,
}

/// Store `TripIt` OAuth access tokens (encrypted at rest).
pub async fn store_tripit_credentials_handler(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
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
                let format = negotiate_format(&headers);
                return ErrorResponse::into_format_response(
                    message,
                    format,
                    StatusCode::INTERNAL_SERVER_ERROR,
                );
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
        Ok(()) => {
            let format = negotiate_format(&headers);
            let response = StatusResponse {
                status: "ok".to_string(),
            };
            StatusResponse::single_format_response(&response, format, StatusCode::OK)
        }
        Err(err) => {
            let format = negotiate_format(&headers);
            ErrorResponse::into_format_response(
                format!("failed to store credentials: {err}"),
                format,
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        }
    }
}

pub fn store_tripit_credentials_handler_docs(op: TransformOperation) -> TransformOperation {
    multi_format_docs!(
        op.description("Store TripIt OAuth access tokens (encrypted at rest)."),
        200 => StatusResponse,
        401 | 500 => ErrorResponse,
    )
    .tag("tripit")
}

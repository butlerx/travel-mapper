use crate::{
    auth::crypto::{decrypt_token, encrypt_token},
    db,
    server::{AppState, middleware::AuthUser, routes::ErrorResponse},
    tripit::TripItConsumer,
};
use aide::transform::TransformOperation;
use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize, JsonSchema)]
pub struct TripItCallbackQuery {
    pub oauth_token: String,
}

/// `TripIt` OAuth callback — exchanges request token for access token.
pub async fn tripit_callback_handler(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(query): Query<TripItCallbackQuery>,
) -> Response {
    let stored = match (db::oauth_tokens::Get {
        token: &query.oauth_token,
    })
    .execute(&state.db)
    .await
    {
        Ok(Some(row)) => row,
        Ok(None) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "unknown or expired oauth_token" })),
            )
                .into_response();
        }
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("failed to lookup request token: {err}") })),
            )
                .into_response();
        }
    };

    if stored.user_id != auth.user_id {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "request token belongs to another user" })),
        )
            .into_response();
    }

    let token_secret = match decrypt_token(
        &stored.token_secret_enc,
        &stored.nonce,
        &state.encryption_key,
    ) {
        Ok(secret) => secret,
        Err(err) => {
            tracing::error!("decrypt request token secret: {err}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "failed to decrypt request token secret" })),
            )
                .into_response();
        }
    };

    let _ = (db::oauth_tokens::Delete {
        token: &query.oauth_token,
    })
    .execute(&state.db)
    .await;

    let consumer = TripItConsumer::new(
        state.tripit_consumer_key.clone(),
        state.tripit_consumer_secret.clone(),
    );

    let request_pair = crate::tripit::OAuthTokenPair {
        token: query.oauth_token.clone(),
        token_secret,
    };

    let client = reqwest::Client::new();
    let access_token = match consumer.access_token(&client, &request_pair).await {
        Ok(pair) => pair,
        Err(err) => {
            tracing::error!("TripIt access_token exchange failed: {err}");
            return Redirect::to("/settings?error=TripIt+authorization+failed").into_response();
        }
    };

    let token_encrypted = encrypt_token(&access_token.token, &state.encryption_key);
    let secret_encrypted = encrypt_token(&access_token.token_secret, &state.encryption_key);

    let (access_token_enc, nonce_token, access_token_secret_enc, nonce_secret) =
        match (token_encrypted, secret_encrypted) {
            (Ok((token_ct, token_nonce)), Ok((secret_ct, secret_nonce))) => {
                (token_ct, token_nonce, secret_ct, secret_nonce)
            }
            (Err(err), _) | (_, Err(err)) => {
                tracing::error!("encrypt access token: {err}");
                return Redirect::to("/settings?error=Failed+to+store+credentials").into_response();
            }
        };

    if let Err(err) = (db::credentials::Upsert {
        user_id: auth.user_id,
        access_token_enc: &access_token_enc,
        access_token_secret_enc: &access_token_secret_enc,
        nonce_token: &nonce_token,
        nonce_secret: &nonce_secret,
    })
    .execute(&state.db)
    .await
    {
        tracing::error!("upsert tripit credentials: {err}");
        return Redirect::to("/settings?error=Failed+to+store+credentials").into_response();
    }

    Redirect::to("/settings?tripit=connected").into_response()
}

pub fn tripit_callback_handler_docs(op: TransformOperation) -> TransformOperation {
    op.description("TripIt OAuth callback — exchanges request token for access token.")
        .response::<302, ()>()
        .response::<400, Json<ErrorResponse>>()
        .response::<403, Json<ErrorResponse>>()
        .response::<500, Json<ErrorResponse>>()
        .tag("tripit")
}

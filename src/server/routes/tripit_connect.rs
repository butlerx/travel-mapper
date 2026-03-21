use crate::{
    auth::crypto::encrypt_token,
    db,
    server::{AppState, middleware::AuthUser, routes::ErrorResponse},
    tripit::TripItConsumer,
};
use aide::transform::TransformOperation;
use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::Host;
use serde_json::json;

/// Start `TripIt` OAuth flow — redirects to `TripIt` authorization page.
pub async fn tripit_connect_handler(
    State(state): State<AppState>,
    Host(host): Host,
    auth: AuthUser,
) -> Response {
    let consumer = TripItConsumer::new(
        state.tripit_consumer_key.clone(),
        state.tripit_consumer_secret.clone(),
    );

    let client = reqwest::Client::new();
    let request_token = match consumer.request_token(&client).await {
        Ok(pair) => pair,
        Err(err) => {
            tracing::error!("TripIt request_token failed: {err}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("failed to obtain request token: {err}") })),
            )
                .into_response();
        }
    };

    let (secret_enc, nonce) =
        match encrypt_token(&request_token.token_secret, &state.encryption_key) {
            Ok(pair) => pair,
            Err(err) => {
                tracing::error!("encrypt request token secret: {err}");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "encryption failed" })),
                )
                    .into_response();
            }
        };

    if let Err(err) = (db::oauth_tokens::Create {
        token: &request_token.token,
        token_secret_enc: &secret_enc,
        nonce: &nonce,
        user_id: auth.user_id,
    })
    .execute(&state.db)
    .await
    {
        tracing::error!("store oauth request token: {err}");
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("failed to store request token: {err}") })),
        )
            .into_response();
    }

    let scheme = if host.contains("localhost") || host.contains("127.0.0.1") {
        "http"
    } else {
        "https"
    };
    let callback_url = format!("{scheme}://{host}/auth/tripit/callback");
    let authorize_url = TripItConsumer::authorize_url(&request_token.token, &callback_url);

    Redirect::temporary(&authorize_url).into_response()
}

pub fn tripit_connect_handler_docs(op: TransformOperation) -> TransformOperation {
    op.description("Start TripIt OAuth flow — redirects to TripIt authorization page.")
        .response::<302, ()>()
        .response::<500, Json<ErrorResponse>>()
        .tag("tripit")
}

use super::{ErrorResponse, multi_format_docs, negotiate_format};
use crate::{
    auth::{decrypt_token, encrypt_token},
    db,
    integrations::tripit::TripItConsumer,
    server::{AppState, middleware::AuthUser},
};
use aide::transform::TransformOperation;
use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Redirect, Response},
};
use indexmap::IndexMap;
use schemars::JsonSchema;
use serde::Deserialize;

/// Query parameters for the `TripIt` OAuth callback.
#[derive(Deserialize, JsonSchema)]
pub struct TripItCallbackQuery {
    /// OAuth request token returned by `TripIt` after user authorization.
    pub oauth_token: String,
}

/// `TripIt` OAuth callback — exchanges request token for access token.
pub async fn tripit_callback_handler(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
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
            let format = negotiate_format(&headers);
            return ErrorResponse::into_format_response(
                "unknown or expired oauth_token",
                format,
                StatusCode::BAD_REQUEST,
            );
        }
        Err(err) => {
            let format = negotiate_format(&headers);
            return ErrorResponse::into_format_response(
                format!("failed to lookup request token: {err}"),
                format,
                StatusCode::INTERNAL_SERVER_ERROR,
            );
        }
    };

    if stored.user_id != auth.user_id {
        let format = negotiate_format(&headers);
        return ErrorResponse::into_format_response(
            "request token belongs to another user",
            format,
            StatusCode::FORBIDDEN,
        );
    }

    let token_secret = match decrypt_token(
        &stored.token_secret_enc,
        &stored.nonce,
        &state.encryption_key,
    ) {
        Ok(secret) => secret,
        Err(err) => {
            tracing::error!(error = %err, "decrypt request token secret");
            let format = negotiate_format(&headers);
            return ErrorResponse::into_format_response(
                "failed to decrypt request token secret",
                format,
                StatusCode::INTERNAL_SERVER_ERROR,
            );
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

    let request_pair = crate::integrations::tripit::OAuthTokenPair {
        token: query.oauth_token.clone(),
        token_secret,
    };

    let client = reqwest::Client::new();
    let access_token = match consumer.access_token(&client, &request_pair).await {
        Ok(pair) => pair,
        Err(err) => {
            tracing::error!(error = %err, "TripIt access_token exchange failed");
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
                tracing::error!(error = %err, "encrypt access token");
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
        tracing::error!(error = %err, "upsert tripit credentials");
        return Redirect::to("/settings?error=Failed+to+store+credentials").into_response();
    }

    Redirect::to("/settings?tripit=connected").into_response()
}

pub fn tripit_callback_handler_docs(op: TransformOperation) -> TransformOperation {
    multi_format_docs!(
        op.description("TripIt OAuth callback — exchanges request token for access token.")
            .response_with::<302, (), _>(|mut res| {
                let response = res.inner();
                response.description =
                    "Redirect to settings page after storing credentials.".to_string();
                response.headers.insert(
                    "Location".to_string(),
                    aide::openapi::ReferenceOr::Item(aide::openapi::Header {
                        description: Some(
                            "Settings page URL, possibly with a status query parameter."
                                .to_string(),
                        ),
                        style: aide::openapi::HeaderStyle::Simple,
                        required: false,
                        deprecated: None,
                        format: aide::openapi::ParameterSchemaOrContent::Schema(
                            aide::openapi::SchemaObject {
                                json_schema: schemars::Schema::from(serde_json::Map::from_iter(
                                    [(
                                        "type".to_owned(),
                                        serde_json::Value::String("string".to_owned()),
                                    )],
                                )),
                                external_docs: None,
                                example: None,
                            },
                        ),
                        example: None,
                        examples: IndexMap::default(),
                        extensions: IndexMap::default(),
                    }),
                );
                res
            }),
        400 | 401 | 403 | 500 => ErrorResponse,
    )
    .tag("tripit")
}

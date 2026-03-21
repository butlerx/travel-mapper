use super::{ErrorResponse, multi_format_docs, negotiate_format};
use crate::{
    auth::encrypt_token,
    db,
    server::{AppState, middleware::AuthUser},
    tripit::TripItConsumer,
};
use aide::transform::TransformOperation;
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::Host;
use indexmap::IndexMap;

/// Start `TripIt` OAuth flow — redirects to `TripIt` authorization page.
pub async fn tripit_connect_handler(
    State(state): State<AppState>,
    Host(host): Host,
    auth: AuthUser,
    headers: HeaderMap,
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
            let format = negotiate_format(&headers);
            return ErrorResponse::into_format_response(
                format!("failed to obtain request token: {err}"),
                format,
                StatusCode::INTERNAL_SERVER_ERROR,
            );
        }
    };

    let (secret_enc, nonce) =
        match encrypt_token(&request_token.token_secret, &state.encryption_key) {
            Ok(pair) => pair,
            Err(err) => {
                tracing::error!("encrypt request token secret: {err}");
                let format = negotiate_format(&headers);
                return ErrorResponse::into_format_response(
                    "encryption failed",
                    format,
                    StatusCode::INTERNAL_SERVER_ERROR,
                );
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
        let format = negotiate_format(&headers);
        return ErrorResponse::into_format_response(
            format!("failed to store request token: {err}"),
            format,
            StatusCode::INTERNAL_SERVER_ERROR,
        );
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
    multi_format_docs!(
        op.description("Start TripIt OAuth flow — redirects to TripIt authorization page.")
            .response_with::<302, (), _>(|mut res| {
                let response = res.inner();
                response.description = "Redirect to TripIt authorization page.".to_string();
                response.headers.insert(
                    "Location".to_string(),
                    aide::openapi::ReferenceOr::Item(aide::openapi::Header {
                        description: Some(
                            "TripIt OAuth authorization URL the client should follow.".to_string(),
                        ),
                        style: aide::openapi::HeaderStyle::Simple,
                        required: false,
                        deprecated: None,
                        format: aide::openapi::ParameterSchemaOrContent::Schema(
                            aide::openapi::SchemaObject {
                                json_schema: schemars::Schema::from(serde_json::Map::from_iter([(
                                    "type".to_owned(),
                                    serde_json::Value::String("string".to_owned()),
                                )])),
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
        401 | 500 => ErrorResponse,
    )
    .tag("tripit")
}

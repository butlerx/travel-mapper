//! [`FormOrJson`] extractor — deserialises a request body from either JSON or
//! `application/x-www-form-urlencoded` based on the `Content-Type` header.

use crate::server::{AppState, error::AppError};
use aide::{OperationInput, generate::GenContext, openapi::Operation};
use axum::{
    body::Bytes,
    extract::{FromRequest, Request},
    http::{HeaderMap, header, request::Parts},
};
use schemars::JsonSchema;
use serde::de::DeserializeOwned;

/// Extractor that deserialises a request body from either JSON or
/// `application/x-www-form-urlencoded`, depending on the `Content-Type`
/// header.  When the body is empty the extractor falls back to
/// `T::default()`, matching the legacy `parse_payload` behaviour.
pub struct FormOrJson<T>(pub T);

impl<T> FormOrJson<T>
where
    T: DeserializeOwned + Default,
{
    pub fn parse(headers: &HeaderMap, body: &[u8]) -> Result<T, AppError> {
        if body.is_empty() {
            return Ok(T::default());
        }

        if headers
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .is_some_and(|ct| ct.starts_with("application/x-www-form-urlencoded"))
        {
            serde_urlencoded::from_bytes(body).map_err(AppError::from)
        } else {
            serde_json::from_slice(body).map_err(AppError::from)
        }
    }
}

fn is_form_content_type(parts: &Parts) -> bool {
    parts
        .headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|ct| ct.starts_with("application/x-www-form-urlencoded"))
}

impl<T> FromRequest<AppState> for FormOrJson<T>
where
    T: DeserializeOwned + Default + 'static,
{
    type Rejection = AppError;

    async fn from_request(req: Request, state: &AppState) -> Result<Self, Self::Rejection> {
        let (parts, body) = req.into_parts();
        let bytes = Bytes::from_request(Request::from_parts(parts.clone(), body), state)
            .await
            .map_err(|err| AppError::BodyRead(axum::Error::new(err)))?;

        if bytes.is_empty() {
            return Ok(Self(T::default()));
        }

        if is_form_content_type(&parts) {
            serde_urlencoded::from_bytes(&bytes)
                .map(Self)
                .map_err(AppError::from)
        } else {
            serde_json::from_slice(&bytes)
                .map(Self)
                .map_err(AppError::from)
        }
    }
}

impl<T: JsonSchema> OperationInput for FormOrJson<T> {
    fn operation_input(ctx: &mut GenContext, operation: &mut Operation) {
        <axum::Json<T> as OperationInput>::operation_input(ctx, operation);
        if let Some(aide::openapi::ReferenceOr::Item(body)) = &mut operation.request_body
            && let Some(json_media) = body.content.get("application/json").cloned()
        {
            body.content
                .insert("application/x-www-form-urlencoded".to_string(), json_media);
        }
    }
}

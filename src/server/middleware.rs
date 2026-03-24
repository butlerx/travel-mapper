//! Tower middleware for HTTP request tracing and diagnostics.

use axum::{body::Body, extract::MatchedPath, http::Request};
use std::time::Duration;

/// Build the tracing span for each incoming HTTP request.
pub fn request_span(request: &Request<Body>) -> tracing::Span {
    let path = request.extensions().get::<MatchedPath>().map_or_else(
        || request.uri().path().to_owned(),
        |m| m.as_str().to_owned(),
    );
    tracing::info_span!(
        "http",
        method = %request.method(),
        path,
    )
}

/// Log the response status and latency when a request completes.
pub fn on_response(
    response: &axum::http::Response<Body>,
    latency: Duration,
    _span: &tracing::Span,
) {
    tracing::info!(
        status = response.status().as_u16(),
        latency_ms = u64::try_from(latency.as_millis()).unwrap_or(u64::MAX),
        "response",
    );
}

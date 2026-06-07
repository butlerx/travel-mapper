//! Tower middleware for HTTP request tracing and diagnostics.

use axum::{
    body::Body,
    extract::{ConnectInfo, MatchedPath},
    http::{Request, header},
};
use std::{net::SocketAddr, time::Duration};

/// Best-effort client IP: prefer proxy-supplied `X-Forwarded-For` (first hop)
/// or `X-Real-IP` for self-hosted deployments behind a reverse proxy, falling
/// back to the direct socket address when served without a proxy.
fn client_ip(request: &Request<Body>) -> String {
    let header_str = |name| {
        request
            .headers()
            .get(name)
            .and_then(|value| value.to_str().ok())
    };

    if let Some(forwarded) = header_str("x-forwarded-for")
        && let Some(first) = forwarded.split(',').next().map(str::trim)
        && !first.is_empty()
    {
        return first.to_owned();
    }
    if let Some(real) = header_str("x-real-ip")
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        return real.to_owned();
    }
    request
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|info| info.0.ip().to_string())
        .unwrap_or_default()
}

/// Build the tracing span for each incoming HTTP request. Carries the request
/// fields; `status` and `latency_ms` are filled in by [`on_response`] once the
/// response is ready.
pub(super) fn request_span(request: &Request<Body>) -> tracing::Span {
    let path = request.extensions().get::<MatchedPath>().map_or_else(
        || request.uri().path().to_owned(),
        |matched| matched.as_str().to_owned(),
    );
    let user_agent = request
        .headers()
        .get(header::USER_AGENT)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();

    tracing::info_span!(
        "http",
        method = %request.method(),
        path,
        uri = %request.uri(),
        version = ?request.version(),
        client = %client_ip(request),
        user_agent = %user_agent,
        status = tracing::field::Empty,
        latency_ms = tracing::field::Empty,
    )
}

/// Record the response status and latency on the request span and emit the
/// single per-request log line.
pub(super) fn on_response(
    response: &axum::http::Response<Body>,
    latency: Duration,
    span: &tracing::Span,
) {
    span.record("status", response.status().as_u16());
    span.record(
        "latency_ms",
        u64::try_from(latency.as_millis()).unwrap_or(u64::MAX),
    );
    tracing::info!("response");
}

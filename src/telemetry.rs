//! Logging and tracing initialisation — configures `tracing-subscriber` with
//! environment-driven filters, pretty output in debug builds, and JSON in release.

use tracing_subscriber::prelude::*;

/// Initialise the global tracing subscriber.
///
/// In debug builds the output is human-readable (`pretty`); in release builds
/// it is machine-parseable JSON. The log level is read from the `RUST_LOG`
/// environment variable, defaulting to `info,tower_http=debug`.
pub fn init() {
    #[cfg(debug_assertions)]
    let log_layer = tracing_subscriber::fmt::layer()
        .pretty()
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE);

    #[cfg(not(debug_assertions))]
    let log_layer = tracing_subscriber::fmt::layer()
        .json()
        .set_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE);

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info,tower_http=debug")),
        )
        .with(log_layer)
        .init();
}

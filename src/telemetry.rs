//! Logging and tracing initialisation — configures `tracing-subscriber` with
//! environment-driven filters, compact single-line output in debug builds, and
//! JSON in release.

use tracing_subscriber::prelude::*;

/// Initialise the global tracing subscriber.
///
/// In debug builds the output is human-readable and compact — one line per
/// event, with the active span's fields (e.g. `http{method,path}`) inline; in
/// release builds it is machine-parseable JSON. The log level is read from the
/// `RUST_LOG` environment variable, defaulting to `info,tower_http=debug`.
///
/// Span-close events are intentionally not emitted: each HTTP request already
/// logs a single `response` line (status + latency) from the trace layer, so a
/// separate `close` line per span would just duplicate it.
pub fn init() {
    #[cfg(debug_assertions)]
    let log_layer = tracing_subscriber::fmt::layer().compact();

    #[cfg(not(debug_assertions))]
    let log_layer = tracing_subscriber::fmt::layer().json();

    tracing_subscriber::registry()
        .with(env_filter())
        .with(log_layer)
        .init();
}

/// Build the log filter from `RUST_LOG` (falling back to a sensible default),
/// then quiet the `aide` target.
///
/// `aide` emits an INFO-level span-close for every route and transform during
/// router construction, which floods startup with hundreds of useless
/// `aide::axum: close, time.busy…` lines. We pin `aide=warn` so that noise is
/// gone at the default `info` level — unless the operator explicitly set an
/// `aide` directive in `RUST_LOG`, in which case we leave their choice alone.
fn env_filter() -> tracing_subscriber::EnvFilter {
    let rust_log = std::env::var("RUST_LOG").unwrap_or_default();
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info,tower_http=debug"));
    if rust_log.contains("aide") {
        filter
    } else {
        filter.add_directive("aide=warn".parse().expect("aide=warn is a valid directive"))
    }
}

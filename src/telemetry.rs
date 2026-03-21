use tracing_subscriber::prelude::*;

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

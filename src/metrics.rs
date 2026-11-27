//! Prometheus exporter and process-level metrics setup.

use std::net::SocketAddr;
use tokio::task::JoinHandle;

/// Keep process metric samplers alive for the lifetime of the server.
#[must_use = "keep the metric samplers alive while the server is running"]
pub struct ServerMetricsTasks {
    _runtime: Option<JoinHandle<()>>,
    _process: JoinHandle<()>,
}

/// Install the Prometheus exporter and process metric samplers.
///
/// # Errors
///
/// Returns an error if the exporter cannot bind or install its recorder, or if
/// histogram buckets are invalid.
pub fn init_server_metrics(
    endpoint: SocketAddr,
) -> Result<ServerMetricsTasks, metrics_exporter_prometheus::BuildError> {
    let builder = metrics_exporter_prometheus::PrometheusBuilder::new()
        .with_http_listener(endpoint)
        .with_recommended_naming(true);
    let builder = builder.set_buckets_for_metric(
        metrics_exporter_prometheus::Matcher::Full("http.server.request.duration".to_owned()),
        &tower_telemetry::metrics::DURATION_BUCKETS,
    )?;
    let builder = builder.set_buckets_for_metric(
        metrics_exporter_prometheus::Matcher::Full("http.server.request.body.size".to_owned()),
        &tower_telemetry::metrics::BODY_SIZE_BUCKETS,
    )?;
    let builder = builder.set_buckets_for_metric(
        metrics_exporter_prometheus::Matcher::Full("http.server.response.body.size".to_owned()),
        &tower_telemetry::metrics::BODY_SIZE_BUCKETS,
    )?;
    builder.install()?;

    tower_telemetry::metrics::install_panic_counter("process.panics");
    let runtime = tower_telemetry::metrics::export_runtime_metrics();
    let process = tower_telemetry::metrics::export_process_metrics();

    Ok(ServerMetricsTasks {
        _runtime: runtime,
        _process: process,
    })
}

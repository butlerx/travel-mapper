//! Travel Mapper — sync `TripIt` travel history to `SQLite` and serve it via a web
//! dashboard, REST API, or CSV export.

#![warn(clippy::pedantic)]

/// Encryption and password-hashing helpers — AES-256-GCM and Argon2.
pub mod auth;
/// CLI subcommands — serve, worker, create-user, seed.
pub mod commands;
/// Database layer — connection pool setup, migrations, and per-table query objects.
pub mod db;
/// Distance calculation utilities — haversine great-circle distance.
pub(crate) mod distance;
/// Geocoding helpers — Nominatim lookups, IATA airport and CRS station resolution.
pub(crate) mod geocode;
/// Third-party travel data integrations — TripIt, flight status, rail status, CSV import.
pub(crate) mod integrations;
/// Axum web server — routing, pages, extractors, and middleware.
pub mod server;
/// Logging and tracing initialisation.
pub mod telemetry;
/// Background sync worker — polls for pending sync jobs and runs imports.
pub mod worker;

/// Wait for a ctrl-c signal, logging any failure to install the handler.
pub async fn shutdown_signal() {
    if let Err(error) = tokio::signal::ctrl_c().await {
        tracing::error!(error = %error, "failed to install ctrl+c handler");
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    tracing::info!("shutdown signal received");
}

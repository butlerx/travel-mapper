//! Travel Mapper — sync `TripIt` travel history to `SQLite` and serve it via a web
//! dashboard, REST API, or CSV export.

#![warn(clippy::pedantic)]

/// Encryption and password-hashing helpers — AES-256-GCM and Argon2.
pub mod auth;
/// Database layer — connection pool setup, migrations, and per-table query objects.
pub mod db;
/// Distance calculation utilities — haversine great-circle distance.
pub mod distance;
/// Geocoding helpers — Nominatim lookups, IATA airport and CRS station resolution.
pub mod geocode;
/// Third-party travel data integrations — TripIt, flight status, rail status, CSV import.
pub mod integrations;
/// Axum web server — routing, pages, extractors, and middleware.
pub mod server;
/// Logging and tracing initialisation.
pub mod telemetry;
/// Background sync worker — polls for pending sync jobs and runs imports.
pub mod worker;

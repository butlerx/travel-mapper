//! Travel Mapper — sync `TripIt` travel history to `SQLite` and serve it via a web
//! dashboard, REST API, or CSV export.

#![warn(clippy::pedantic)]

pub mod auth;
pub mod db;
/// Distance calculation utilities — haversine great-circle distance.
pub mod distance;
pub mod geocode;
pub mod integrations;
pub mod server;
pub mod telemetry;
pub mod worker;

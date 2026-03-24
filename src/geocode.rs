//! Geocoding helpers — Nominatim forward/reverse geocoding and IATA airport
//! lookups, with multi-strategy fallback for noisy location strings.

/// IATA airport code lookup from an embedded dataset.
pub mod airports;
/// Nominatim geocoder via the [`nominatim`] crate and `Geocoder` struct.
mod nominatim;
/// Trip coordinate resolution and timezone sanity checks.
mod resolve;
/// String-cleaning utilities for location names and timezone mapping.
mod sanitize;

pub use nominatim::Geocoder;

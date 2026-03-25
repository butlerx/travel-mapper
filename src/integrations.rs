//! External service integrations: generic CSV/delimited import, `TripIt` API
//! client, and flight status enrichment.

/// `AviationStack` flight status API client.
pub mod flight_status;
/// Generic CSV/delimited import — auto-detects Flighty, myFlightradar24,
/// OpenFlights, and App in the Air formats.
pub mod generic_csv;
pub mod tripit;

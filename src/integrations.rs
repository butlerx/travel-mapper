//! External service integrations: generic CSV/delimited import and `TripIt` API
//! client.

/// Generic CSV/delimited import — auto-detects Flighty, myFlightradar24,
/// OpenFlights, and App in the Air formats.
pub mod generic_csv;
pub mod tripit;

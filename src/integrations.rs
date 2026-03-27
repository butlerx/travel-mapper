//! External service integrations: generic CSV/delimited import, `TripIt` API
//! client, and flight status enrichment.

/// AirLabs flight status API client.
pub mod airlabs;
/// National Rail Darwin `OpenLDBWS` SOAP client for UK rail status.
pub mod darwin;
/// Deutsche Bahn RIS Journeys API client for German rail status.
pub mod db_ris;
/// Flight status API trait and shared types.
pub mod flight_status;
/// Generic CSV/delimited import — auto-detects Flighty, myFlightradar24,
/// OpenFlights, and App in the Air formats.
pub mod generic_csv;
/// OpenSky Network API client for route verification via ADS-B data.
pub mod opensky;
/// Rail status API trait and shared types.
pub mod rail_status;
/// Transitland GTFS-RT API client for rail status enrichment.
pub mod transitland;
/// `TripIt` API client and OAuth 1.0a integration.
pub mod tripit;

//! External service integrations: generic CSV/delimited import, `TripIt` API
//! client, and flight status enrichment.

/// AirLabs flight status API client.
pub(crate) mod airlabs;
/// National Rail Darwin `OpenLDBWS` SOAP client for UK rail status.
pub(crate) mod darwin;
/// Deutsche Bahn RIS Journeys API client for German rail status.
pub(crate) mod db_ris;
/// Flight status API trait and shared types.
pub(crate) mod flight_status;
/// Generic CSV/delimited import — auto-detects Flighty, myFlightradar24,
/// OpenFlights, and App in the Air formats.
pub(crate) mod generic_csv;
/// OpenSky Network API client for route verification via ADS-B data.
pub(crate) mod opensky;
/// Rail status API trait and shared types.
pub(crate) mod rail_status;
/// Transitland GTFS-RT API client for rail status enrichment.
pub(crate) mod transitland;
/// `TripIt` API client and OAuth 1.0a integration.
pub(crate) mod tripit;

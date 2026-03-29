//! `TripIt` API integration: OAuth signing and API client.

mod auth;
mod fetch;

pub use auth::{AuthError, OAuthTokenPair, TripItAuth, TripItConsumer};
pub use fetch::{FetchError, Trip, TripItApi, TripItClient, fetch_trips};

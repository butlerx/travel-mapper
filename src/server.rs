//! Application state, router setup, and request handling.

pub(crate) mod components;
pub(crate) mod error;
pub(crate) mod middleware;
pub(crate) mod pages;
pub(crate) mod routes;
pub(crate) mod session;
mod state;
#[cfg(test)]
pub(crate) mod test_helpers;

pub use state::{AppState, create_router};

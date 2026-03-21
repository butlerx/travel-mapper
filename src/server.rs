//! Application state, router setup, and request handling.

pub mod components;
pub mod middleware;
pub mod pages;
pub mod routes;
pub mod session;
mod state;
#[cfg(test)]
pub(crate) mod test_helpers;

pub use state::{AppState, create_router};

//! HTTP route handlers and response formatting.

pub mod handlers;
pub mod pages;
mod response;
mod static_assets;

pub use handlers::{
    ErrorResponse, HealthResponse, HopQuery, SyncQueuedResponse, health_handler,
    health_handler_docs, hops_handler, hops_handler_docs, sync_handler, sync_handler_docs,
};
pub use pages::{
    dashboard_page, landing_page, login_page, not_found_page, register_page, settings_page,
    unauthorized_page,
};
pub use static_assets::{serve_css, serve_js};

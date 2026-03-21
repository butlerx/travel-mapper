//! HTTP route handlers and response formatting.

pub mod api_keys;
pub mod health;
pub mod hops;
pub mod login;
pub mod logout;
pub mod register;
mod static_assets;
pub mod sync;
pub mod tripit_callback;
pub mod tripit_connect;
pub mod tripit_credentials;
pub mod types;

pub use api_keys::{
    ApiKeyRequest, ApiKeyResponse, create_api_key_handler, create_api_key_handler_docs,
};
pub use health::{HealthResponse, health_handler, health_handler_docs};
pub use hops::{HopQuery, HopResponse, HopTravelType, hops_handler, hops_handler_docs};
pub use login::{AuthResponse, LoginRequest, login_handler, login_handler_docs};
pub use logout::{logout_handler, logout_handler_docs};
pub use register::{RegisterRequest, register_handler, register_handler_docs};
pub use static_assets::{serve_css, serve_js};
pub use sync::{sync_handler, sync_handler_docs};
pub use tripit_callback::{
    TripItCallbackQuery, tripit_callback_handler, tripit_callback_handler_docs,
};
pub use tripit_connect::{tripit_connect_handler, tripit_connect_handler_docs};
pub use tripit_credentials::{
    TripItCredentialsRequest, store_tripit_credentials_handler,
    store_tripit_credentials_handler_docs,
};
pub use types::{ErrorResponse, StatusResponse};

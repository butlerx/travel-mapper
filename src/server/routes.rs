//! HTTP route handlers and response formatting.

mod api_keys;
mod flighty;
mod health;
mod hops;
mod login;
mod logout;
mod register;
mod static_assets;
mod sync;
mod tripit_callback;
mod tripit_connect;
mod tripit_credentials;
mod types;

pub(super) use api_keys::{create_api_key_handler, create_api_key_handler_docs};
pub(super) use flighty::import_flighty_handler;
pub(super) use health::{health_handler, health_handler_docs};
pub(super) use hops::{HopResponse, hops_handler, hops_handler_docs};
pub(super) use login::{AuthResponse, login_handler, login_handler_docs};
pub(super) use logout::{logout_handler, logout_handler_docs};
pub(super) use register::{register_handler, register_handler_docs};
pub(super) use static_assets::{serve_css, serve_js};
pub(super) use sync::{sync_handler, sync_handler_docs};
pub(super) use tripit_callback::{tripit_callback_handler, tripit_callback_handler_docs};
pub(super) use tripit_connect::{tripit_connect_handler, tripit_connect_handler_docs};
pub(super) use tripit_credentials::{
    store_tripit_credentials_handler, store_tripit_credentials_handler_docs,
};
pub(super) use types::{
    ErrorResponse, MultiFormatResponse, ResponseFormat, StatusResponse, multi_format_docs,
    negotiate_format,
};

// Leptos `#[component]` macro generates `#[must_use]` automatically,
// and component props must own their data (String, not &str).
#![allow(clippy::must_use_candidate, clippy::needless_pass_by_value)]

mod auth_page;
mod dashboard_page;
mod error_page;
mod landing_page;
mod navbar;
mod settings_page;
mod shell;

pub use auth_page::AuthFormPage;
pub use dashboard_page::DashboardPage;
pub use error_page::ErrorPage;
pub use landing_page::LandingPage;
pub use settings_page::SettingsPage;

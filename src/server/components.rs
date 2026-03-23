// Leptos `#[component]` macro generates `#[must_use]` automatically,
// and component props must own their data (String, not &str).
#![allow(clippy::must_use_candidate, clippy::needless_pass_by_value)]

mod add_flight_page;
mod auth_page;
mod dashboard_page;
mod error_page;
mod hop_detail_page;
mod landing_page;
mod navbar;
mod settings_page;
mod shell;
mod stats_page;

pub(crate) use add_flight_page::AddFlightPage;
pub(crate) use auth_page::AuthFormPage;
pub(crate) use dashboard_page::DashboardPage;
pub(crate) use error_page::ErrorPage;
pub(crate) use hop_detail_page::HopDetailPage;
pub(crate) use landing_page::LandingPage;
pub(crate) use navbar::NavBar;
pub(crate) use settings_page::SettingsPage;
pub(crate) use shell::Shell;
pub(crate) use stats_page::StatsPage;

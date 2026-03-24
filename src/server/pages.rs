//! Page handlers for HTML views.
// Leptos `#[component]` macro generates `#[must_use]` automatically,
// and component props must own their data (String, not &str).
#![allow(clippy::must_use_candidate, clippy::needless_pass_by_value)]

pub(super) mod add_journey;
pub(crate) mod dashboard;
pub(crate) mod journey_detail;
pub(super) mod landing;
pub(super) mod login;
pub(super) mod not_found;
pub(super) mod register;
pub(super) mod settings;
pub(crate) mod stats;
pub(crate) mod trip_detail;
pub(crate) mod trips;
pub(super) mod unauthorized;

use super::{AppState, components::ErrorPage};
use crate::db;
use aide::axum::ApiRouter;
use axum::routing::get;
use axum_extra::extract::CookieJar;
use leptos::prelude::*;
use serde::Deserialize;

/// Page and static-asset routes that do not need `OpenAPI` metadata.
pub(super) fn page_routes() -> ApiRouter<AppState> {
    ApiRouter::new()
        .route("/", get(landing::page))
        .route("/register", get(register::page))
        .route("/login", get(login::page))
        .route("/dashboard", get(dashboard::page))
        .route("/settings", get(settings::page))
        .route("/stats", get(stats::page))
        .route("/journeys/new", get(add_journey::page))
}

/// Check whether the request carries a valid, non-expired session cookie.
async fn has_valid_session(jar: &CookieJar, state: &AppState) -> bool {
    let Some(cookie) = jar.get("session_id") else {
        return false;
    };
    let Ok(Some(session)) = (db::sessions::Get {
        token: cookie.value(),
    })
    .execute(&state.db)
    .await
    else {
        return false;
    };
    let Ok(Some(now)) = sqlx::query_scalar::<_, Option<String>>("SELECT datetime('now')")
        .fetch_one(&state.db)
        .await
    else {
        return false;
    };
    session.expires_at > now
}

#[derive(Deserialize, Default, schemars::JsonSchema)]
pub(crate) struct FormFeedback {
    pub(crate) error: Option<String>,
    pub(crate) success: Option<String>,
}

fn render_error_page(
    code: &'static str,
    title: &'static str,
    message: &'static str,
    link_href: &'static str,
    link_text: &'static str,
) -> String {
    let html = view! {
        <ErrorPage code=code title=title message=message link_href=link_href link_text=link_text />
    };
    html.to_html()
}

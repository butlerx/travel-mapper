//! Page handlers for HTML views.

mod dashboard;
mod landing;
mod login;
mod not_found;
mod register;
mod settings;
mod unauthorized;

pub(super) use dashboard::dashboard_page;
pub(super) use landing::landing_page;
pub(super) use login::login_page;
pub(super) use not_found::not_found_page;
pub(super) use register::register_page;
pub(super) use settings::settings_page;
pub(super) use unauthorized::unauthorized_page;

use super::{AppState, components::ErrorPage};
use crate::db;
use axum_extra::extract::CookieJar;
use leptos::prelude::*;
use serde::Deserialize;

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

#[derive(Deserialize, Default)]
pub(super) struct FormFeedback {
    pub(super) error: Option<String>,
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

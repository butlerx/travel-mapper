//! Page handlers for HTML views.

mod dashboard;
mod landing;
mod login;
mod not_found;
mod register;
mod settings;
mod unauthorized;

pub use dashboard::dashboard_page;
pub use landing::landing_page;
pub use login::login_page;
pub use not_found::not_found_page;
pub use register::register_page;
pub use settings::settings_page;
pub use unauthorized::unauthorized_page;

use crate::{db, server::AppState};
use axum_extra::extract::CookieJar;
use leptos::prelude::*;
use serde::Deserialize;

use crate::server::components::ErrorPage;

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
pub struct FormFeedback {
    pub error: Option<String>,
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

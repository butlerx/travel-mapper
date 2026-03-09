use crate::{
    app::{AuthFormPage, DashboardPage, ErrorPage, LandingPage, SettingsPage},
    auth::AuthUser,
    db,
    server::AppState,
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::CookieJar;
use leptos::prelude::*;
use serde::Deserialize;

/// Check whether the request carries a valid, non-expired session cookie.
async fn has_valid_session(jar: &CookieJar, state: &AppState) -> bool {
    let Some(cookie) = jar.get("session_id") else {
        return false;
    };
    let Ok(Some(session)) = db::get_session(&state.db, cookie.value()).await else {
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

pub async fn landing_page(State(state): State<AppState>, jar: CookieJar) -> Response {
    if has_valid_session(&jar, &state).await {
        return Redirect::to("/dashboard").into_response();
    }

    let html = view! { <LandingPage /> };
    (StatusCode::OK, axum::response::Html(html.to_html())).into_response()
}

#[derive(Deserialize, Default)]
pub struct FormFeedback {
    pub error: Option<String>,
}

pub async fn register_page(
    State(state): State<AppState>,
    jar: CookieJar,
    Query(feedback): Query<FormFeedback>,
) -> Response {
    if has_valid_session(&jar, &state).await {
        return Redirect::to("/dashboard").into_response();
    }

    let html = view! {
        <AuthFormPage
            title="Register"
            action="/auth/register"
            submit_label="Create Account"
            footer_text="Already have an account? "
            footer_link_href="/login"
            footer_link_text="Log in"
            autocomplete_password="new-password"
            error=feedback.error
        />
    };
    (StatusCode::OK, axum::response::Html(html.to_html())).into_response()
}

pub async fn login_page(
    State(state): State<AppState>,
    jar: CookieJar,
    Query(feedback): Query<FormFeedback>,
) -> Response {
    if has_valid_session(&jar, &state).await {
        return Redirect::to("/dashboard").into_response();
    }

    let html = view! {
        <AuthFormPage
            title="Log In"
            action="/auth/login"
            submit_label="Log In"
            footer_text="Don\u{2019}t have an account? "
            footer_link_href="/register"
            footer_link_text="Register"
            autocomplete_password="current-password"
            error=feedback.error
        />
    };
    (StatusCode::OK, axum::response::Html(html.to_html())).into_response()
}

#[derive(Deserialize, Default)]
pub struct DashboardFeedback {
    pub error: Option<String>,
}

pub async fn dashboard_page(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(feedback): Query<DashboardFeedback>,
) -> Response {
    let hops = db::get_all_hops(&state.db, auth.user_id, None)
        .await
        .unwrap_or_default();

    let hop_count = hops.len();
    let hops_json = serde_json::to_string(&hops).unwrap_or_default();

    let html = view! {
        <DashboardPage
            hops_json=hops_json
            hop_count=hop_count
            error=feedback.error
        />
    };
    (StatusCode::OK, axum::response::Html(html.to_html())).into_response()
}

#[derive(Deserialize, Default)]
pub struct SettingsFeedback {
    pub error: Option<String>,
    pub tripit: Option<String>,
}

pub async fn settings_page(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(feedback): Query<SettingsFeedback>,
) -> Response {
    let has_tripit = db::has_tripit_credentials(&state.db, auth.user_id)
        .await
        .unwrap_or(false);
    let sync_state = db::get_or_create_sync_state(&state.db, auth.user_id)
        .await
        .ok();

    let html = view! {
        <SettingsPage
            has_tripit=has_tripit
            sync_status=sync_state.as_ref().map(|s| s.sync_status.clone())
            last_sync_at=sync_state.as_ref().and_then(|s| s.last_sync_at.clone())
            trips_fetched=sync_state.as_ref().map(|s| s.trips_fetched)
            hops_fetched=sync_state.as_ref().map(|s| s.hops_fetched)
            error=feedback.error
            tripit_connected=feedback.tripit
        />
    };
    (StatusCode::OK, axum::response::Html(html.to_html())).into_response()
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

pub async fn not_found_page() -> Response {
    (
        StatusCode::NOT_FOUND,
        axum::response::Html(render_error_page(
            "404",
            "Page Not Found",
            "The page you\u{2019}re looking for doesn\u{2019}t exist or has been moved.",
            "/",
            "Go Home",
        )),
    )
        .into_response()
}

#[must_use]
pub fn unauthorized_page() -> Response {
    (
        StatusCode::UNAUTHORIZED,
        axum::response::Html(render_error_page(
            "401",
            "Unauthorized",
            "You need to log in to access this page.",
            "/login",
            "Log In",
        )),
    )
        .into_response()
}

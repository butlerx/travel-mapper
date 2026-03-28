/// API keys management section.
mod api_keys_section;
/// Generic CSV/delimited import section.
mod csv_import_section;
/// Email address and verification status section.
mod email_section;
/// Calendar feed subscription section.
mod feed_section;
mod profile_section;
/// Push notification subscription section.
mod push_section;
/// Shareable stats link section.
mod share_section;
/// `TripIt` connection and sync status section.
mod tripit_section;

use crate::{
    db,
    server::components::{NavBar, Shell},
};
use api_keys_section::ApiKeysSection;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use csv_import_section::CsvImportSection;
use email_section::EmailSection;
use feed_section::FeedSection;
use leptos::prelude::*;
use profile_section::ProfileSection;
use push_section::PushSection;
use serde::Deserialize;
use share_section::ShareSection;
use tripit_section::TripitSection;

/// Flash feedback messages displayed on the settings page after user actions.
#[derive(Deserialize, Default, schemars::JsonSchema)]
pub struct SettingsFeedback {
    pub error: Option<String>,
    pub tripit: Option<String>,
    pub csv: Option<String>,
    pub email: Option<String>,
    pub profile: Option<String>,
    /// Raw API key shown once after creation (not persisted).
    pub new_api_key: Option<String>,
}

/// User profile fields submitted from the settings form.
pub struct UserProfileData {
    pub email: String,
    pub email_verified: bool,
    pub first_name: String,
    pub last_name: String,
    pub vapid_public_key: Option<String>,
}

/// All data needed to render the settings page.
pub struct PageData {
    pub has_tripit: bool,
    pub sync_state: Option<db::sync_state::Row>,
    pub feedback: SettingsFeedback,
    pub profile: UserProfileData,
    pub feed_tokens: Vec<db::feed_tokens::Row>,
    pub share_tokens: Vec<db::share_tokens::Row>,
    pub api_keys: Vec<db::api_keys::Row>,
    pub base_url: String,
}

/// Render the full settings page with all sections populated from the database.
pub fn render_page(data: PageData) -> Response {
    let html = view! {
        <Settings
            has_tripit=data.has_tripit
            sync_status=data.sync_state.as_ref().map(|s| s.sync_status.clone())
            last_sync_at=data.sync_state.as_ref().and_then(|s| s.last_sync_at.clone())
            trips_fetched=data.sync_state.as_ref().map(|s| s.trips_fetched)
            hops_fetched=data.sync_state.map(|s| s.hops_fetched)
            error=data.feedback.error
            tripit_connected=data.feedback.tripit
            csv_imported=data.feedback.csv
            email_feedback=data.feedback.email
            profile_feedback=data.feedback.profile
            email=data.profile.email
            email_verified=data.profile.email_verified
            first_name=data.profile.first_name
            last_name=data.profile.last_name
            vapid_public_key=data.profile.vapid_public_key
            feed_tokens=data.feed_tokens
            share_tokens=data.share_tokens
            api_keys=data.api_keys
            new_api_key=data.feedback.new_api_key
            base_url=data.base_url
        />
    };
    (StatusCode::OK, axum::response::Html(html.to_html())).into_response()
}

#[component]
fn Settings(
    has_tripit: bool,
    sync_status: Option<String>,
    last_sync_at: Option<String>,
    trips_fetched: Option<i64>,
    hops_fetched: Option<i64>,
    #[prop(optional_no_strip)] error: Option<String>,
    #[prop(optional_no_strip)] tripit_connected: Option<String>,
    #[prop(optional_no_strip)] csv_imported: Option<String>,
    #[prop(optional_no_strip)] email_feedback: Option<String>,
    #[prop(optional_no_strip)] profile_feedback: Option<String>,
    #[prop(optional_no_strip)] email: String,
    email_verified: bool,
    first_name: String,
    last_name: String,
    #[prop(optional_no_strip)] vapid_public_key: Option<String>,
    feed_tokens: Vec<db::feed_tokens::Row>,
    share_tokens: Vec<db::share_tokens::Row>,
    api_keys: Vec<db::api_keys::Row>,
    #[prop(optional_no_strip)] new_api_key: Option<String>,
    base_url: String,
) -> impl IntoView {
    let email_alert = email_feedback
        .as_deref()
        .map(|v| match v {
            "verified" => "Email address verified successfully!",
            "updated" => "Email address updated. Please check your inbox for a verification link.",
            "sent" => "Verification email sent. Please check your inbox.",
            _ => "",
        })
        .filter(|msg| !msg.is_empty());

    let profile_alert = profile_feedback
        .as_deref()
        .map(|v| match v {
            "updated" => "Profile updated successfully!",
            _ => "",
        })
        .filter(|msg| !msg.is_empty());

    view! {
        <Shell title="Settings".to_owned()>
            <NavBar current="settings" />
            <main class="settings-page">
                {error.map(|e| view! {
                    <div class="alert alert-error" role="alert">{e}</div>
                })}
                {tripit_connected.filter(|v| v == "connected").map(|_| view! {
                    <div class="alert alert-success" role="status">"TripIt account connected successfully!"</div>
                })}
                {csv_imported.map(|count| view! {
                    <div class="alert alert-success" role="status">
                        {format!("Successfully imported {count} flights!")}
                    </div>
                })}
                {email_alert.map(|msg| view! {
                    <div class="alert alert-success" role="status">{msg}</div>
                })}
                {profile_alert.map(|msg| view! {
                    <div class="alert alert-success" role="status">{msg}</div>
                })}

                <div class="settings-group">
                    <h2 class="settings-group-heading">"Account"</h2>
                    <div class="settings-grid">
                        <ProfileSection first_name=first_name last_name=last_name />
                        <EmailSection email=email email_verified=email_verified />
                    </div>
                </div>

                <div class="settings-group">
                    <h2 class="settings-group-heading">"Data Sources"</h2>
                    <TripitSection
                        has_tripit=has_tripit
                        sync_status=sync_status
                        last_sync_at=last_sync_at
                        trips_fetched=trips_fetched
                        hops_fetched=hops_fetched
                    />
                    <CsvImportSection />
                </div>

                <div class="settings-group">
                    <h2 class="settings-group-heading">"Access & Sharing"</h2>
                    <FeedSection tokens=feed_tokens base_url=base_url.clone() />
                    <ShareSection tokens=share_tokens base_url=base_url />
                    <ApiKeysSection keys=api_keys new_key=new_api_key />
                    <PushSection vapid_public_key=vapid_public_key />
                </div>

                <script>
                    "document.querySelectorAll('[data-copy-trigger]').forEach(function(btn){"
                    "btn.addEventListener('click',function(){"
                    "var code=btn.closest('.new-token-value').querySelector('[data-copy-value]');"
                    "var text=code.getAttribute('data-copy-value');"
                    "navigator.clipboard.writeText(text).then(function(){"
                    "btn.textContent='Copied!';"
                    "setTimeout(function(){btn.textContent='Copy';},2000);"
                    "});"
                    "});"
                    "});"
                </script>
            </main>
        </Shell>
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        auth::encrypt_token,
        db,
        server::{create_router, test_helpers::*},
    };
    use axum::{
        body::Body,
        http::{Request, StatusCode, header},
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn settings_page_without_tripit_renders_sections() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/settings")
                    .header(header::COOKIE, cookie)
                    .header(header::ACCEPT, "text/html")
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_text(response).await;
        assert!(body.contains("Settings"));
        assert!(body.contains("TripIt"));
        assert!(body.contains("Not Connected"));
        assert!(body.contains("Connect TripIt"));
        assert!(body.contains("/auth/tripit/connect"));
        assert!(body.contains("API Keys"));
    }

    #[tokio::test]
    async fn settings_page_with_tripit_and_sync_state_renders_connected_and_sync_now() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let user = db::users::GetByUsername { username: "alice" }
            .execute(&pool)
            .await
            .expect("lookup failed")
            .expect("missing user");
        let key = [7_u8; 32];
        let (access_token_enc, nonce_token) =
            encrypt_token("token", &key).expect("failed to encrypt access token");
        let (access_token_secret_enc, nonce_secret) =
            encrypt_token("secret", &key).expect("failed to encrypt access token secret");

        db::credentials::Upsert {
            user_id: user.id,
            access_token_enc: &access_token_enc,
            access_token_secret_enc: &access_token_secret_enc,
            nonce_token: &nonce_token,
            nonce_secret: &nonce_secret,
        }
        .execute(&pool)
        .await
        .expect("failed to upsert tripit credentials");

        let mut sync_state = db::sync_state::GetOrCreate { user_id: user.id }
            .execute(&pool)
            .await
            .expect("failed to fetch sync state");
        sync_state.sync_status = "idle".to_string();
        sync_state.last_sync_at = Some("2026-01-02 03:04:05".to_string());
        sync_state.trips_fetched = 3;
        sync_state.hops_fetched = 12;
        db::sync_state::Update {
            user_id: user.id,
            state: &sync_state,
        }
        .execute(&pool)
        .await
        .expect("failed to update sync state");

        let app = create_router(test_app_state(pool));
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/settings")
                    .header(header::COOKIE, cookie)
                    .header(header::ACCEPT, "text/html")
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_text(response).await;
        assert!(body.contains("Connected"));
        assert!(body.contains("Sync Now"));
    }

    #[tokio::test]
    async fn settings_page_with_error_renders_alert() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/settings?error=Something+broke")
                    .header(header::COOKIE, cookie)
                    .header(header::ACCEPT, "text/html")
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_text(response).await;
        assert!(body.contains("Something broke"));
        assert!(body.contains("alert-error"));
    }

    #[tokio::test]
    async fn settings_page_tripit_connected_feedback_renders_success_alert() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/settings?tripit=connected")
                    .header(header::COOKIE, cookie)
                    .header(header::ACCEPT, "text/html")
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_text(response).await;
        assert!(body.contains("TripIt account connected successfully"));
    }

    #[tokio::test]
    async fn settings_navbar_marks_settings_link_as_current_page() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/settings")
                    .header(header::COOKIE, cookie)
                    .header(header::ACCEPT, "text/html")
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_text(response).await;
        assert!(body.contains("<nav"));
        assert!(body.contains("href=\"/settings\""));
        assert!(body.contains("aria-current=\"page\""));
    }

    #[tokio::test]
    async fn settings_page_renders_csv_import_section() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/settings")
                    .header(header::COOKIE, cookie)
                    .header(header::ACCEPT, "text/html")
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_text(response).await;
        assert!(body.contains("CSV Import"));
        assert!(body.contains("/import/csv"));
        assert!(body.contains("multipart/form-data"));
        assert!(body.contains("Import Flights"));
    }

    #[tokio::test]
    async fn settings_page_csv_imported_feedback_renders_success_alert() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/settings?csv=42")
                    .header(header::COOKIE, cookie)
                    .header(header::ACCEPT, "text/html")
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_text(response).await;
        assert!(body.contains("42 flights"));
        assert!(body.contains("alert-success"));
    }
}

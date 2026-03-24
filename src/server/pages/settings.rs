/// API keys management section.
mod api_keys_section;
/// Generic CSV/delimited import section.
mod csv_import_section;
/// Sync status and trigger section.
mod sync_section;
/// `TripIt` connection section.
mod tripit_section;

use crate::{
    db,
    server::{
        AppState,
        components::{NavBar, Shell},
        middleware::AuthUser,
    },
};
use api_keys_section::ApiKeysSection;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use csv_import_section::CsvImportSection;
use leptos::prelude::*;
use serde::Deserialize;
use sync_section::SyncSection;
use tripit_section::TripitSection;

#[derive(Deserialize, Default)]
pub struct SettingsFeedback {
    pub error: Option<String>,
    pub tripit: Option<String>,
    pub csv: Option<String>,
}

pub async fn page(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(feedback): Query<SettingsFeedback>,
) -> Response {
    let has_tripit = db::credentials::Has {
        user_id: auth.user_id,
    }
    .execute(&state.db)
    .await
    .unwrap_or(false);
    let sync_state = db::sync_state::GetOrCreate {
        user_id: auth.user_id,
    }
    .execute(&state.db)
    .await
    .ok();

    let html = view! {
        <Settings
            has_tripit=has_tripit
            sync_status=sync_state.as_ref().map(|s| s.sync_status.clone())
            last_sync_at=sync_state.as_ref().and_then(|s| s.last_sync_at.clone())
            trips_fetched=sync_state.as_ref().map(|s| s.trips_fetched)
            hops_fetched=sync_state.as_ref().map(|s| s.hops_fetched)
            error=feedback.error
            tripit_connected=feedback.tripit
            csv_imported=feedback.csv
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
) -> impl IntoView {
    view! {
        <Shell title="Settings".to_owned()>
            <NavBar current="settings" />
            <main class="container">
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

                <TripitSection has_tripit=has_tripit />

                <SyncSection
                    has_tripit=has_tripit
                    sync_status=sync_status
                    last_sync_at=last_sync_at
                    trips_fetched=trips_fetched
                    hops_fetched=hops_fetched
                />

                <CsvImportSection />

                <ApiKeysSection />
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
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_text(response).await;
        assert!(body.contains("Settings"));
        assert!(body.contains("TripIt Connection"));
        assert!(body.contains("Not Connected"));
        assert!(body.contains("Connect TripIt"));
        assert!(body.contains("/auth/tripit/connect"));
        assert!(body.contains("Sync Status"));
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

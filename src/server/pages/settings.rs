use crate::{
    db,
    server::{AppState, components::SettingsPage, middleware::AuthUser},
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use leptos::prelude::*;
use serde::Deserialize;

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

#[cfg(test)]
mod tests {
    use crate::{
        auth::encrypt_token,
        db,
        server::{create_router, test_helpers::helpers::*},
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
        let user = db::get_user_by_username(&pool, "alice")
            .await
            .expect("lookup failed")
            .expect("missing user");
        let key = [7_u8; 32];
        let (access_token_enc, nonce_token) =
            encrypt_token("token", &key).expect("failed to encrypt access token");
        let (access_token_secret_enc, nonce_secret) =
            encrypt_token("secret", &key).expect("failed to encrypt access token secret");

        db::upsert_tripit_credentials(
            &pool,
            user.id,
            &access_token_enc,
            &access_token_secret_enc,
            &nonce_token,
            &nonce_secret,
        )
        .await
        .expect("failed to upsert tripit credentials");

        let mut sync_state = db::get_or_create_sync_state(&pool, user.id)
            .await
            .expect("failed to fetch sync state");
        sync_state.sync_status = "idle".to_string();
        sync_state.last_sync_at = Some("2026-01-02 03:04:05".to_string());
        sync_state.trips_fetched = 3;
        sync_state.hops_fetched = 12;
        db::update_sync_state(&pool, user.id, &sync_state)
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
}

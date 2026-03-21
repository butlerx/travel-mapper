use crate::{
    db,
    server::{AppState, components::DashboardPage, middleware::AuthUser},
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use leptos::prelude::*;
use serde::Deserialize;

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

#[cfg(test)]
mod tests {
    use crate::{
        db,
        models::TravelType,
        server::{create_router, test_helpers::helpers::*},
    };
    use axum::{
        body::Body,
        http::{Request, StatusCode, header},
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn dashboard_without_hops_renders_empty_state_and_nav() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/dashboard")
                    .header(header::COOKIE, cookie)
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_text(response).await;
        assert!(body.contains("No hops yet"));
        assert!(body.contains("href=\"/settings\""));
        assert!(body.contains("<nav"));
        assert!(body.contains("Dashboard"));
        assert!(body.contains("Settings"));
        assert!(body.contains("action=\"/auth/logout\""));
    }

    #[tokio::test]
    async fn dashboard_with_hops_renders_map_controls_and_script() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let user = db::get_user_by_username(&pool, "alice")
            .await
            .expect("lookup failed")
            .expect("missing user");
        db::insert_hops(
            &pool,
            "trip-1",
            user.id,
            &[sample_hop(
                TravelType::Air,
                "LHR",
                "JFK",
                "2024-02-01",
                "2024-02-01",
            )],
        )
        .await
        .expect("insert failed");

        let app = create_router(test_app_state(pool));
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/dashboard")
                    .header(header::COOKIE, cookie)
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_text(response).await;
        assert!(body.contains("id=\"map\""));
        assert!(body.contains("id=\"filter-type\""));
        assert!(body.contains("id=\"filter-year\""));
        assert!(body.contains("map-legend"));
        assert!(body.contains("window.allHops="));
        assert!(body.contains("/static/map.js"));
    }

    #[tokio::test]
    async fn dashboard_with_error_renders_alert() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/dashboard?error=Sync+failed")
                    .header(header::COOKIE, cookie)
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_text(response).await;
        assert!(body.contains("Sync failed"));
        assert!(body.contains("alert-error"));
    }

    #[tokio::test]
    async fn dashboard_without_auth_and_html_accept_renders_401_page() {
        let pool = test_pool().await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/dashboard")
                    .header(header::ACCEPT, "text/html")
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        let body = body_text(response).await;
        assert!(body.contains("401"));
        assert!(body.contains("Unauthorized"));
        assert!(body.contains("href=\"/login\""));
    }
}

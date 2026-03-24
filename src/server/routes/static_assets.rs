use aide::axum::ApiRouter;
use axum::{Json, response::IntoResponse, routing::get};
use serde::Serialize;

use crate::server::{APP_DESCRIPTION, APP_NAME, APP_SHORT_NAME, THEME_COLOR};

const CSS: &str = include_str!("../../../static/style.css");
const MAP_JS: &str = include_str!("../../../static/map.js");
const STATS_MAP_JS: &str = include_str!("../../../static/stats-map.js");
const NAV_JS: &str = include_str!("../../../static/nav.js");
const LOGO: &str = include_str!("../../../static/logo.svg");
const SW_JS: &str = include_str!("../../../static/sw.js");

async fn serve_css() -> impl IntoResponse {
    (
        [
            ("content-type", "text/css; charset=utf-8"),
            ("cache-control", "public, max-age=86400"),
        ],
        CSS,
    )
}

async fn serve_js() -> impl IntoResponse {
    (
        [
            ("content-type", "application/javascript; charset=utf-8"),
            ("cache-control", "public, max-age=86400"),
        ],
        MAP_JS,
    )
}

async fn serve_stats_js() -> impl IntoResponse {
    (
        [
            ("content-type", "application/javascript; charset=utf-8"),
            ("cache-control", "public, max-age=86400"),
        ],
        STATS_MAP_JS,
    )
}

async fn serve_nav_js() -> impl IntoResponse {
    (
        [
            ("content-type", "application/javascript; charset=utf-8"),
            ("cache-control", "public, max-age=86400"),
        ],
        NAV_JS,
    )
}

async fn serve_logo() -> impl IntoResponse {
    (
        [
            ("content-type", "image/svg+xml"),
            ("cache-control", "public, max-age=86400"),
        ],
        LOGO,
    )
}

#[derive(Serialize)]
struct ManifestIcon {
    src: &'static str,
    sizes: &'static str,
    #[serde(rename = "type")]
    mime: &'static str,
    purpose: &'static str,
}

#[derive(Serialize)]
struct Manifest {
    name: &'static str,
    short_name: &'static str,
    description: &'static str,
    start_url: &'static str,
    scope: &'static str,
    display: &'static str,
    orientation: &'static str,
    theme_color: &'static str,
    background_color: &'static str,
    icons: [ManifestIcon; 1],
}

pub async fn serve_manifest() -> impl IntoResponse {
    (
        [
            ("content-type", "application/manifest+json"),
            ("cache-control", "public, max-age=86400"),
        ],
        Json(Manifest {
            name: APP_NAME,
            short_name: APP_SHORT_NAME,
            description: APP_DESCRIPTION,
            start_url: "/dashboard",
            scope: "/",
            display: "standalone",
            orientation: "any",
            theme_color: THEME_COLOR,
            background_color: THEME_COLOR,
            icons: [ManifestIcon {
                src: "/static/logo.svg",
                sizes: "any",
                mime: "image/svg+xml",
                purpose: "any",
            }],
        }),
    )
}

pub async fn serve_sw() -> impl IntoResponse {
    (
        [
            ("content-type", "application/javascript; charset=utf-8"),
            ("cache-control", "no-cache"),
            ("service-worker-allowed", "/"),
        ],
        SW_JS,
    )
}

pub fn routes() -> ApiRouter<crate::server::AppState> {
    ApiRouter::new()
        .route("/style.css", get(serve_css))
        .route("/map.js", get(serve_js))
        .route("/stats-map.js", get(serve_stats_js))
        .route("/nav.js", get(serve_nav_js))
        .route("/logo.svg", get(serve_logo))
}

#[cfg(test)]
mod tests {
    use crate::server::{create_router, test_helpers::*};
    use axum::{
        body::Body,
        http::{Request, StatusCode, header},
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn static_css_route_serves_text_css_content_type() {
        let pool = test_pool().await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/static/style.css")
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);
        let content_type = response
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .expect("content-type should exist");
        assert!(content_type.contains("text/css"));
    }

    #[tokio::test]
    async fn static_js_route_serves_javascript_content_type() {
        let pool = test_pool().await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/static/map.js")
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);
        let content_type = response
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .expect("content-type should exist");
        assert!(content_type.contains("application/javascript"));
    }

    #[tokio::test]
    async fn static_stats_js_route_serves_javascript_content_type() {
        let pool = test_pool().await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/static/stats-map.js")
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);
        let content_type = response
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .expect("content-type should exist");
        assert!(content_type.contains("application/javascript"));
    }

    #[tokio::test]
    async fn manifest_route_serves_json_content_type() {
        let pool = test_pool().await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/manifest.json")
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);
        let content_type = response
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .expect("content-type should exist");
        assert!(content_type.contains("manifest+json"));

        let body = body_text(response).await;
        let json: serde_json::Value =
            serde_json::from_str(&body).expect("manifest should be valid JSON");
        assert_eq!(json["name"], crate::server::APP_NAME);
        assert_eq!(json["short_name"], crate::server::APP_SHORT_NAME);
        assert_eq!(json["description"], crate::server::APP_DESCRIPTION);
        assert_eq!(json["theme_color"], crate::server::THEME_COLOR);
        assert_eq!(json["background_color"], crate::server::THEME_COLOR);
        assert_eq!(json["start_url"], "/dashboard");
        assert_eq!(json["display"], "standalone");
    }

    #[tokio::test]
    async fn sw_route_serves_javascript_content_type() {
        let pool = test_pool().await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/sw.js")
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);
        let content_type = response
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .expect("content-type should exist");
        assert!(content_type.contains("application/javascript"));
    }

    #[tokio::test]
    async fn nav_js_route_serves_javascript_content_type() {
        let pool = test_pool().await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/static/nav.js")
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);
        let content_type = response
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .expect("content-type should exist");
        assert!(content_type.contains("application/javascript"));
    }
}

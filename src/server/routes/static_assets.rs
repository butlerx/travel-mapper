use aide::axum::ApiRouter;
use axum::{Json, response::IntoResponse, routing::get};
use serde::Serialize;

use crate::server::{APP_DESCRIPTION, APP_NAME, APP_SHORT_NAME, THEME_COLOR};

/// Generate an async handler that serves a static asset with the given
/// content-type and a one-day public cache header.
macro_rules! static_handler {
    ($name:ident, $content:expr, $content_type:expr) => {
        async fn $name() -> impl IntoResponse {
            (
                [
                    ("content-type", $content_type),
                    ("cache-control", "public, max-age=86400"),
                ],
                $content,
            )
        }
    };
}

macro_rules! svg_handler {
    ($name:ident, $content:expr) => {
        static_handler!($name, $content, "image/svg+xml");
    };
}

macro_rules! js_handler {
    ($name:ident, $content:expr) => {
        static_handler!($name, $content, "application/javascript; charset=utf-8");
    };
}

static_handler!(
    serve_css,
    concat!(
        include_str!("../../../static/css/tokens.css"),
        include_str!("../../../static/css/reset.css"),
        include_str!("../../../static/css/layout.css"),
        include_str!("../../../static/css/typography.css"),
        include_str!("../../../static/css/nav.css"),
        include_str!("../../../static/css/cards.css"),
        include_str!("../../../static/css/buttons.css"),
        include_str!("../../../static/css/forms.css"),
        include_str!("../../../static/css/dashboard.css"),
        include_str!("../../../static/css/map.css"),
        include_str!("../../../static/css/pages.css"),
        include_str!("../../../static/css/stats.css"),
        include_str!("../../../static/css/journey-detail.css"),
        include_str!("../../../static/css/settings.css"),
        include_str!("../../../static/css/utilities.css"),
    ),
    "text/css; charset=utf-8"
);

js_handler!(serve_js, include_str!("../../../static/js/map.js"));
js_handler!(
    serve_stats_js,
    include_str!("../../../static/js/stats-map.js")
);
js_handler!(serve_nav_js, include_str!("../../../static/js/nav.js"));
js_handler!(
    serve_add_journey_js,
    include_str!("../../../static/js/add-journey.js")
);
js_handler!(
    serve_journey_map_js,
    include_str!("../../../static/js/journey-map.js")
);
js_handler!(
    serve_edit_panel_js,
    include_str!("../../../static/js/edit-panel.js")
);
js_handler!(
    serve_trip_detail_js,
    include_str!("../../../static/js/trip-detail.js")
);
js_handler!(
    serve_trip_map_js,
    include_str!("../../../static/js/trip-map.js")
);
js_handler!(serve_push_js, include_str!("../../../static/js/push.js"));
js_handler!(
    serve_settings_copy_js,
    include_str!("../../../static/js/settings-copy.js")
);
js_handler!(
    serve_sw_register_js,
    include_str!("../../../static/js/sw-register.js")
);
js_handler!(
    serve_auto_submit_js,
    include_str!("../../../static/js/auto-submit.js")
);

svg_handler!(serve_logo, include_str!("../../../static/icons/logo.svg"));
svg_handler!(
    serve_icon_plane,
    include_str!("../../../static/icons/plane.svg")
);
svg_handler!(
    serve_icon_train,
    include_str!("../../../static/icons/train.svg")
);
svg_handler!(
    serve_icon_boat,
    include_str!("../../../static/icons/boat.svg")
);
svg_handler!(
    serve_icon_transport,
    include_str!("../../../static/icons/transport.svg")
);

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
                src: "/static/icons/logo.svg",
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
        include_str!("../../../static/sw.js"),
    )
}

pub fn routes() -> ApiRouter<crate::server::AppState> {
    ApiRouter::new()
        .route("/style.css", get(serve_css))
        .route("/map.js", get(serve_js))
        .route("/stats-map.js", get(serve_stats_js))
        .route("/nav.js", get(serve_nav_js))
        .route("/add-journey.js", get(serve_add_journey_js))
        .route("/journey-map.js", get(serve_journey_map_js))
        .route("/edit-panel.js", get(serve_edit_panel_js))
        .route("/trip-detail.js", get(serve_trip_detail_js))
        .route("/trip-map.js", get(serve_trip_map_js))
        .route("/push.js", get(serve_push_js))
        .route("/settings-copy.js", get(serve_settings_copy_js))
        .route("/sw-register.js", get(serve_sw_register_js))
        .route("/auto-submit.js", get(serve_auto_submit_js))
        .route("/icons/logo.svg", get(serve_logo))
        .route("/icons/plane.svg", get(serve_icon_plane))
        .route("/icons/train.svg", get(serve_icon_train))
        .route("/icons/boat.svg", get(serve_icon_boat))
        .route("/icons/transport.svg", get(serve_icon_transport))
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

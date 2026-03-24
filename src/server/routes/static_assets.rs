use aide::axum::ApiRouter;
use axum::{response::IntoResponse, routing::get};

const CSS: &str = include_str!("../../../static/style.css");
const MAP_JS: &str = include_str!("../../../static/map.js");
const STATS_MAP_JS: &str = include_str!("../../../static/stats-map.js");
const LOGO: &str = include_str!("../../../static/logo.svg");

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

async fn serve_logo() -> impl IntoResponse {
    (
        [
            ("content-type", "image/svg+xml"),
            ("cache-control", "public, max-age=86400"),
        ],
        LOGO,
    )
}

pub fn routes() -> ApiRouter<crate::server::AppState> {
    ApiRouter::new()
        .route("/style.css", get(serve_css))
        .route("/map.js", get(serve_js))
        .route("/stats-map.js", get(serve_stats_js))
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
}

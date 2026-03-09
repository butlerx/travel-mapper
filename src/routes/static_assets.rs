use axum::response::IntoResponse;

const CSS: &str = include_str!("../../static/style.css");
const MAP_JS: &str = include_str!("../../static/map.js");

pub async fn serve_css() -> impl IntoResponse {
    (
        [
            ("content-type", "text/css; charset=utf-8"),
            ("cache-control", "public, max-age=86400"),
        ],
        CSS,
    )
}

pub async fn serve_js() -> impl IntoResponse {
    (
        [
            ("content-type", "application/javascript; charset=utf-8"),
            ("cache-control", "public, max-age=86400"),
        ],
        MAP_JS,
    )
}

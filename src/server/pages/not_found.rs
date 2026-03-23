use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

pub async fn page() -> Response {
    (
        StatusCode::NOT_FOUND,
        axum::response::Html(super::render_error_page(
            "404",
            "Page Not Found",
            "The page you\u{2019}re looking for doesn\u{2019}t exist or has been moved.",
            "/",
            "Go Home",
        )),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use crate::server::{create_router, test_helpers::*};
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn not_found_page_renders_expected_content() {
        let pool = test_pool().await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/nonexistent-route")
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let body = body_text(response).await;
        assert!(body.contains("404"));
        assert!(body.contains("Page Not Found"));
        assert!(body.contains("href=\"/\""));
    }
}

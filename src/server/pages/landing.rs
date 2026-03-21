use crate::server::{AppState, components::LandingPage};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::CookieJar;
use leptos::prelude::*;

pub async fn landing_page(State(state): State<AppState>, jar: CookieJar) -> Response {
    if super::has_valid_session(&jar, &state).await {
        return Redirect::to("/dashboard").into_response();
    }

    let html = view! { <LandingPage /> };
    (StatusCode::OK, axum::response::Html(html.to_html())).into_response()
}

#[cfg(test)]
mod tests {
    use crate::server::{create_router, test_helpers::helpers::*};
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn landing_page_contains_hero_content() {
        let pool = test_pool().await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/")
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_text(response).await;
        assert!(body.contains("Travel Export"));
        assert!(body.contains("Get Started"));
        assert!(body.contains("href=\"/register\""));
        assert!(body.contains("href=\"/login\""));
    }
}

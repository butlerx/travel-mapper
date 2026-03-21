use crate::server::{AppState, components::AuthFormPage};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::CookieJar;
use leptos::prelude::*;

use super::FormFeedback;

pub async fn register_page(
    State(state): State<AppState>,
    jar: CookieJar,
    Query(feedback): Query<FormFeedback>,
) -> Response {
    if super::has_valid_session(&jar, &state).await {
        return Redirect::to("/dashboard").into_response();
    }

    let html = view! {
        <AuthFormPage
            title="Register"
            action="/auth/register"
            submit_label="Create Account"
            footer_text="Already have an account? "
            footer_link_href="/login"
            footer_link_text="Log in"
            autocomplete_password="new-password"
            error=feedback.error
        />
    };
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
    async fn register_page_renders_form() {
        let pool = test_pool().await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/register")
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_text(response).await;
        assert!(body.contains("Register"));
        assert!(body.contains("action=\"/auth/register\""));
        assert!(body.contains("method=\"post\""));
        assert!(body.contains("id=\"username\""));
        assert!(body.contains("name=\"username\""));
        assert!(body.contains("id=\"password\""));
        assert!(body.contains("name=\"password\""));
        assert!(body.contains("Create Account"));
        assert!(body.contains("href=\"/login\""));
    }

    #[tokio::test]
    async fn register_page_with_error_renders_alert() {
        let pool = test_pool().await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/register?error=Username+taken")
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_text(response).await;
        assert!(body.contains("Username taken"));
        assert!(body.contains("alert-error"));
    }
}

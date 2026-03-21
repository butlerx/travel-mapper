use crate::server::{AppState, components::AuthFormPage};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::CookieJar;
use leptos::prelude::*;

use super::FormFeedback;

pub async fn login_page(
    State(state): State<AppState>,
    jar: CookieJar,
    Query(feedback): Query<FormFeedback>,
) -> Response {
    if super::has_valid_session(&jar, &state).await {
        return Redirect::to("/dashboard").into_response();
    }

    let html = view! {
        <AuthFormPage
            title="Log In"
            action="/auth/login"
            submit_label="Log In"
            footer_text="Don\u{2019}t have an account? "
            footer_link_href="/register"
            footer_link_text="Register"
            autocomplete_password="current-password"
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
    async fn login_page_renders_form() {
        let pool = test_pool().await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/login")
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_text(response).await;
        assert!(body.contains("Log In"));
        assert!(body.contains("action=\"/auth/login\""));
        assert!(body.contains("method=\"post\""));
        assert!(body.contains("id=\"username\""));
        assert!(body.contains("name=\"username\""));
        assert!(body.contains("id=\"password\""));
        assert!(body.contains("name=\"password\""));
        assert!(body.contains("href=\"/register\""));
    }

    #[tokio::test]
    async fn login_page_with_error_renders_alert() {
        let pool = test_pool().await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/login?error=Invalid+credentials")
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_text(response).await;
        assert!(body.contains("Invalid credentials"));
        assert!(body.contains("alert-error"));
    }
}

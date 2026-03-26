use crate::server::{AppState, components::Shell};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::CookieJar;
use leptos::prelude::*;

use super::FormFeedback;

pub async fn page(
    State(state): State<AppState>,
    jar: CookieJar,
    Query(feedback): Query<FormFeedback>,
) -> Response {
    if !state.registration_enabled {
        return Redirect::to("/login?error=Registration+is+disabled").into_response();
    }

    if super::has_valid_session(&jar, &state).await {
        return Redirect::to("/dashboard").into_response();
    }

    let html = view! {
        <Shell title="Register".to_owned()>
            <main class="auth-page">
                <div class="card auth-card">
                    <h1>"Register"</h1>
                    {feedback.error.map(|e| view! {
                        <div class="alert alert-error" role="alert">{e}</div>
                    })}
                    <form method="post" action="/auth/register">
                        <div class="form-group">
                            <label for="first_name">"First Name"</label>
                            <input type="text" id="first_name" name="first_name" required autocomplete="given-name" placeholder=" " />
                        </div>
                        <div class="form-group">
                            <label for="last_name">"Last Name"</label>
                            <input type="text" id="last_name" name="last_name" required autocomplete="family-name" placeholder=" " />
                        </div>
                        <div class="form-group">
                            <label for="email">"Email"</label>
                            <input type="email" id="email" name="email" required autocomplete="email" placeholder=" " />
                        </div>
                        <div class="form-group">
                            <label for="username">"Username"</label>
                            <input type="text" id="username" name="username" required autocomplete="username" placeholder=" " />
                        </div>
                        <div class="form-group">
                            <label for="password">"Password"</label>
                            <input type="password" id="password" name="password" required autocomplete="new-password" placeholder=" " />
                        </div>
                        <button class="btn btn-primary btn-full" type="submit">"Create Account"</button>
                    </form>
                    <div class="form-footer">
                        <p>"Already have an account? " <a href="/login">"Log in"</a></p>
                    </div>
                </div>
            </main>
        </Shell>
    };
    (StatusCode::OK, axum::response::Html(html.to_html())).into_response()
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
        assert!(body.contains("id=\"first_name\""));
        assert!(body.contains("name=\"first_name\""));
        assert!(body.contains("id=\"last_name\""));
        assert!(body.contains("name=\"last_name\""));
        assert!(body.contains("id=\"email\""));
        assert!(body.contains("name=\"email\""));
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

    #[tokio::test]
    async fn register_page_redirects_when_disabled() {
        let pool = test_pool().await;
        let mut state = test_app_state(pool);
        state.registration_enabled = false;
        let app = create_router(state);

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

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(
            response.headers().get("location").unwrap(),
            "/login?error=Registration+is+disabled"
        );
    }
}

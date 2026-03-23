use crate::server::{AppState, components::Shell};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::CookieJar;
use leptos::prelude::*;

pub async fn page(State(state): State<AppState>, jar: CookieJar) -> Response {
    if super::has_valid_session(&jar, &state).await {
        return Redirect::to("/dashboard").into_response();
    }

    let html = view! { <Landing /> };
    (StatusCode::OK, axum::response::Html(html.to_html())).into_response()
}

#[component]
fn Landing() -> impl IntoView {
    view! {
        <Shell title="Travel Mapper".to_owned()>
            <main class="auth-page">
                <div class="card auth-card">
                    <div class="hero">
                        <img class="hero-logo" src="/static/logo.svg" alt="" width="32" height="32" />
                        <div class="hero-badge">{"\u{1F30D}"}" Every journey. Every route. One map."</div>
                        <h1>"Your Travel Story,\u{2003}Visualised"</h1>
                        <p>"Import your trips from TripIt and see every journey mapped, measured, and beautifully presented \u{2014} air, rail, boat trips, and more."</p>
                        <div class="hero-actions">
                            <a class="btn btn-primary" href="/register">"Get Started"</a>
                            <a class="btn btn-secondary" href="/login">"Log In"</a>
                        </div>
                    </div>
                </div>

                <div class="landing-features">
                    <div class="feature-card">
                        <div class="feature-icon">{"\u{1F5FA}\u{FE0F}"}</div>
                        <h3>"Interactive Map"</h3>
                        <p>"See every route on a dark, beautiful map with frequency-weighted arcs. Filter by year or travel type."</p>
                    </div>
                    <div class="feature-card">
                        <div class="feature-icon">{"\u{1F4CA}"}</div>
                        <h3>"Travel Stats"</h3>
                        <p>"Total distance, places visited, countries explored \u{2014} your travel history at a glance."</p>
                    </div>
                    <div class="feature-card">
                        <div class="feature-icon">{"\u{1F504}"}</div>
                        <h3>"TripIt Sync"</h3>
                        <p>"One-click import from TripIt. Background sync keeps your data up to date automatically."</p>
                    </div>
                    <div class="feature-card">
                        <div class="feature-icon">{"\u{1F4E4}"}</div>
                        <h3>"Export Anywhere"</h3>
                        <p>"JSON, CSV, or HTML. Feed your data into Kepler.gl, spreadsheets, or your own tools."</p>
                    </div>
                </div>
            </main>
        </Shell>
    }
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

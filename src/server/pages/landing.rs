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

    let show_register = state.registration_enabled;
    let html = view! { <Landing show_register /> };
    (StatusCode::OK, axum::response::Html(html.to_html())).into_response()
}

#[component]
fn Landing(show_register: bool) -> impl IntoView {
    view! {
        <Shell title="Travel Mapper".to_owned() body_class="landing-layout">
            <main class="landing-page">
                <section class="landing-hero">
                    <img class="landing-hero-logo" src="/static/icons/logo.svg" alt="" width="32" height="32" />
                    <div class="landing-hero-badge">{"\u{1F30D}"}" Every journey. Every route. One map."</div>
                    <h1 class="landing-hero-title">"Your Travel Story, Visualised"</h1>
                    <p class="landing-hero-subtitle">"Import your trips from TripIt or CSV and see every journey mapped, measured, and beautifully presented \u{2014} air, rail, boat, and more."</p>
                    <div class="landing-hero-actions">
                        {show_register.then(|| view! {
                            <a class="btn btn-primary btn-lg" href="/register">"Get Started"</a>
                        })}
                        <a class="btn btn-secondary btn-lg" href="/login">"Log In"</a>
                    </div>
                </section>

                <section class="landing-features">
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
                </section>

                <section class="landing-steps">
                    <h2 class="landing-section-title">"How It Works"</h2>
                    <div class="steps-grid">
                        <div class="step-card">
                            <div class="step-number">"1"</div>
                            <h3>"Connect"</h3>
                            <p>"Link your TripIt account or import a CSV from Flighty, myFlightradar24, or App in the Air."</p>
                        </div>
                        <div class="step-card">
                            <div class="step-number">"2"</div>
                            <h3>"Sync"</h3>
                            <p>"Your trips import automatically. Background sync keeps everything up to date."</p>
                        </div>
                        <div class="step-card">
                            <div class="step-number">"3"</div>
                            <h3>"Explore"</h3>
                            <p>"See your journeys on an interactive map, track stats, and export data anywhere."</p>
                        </div>
                    </div>
                </section>

                <section class="landing-cta">
                    <h2 class="landing-section-title">"Ready to map your travels?"</h2>
                    <p class="landing-cta-subtitle">"Self-hosted, open source, and your data stays yours."</p>
                    <div class="landing-hero-actions">
                        {show_register.then(|| view! {
                            <a class="btn btn-primary btn-lg" href="/register">"Get Started"</a>
                        })}
                        <a class="btn btn-secondary btn-lg" href="/login">"Log In"</a>
                    </div>
                </section>
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
        assert!(body.contains("Travel Mapper"));
        assert!(body.contains("Get Started"));
        assert!(body.contains("href=\"/register\""));
        assert!(body.contains("href=\"/login\""));
    }

    #[tokio::test]
    async fn landing_page_hides_register_when_disabled() {
        let pool = test_pool().await;
        let mut state = test_app_state(pool);
        state.registration_enabled = false;
        let app = create_router(state);

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
        assert!(!body.contains("Get Started"));
        assert!(!body.contains("href=\"/register\""));
        assert!(body.contains("href=\"/login\""));
    }
}

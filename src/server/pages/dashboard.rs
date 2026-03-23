/// Travel statistics computation from hop data.
mod travel_stats;

use crate::{
    db,
    server::{
        AppState,
        components::{NavBar, Shell},
        middleware::AuthUser,
        routes::HopResponse,
    },
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use leptos::prelude::*;
use serde::Deserialize;
use travel_stats::{TravelStats, compute_stats, format_distance, format_year_range};

#[derive(Deserialize, Default)]
pub struct DashboardFeedback {
    pub error: Option<String>,
}

pub async fn page(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(feedback): Query<DashboardFeedback>,
) -> Response {
    let hops = db::hops::GetAll {
        user_id: auth.user_id,
        travel_type_filter: None,
    }
    .execute(&state.db)
    .await
    .unwrap_or_default();

    let hop_count = hops.len();
    let responses: Vec<HopResponse> = hops.into_iter().map(HopResponse::from).collect();
    let travel_stats = compute_stats(&responses);
    let hops_json = serde_json::to_string(&responses).unwrap_or_default();

    let html = view! {
        <DashboardPage
            hops_json=hops_json
            hop_count=hop_count
            stats=travel_stats
            error=feedback.error
        />
    };
    (StatusCode::OK, axum::response::Html(html.to_html())).into_response()
}

#[component]
fn DashboardPage(
    hops_json: String,
    hop_count: usize,
    stats: TravelStats,
    #[prop(optional_no_strip)] error: Option<String>,
) -> impl IntoView {
    let has_hops = hop_count > 0;
    let hops_script = format!("window.allHops={hops_json};");

    let distance = format_distance(stats.total_distance_km);
    let year_range = format_year_range(stats.first_year.as_ref(), stats.last_year.as_ref());

    view! {
        <Shell title="Dashboard".to_owned() body_class="dashboard-layout">
            <NavBar current="dashboard" />
            {error.map(|e| view! {
                <div class="alert alert-error" role="alert">{e}</div>
            })}

            {if has_hops {
                view! {
                    <StatsBar stats=stats distance=distance year_range=year_range />
                    <div class="dashboard-main">
                        <div class="dashboard-map-col">
                            <div id="map"></div>
                            <MapControls hop_count=hop_count />
                        </div>
                        <aside id="journey-sidebar" class="journey-sidebar"></aside>
                    </div>
                    <link rel="stylesheet" href="https://unpkg.com/leaflet@1.9.4/dist/leaflet.css"
                        integrity="sha256-p4NxAoJBhIIN+hmNHrzRCf9tD/miZyoHS5obTRR9BMY="
                        crossorigin="" />
                    <script src="https://unpkg.com/leaflet@1.9.4/dist/leaflet.js"
                        integrity="sha256-20nQCchB9co0qIjJZRGuk2/Z9VM+kNiyxNV1lvTlZBo="
                        crossorigin=""></script>
                    <script inner_html=hops_script></script>
                    <script src="/static/map.js"></script>
                }.into_any()
            } else {
                view! {
                    <main class="container-wide">
                        <section class="card">
                            <div class="empty-state">
                                <div class="empty-state-icon">{"\u{1F30D}"}</div>
                                <p>"No hops yet. Connect TripIt in " <a href="/settings">"Settings"</a> " and sync to see your travel data."</p>
                            </div>
                        </section>
                    </main>
                }.into_any()
            }}
        </Shell>
    }
}

#[component]
fn MapControls(hop_count: usize) -> impl IntoView {
    view! {
        <div class="map-controls">
            <div class="map-filters">
                <label for="filter-type">{"\u{1F3F7}\u{FE0F} Type"}</label>
                <select id="filter-type">
                    <option value="all">"All Types"</option>
                    <option value="air">{"\u{2708}\u{FE0F} Air"}</option>
                    <option value="rail">{"\u{1F686} Rail"}</option>
                    <option value="boat">{"\u{1F6A2} Boat"}</option>
                    <option value="transport">{"\u{1F697} Transport"}</option>
                </select>
                <label for="filter-year">{"\u{1F4C5} Year"}</label>
                <select id="filter-year">
                    <option value="all">"All Years"</option>
                </select>
            </div>
            <div class="map-legend">
                <h3>{"\u{1F5FA}\u{FE0F} Routes"}</h3>
                <div class="legend-item">
                    <div class="legend-swatch legend-air"></div>
                    <span>{"\u{2708}\u{FE0F} Air"}</span>
                </div>
                <div class="legend-item">
                    <div class="legend-swatch legend-rail"></div>
                    <span>{"\u{1F686} Rail"}</span>
                </div>
                <div class="legend-item">
                    <div class="legend-swatch legend-boat"></div>
                    <span>{"\u{1F6A2} Boat"}</span>
                </div>
                <div class="legend-item">
                    <div class="legend-swatch legend-transport"></div>
                    <span>{"\u{1F697} Transport"}</span>
                </div>
                <div class="legend-count" id="hop-count">{hop_count}" journeys"</div>
            </div>
        </div>
    }
}

#[component]
fn StatsBar(stats: TravelStats, distance: String, year_range: String) -> impl IntoView {
    view! {
        <div class="dashboard-stats">
            <div class="stat-row">
                <div class="stat-card">
                    <div class="stat-label">"Journeys"</div>
                    <div class="stat-value">{stats.total_journeys}</div>
                </div>
                <div class="stat-card">
                    <div class="stat-label">"Routes"</div>
                    <div class="stat-value">{stats.total_flights + stats.total_rail}</div>
                </div>
                <div class="stat-card">
                    <div class="stat-label">"Distance"</div>
                    <div class="stat-value">{distance}</div>
                </div>
                <div class="stat-card">
                    <div class="stat-label">"Places"</div>
                    <div class="stat-value">{stats.cities_visited}</div>
                </div>
                <div class="stat-card">
                    <div class="stat-label">"Countries"</div>
                    <div class="stat-value">{stats.airports_visited}</div>
                </div>
                <div class="stat-card">
                    <div class="stat-label">"Years"</div>
                    <div class="stat-value">{year_range}</div>
                </div>
            </div>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        db,
        db::hops::TravelType,
        server::{create_router, test_helpers::*},
    };
    use axum::{
        body::Body,
        http::{Request, StatusCode, header},
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn dashboard_without_hops_renders_empty_state_and_nav() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/dashboard")
                    .header(header::COOKIE, cookie)
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_text(response).await;
        assert!(body.contains("No hops yet"));
        assert!(body.contains("href=\"/settings\""));
        assert!(body.contains("<nav"));
        assert!(body.contains("Dashboard"));
        assert!(body.contains("Settings"));
        assert!(body.contains("action=\"/auth/logout\""));
    }

    #[tokio::test]
    async fn dashboard_with_hops_renders_map_controls_and_script() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let user = db::users::GetByUsername { username: "alice" }
            .execute(&pool)
            .await
            .expect("lookup failed")
            .expect("missing user");
        db::hops::Create {
            trip_id: "trip-1",
            user_id: user.id,
            hops: &[sample_hop(
                TravelType::Air,
                "LHR",
                "JFK",
                "2024-02-01",
                "2024-02-01",
            )],
        }
        .execute(&pool)
        .await
        .expect("insert failed");

        let app = create_router(test_app_state(pool));
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/dashboard")
                    .header(header::COOKIE, cookie)
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_text(response).await;
        assert!(body.contains("id=\"map\""));
        assert!(body.contains("id=\"filter-type\""));
        assert!(body.contains("id=\"filter-year\""));
        assert!(body.contains("map-legend"));
        assert!(body.contains("window.allHops="));
        assert!(body.contains("/static/map.js"));
    }

    #[tokio::test]
    async fn dashboard_with_error_renders_alert() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/dashboard?error=Sync+failed")
                    .header(header::COOKIE, cookie)
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_text(response).await;
        assert!(body.contains("Sync failed"));
        assert!(body.contains("alert-error"));
    }

    #[tokio::test]
    async fn dashboard_without_auth_and_html_accept_renders_401_page() {
        let pool = test_pool().await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/dashboard")
                    .header(header::ACCEPT, "text/html")
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        let body = body_text(response).await;
        assert!(body.contains("401"));
        assert!(body.contains("Unauthorized"));
        assert!(body.contains("href=\"/login\""));
    }

    #[tokio::test]
    async fn dashboard_hops_json_includes_both_past_and_future_dated_hops() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let user = db::users::GetByUsername { username: "alice" }
            .execute(&pool)
            .await
            .expect("lookup failed")
            .expect("missing user");

        db::hops::Create {
            trip_id: "trip-past",
            user_id: user.id,
            hops: &[sample_hop(
                TravelType::Air,
                "LHR",
                "JFK",
                "2024-01-15",
                "2024-01-15",
            )],
        }
        .execute(&pool)
        .await
        .expect("insert past hop failed");

        db::hops::Create {
            trip_id: "trip-future",
            user_id: user.id,
            hops: &[sample_hop(
                TravelType::Air,
                "SFO",
                "NRT",
                "2099-06-01",
                "2099-06-02",
            )],
        }
        .execute(&pool)
        .await
        .expect("insert future hop failed");

        let app = create_router(test_app_state(pool));
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/dashboard")
                    .header(header::COOKIE, cookie)
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_text(response).await;
        assert!(body.contains("window.allHops="));
        assert!(
            body.contains("LHR"),
            "past hop origin should be in hops JSON"
        );
        assert!(
            body.contains("SFO"),
            "future hop origin should be in hops JSON"
        );
        assert!(body.contains("JFK"), "past hop dest should be in hops JSON");
        assert!(
            body.contains("NRT"),
            "future hop dest should be in hops JSON"
        );
    }
}

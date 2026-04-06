use crate::{
    db,
    server::{
        AppState,
        components::{
            NavBar, Shell,
            format_utils::{format_distance, format_year_range},
            map_controls::MapControls,
            overview_cards::{DetailedStats, OverviewCards},
            stats_filters::StatsFilters,
        },
        extractors::AuthUser,
        routes::{JourneyResponse, journeys::JourneyTravelType},
    },
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use leptos::prelude::*;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};

/// Query parameters for filtering the dashboard.
#[derive(Deserialize, Default)]
pub struct DashboardQuery {
    pub year: Option<String>,
    pub travel_type: Option<String>,
    pub origin: Option<String>,
    pub dest: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub airline: Option<String>,
    pub cabin_class: Option<String>,
    pub flight_reason: Option<String>,
    pub q: Option<String>,
    pub error: Option<String>,
}

impl DashboardQuery {
    /// Normalize empty strings to `None` so SQL `?N IS NULL` clauses behave
    /// correctly — HTML forms submit empty inputs as `""`, which Serde
    /// deserializes as `Some("")` rather than `None`.
    fn normalize(&mut self) {
        fn strip_empty(opt: &mut Option<String>) {
            if opt.as_deref().is_some_and(|s| s.trim().is_empty()) {
                *opt = None;
            }
        }
        strip_empty(&mut self.year);
        strip_empty(&mut self.travel_type);
        strip_empty(&mut self.origin);
        strip_empty(&mut self.dest);
        strip_empty(&mut self.date_from);
        strip_empty(&mut self.date_to);
        strip_empty(&mut self.airline);
        strip_empty(&mut self.cabin_class);
        strip_empty(&mut self.flight_reason);
        strip_empty(&mut self.q);
    }
}

pub async fn page(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(mut query): Query<DashboardQuery>,
) -> Response {
    query.normalize();

    let all_hops = db::hops::GetAll {
        user_id: auth.user_id,
        travel_type_filter: None,
    }
    .execute(&state.db)
    .await
    .unwrap_or_default();

    let available_years = extract_available_years(&all_hops);

    // Resolve effective date range: explicit date_from/date_to override year.
    let (effective_date_from, effective_date_to) = resolve_date_range(&query);

    // Fetch filtered journeys via Search.
    let hops = (db::hops::Search {
        user_id: auth.user_id,
        travel_type: query.travel_type.as_deref(),
        origin: query.origin.as_deref(),
        dest: query.dest.as_deref(),
        date_from: effective_date_from.as_deref(),
        date_to: effective_date_to.as_deref(),
        airline: query.airline.as_deref(),
        flight_number: None,
        cabin_class: query.cabin_class.as_deref(),
        flight_reason: query.flight_reason.as_deref(),
        q: query.q.as_deref(),
    })
    .execute(&state.db)
    .await
    .unwrap_or_default();

    let journey_count = hops.len();
    let mut responses: Vec<JourneyResponse> = hops.into_iter().map(JourneyResponse::from).collect();

    apply_enrichments(&state, &mut responses).await;

    let travel_stats = DetailedStats::from(responses.as_slice());
    let journeys_json = serde_json::to_string(&responses).unwrap_or_default();

    let html = view! {
        <DashboardPage
            journeys_json=journeys_json
            journey_count=journey_count
            stats=travel_stats
            available_years=available_years
            query=query
        />
    };
    (StatusCode::OK, axum::response::Html(html.to_html())).into_response()
}

fn extract_available_years(hops: &[db::hops::Row]) -> Vec<String> {
    let mut year_set: HashSet<String> = HashSet::new();
    for hop in hops {
        if hop.start_date.len() >= 4 {
            year_set.insert(hop.start_date[..4].to_owned());
        }
    }
    let mut years: Vec<String> = year_set.into_iter().collect();
    years.sort_unstable();
    years
}

fn resolve_date_range(query: &DashboardQuery) -> (Option<String>, Option<String>) {
    let date_from = query
        .date_from
        .clone()
        .or_else(|| query.year.as_ref().map(|y| format!("{y}-01-01")));
    let date_to = query
        .date_to
        .clone()
        .or_else(|| query.year.as_ref().map(|y| format!("{y}-12-31")));
    (date_from, date_to)
}

async fn apply_enrichments(state: &AppState, responses: &mut [JourneyResponse]) {
    let hop_ids: Vec<i64> = responses.iter().map(|r| r.id).collect();
    if let Ok(enrichments) = (db::status_enrichments::GetByHopIds { hop_ids })
        .execute(&state.db)
        .await
    {
        let enrichment_map: HashMap<i64, db::status_enrichments::Row> =
            enrichments.into_iter().map(|e| (e.hop_id, e)).collect();
        for response in responses.iter_mut() {
            if let Some(enrichment) = enrichment_map.get(&response.id) {
                response.apply_enrichment(enrichment);
            }
        }
    }

    let hop_ids_for_opensky: Vec<i64> = responses
        .iter()
        .filter(|r| r.travel_type == JourneyTravelType::Air)
        .map(|r| r.id)
        .collect();
    if !hop_ids_for_opensky.is_empty()
        && let Ok(opensky_enrichments) = (db::status_enrichments::GetByHopIdsAndProvider {
            hop_ids: hop_ids_for_opensky,
            provider: "opensky",
        })
        .execute(&state.db)
        .await
    {
        let opensky_map: HashMap<i64, db::status_enrichments::Row> = opensky_enrichments
            .into_iter()
            .map(|e| (e.hop_id, e))
            .collect();
        for response in responses.iter_mut() {
            if let Some(enrichment) = opensky_map.get(&response.id) {
                response.apply_opensky_verification(enrichment);
            }
        }
    }
}

#[component]
fn DashboardPage(
    journeys_json: String,
    journey_count: usize,
    stats: DetailedStats,
    available_years: Vec<String>,
    query: DashboardQuery,
) -> impl IntoView {
    let has_journeys = journey_count > 0;
    let error = query.error;

    let distance = format_distance(stats.total_distance_km);
    let year_range = format_year_range(stats.first_year.as_ref(), stats.last_year.as_ref());

    view! {
        <Shell title="Dashboard".to_owned() body_class="dashboard-layout">
            <NavBar current="dashboard" />
            {error.map(|e| view! {
                <div class="alert alert-error" role="alert">{e}</div>
            })}

            {if has_journeys {
                view! {
                    <OverviewCards stats=stats distance=distance year_range=year_range />
                    <StatsFilters
                        available_years=available_years
                        selected_year=query.year
                        selected_travel_type=query.travel_type
                        action="/dashboard".to_owned()
                        extended=true
                        selected_origin=query.origin
                        selected_dest=query.dest
                        selected_date_from=query.date_from
                        selected_date_to=query.date_to
                        selected_airline=query.airline
                        selected_cabin_class=query.cabin_class
                        selected_flight_reason=query.flight_reason
                        selected_q=query.q
                    />
                    <div class="dashboard-main">
                        <div class="dashboard-map-col">
                            <div id="map"></div>
                            <MapControls journey_count=journey_count />
                        </div>
                        <aside id="journey-sidebar" class="journey-sidebar"></aside>
                    </div>
                    <link rel="stylesheet" href="https://unpkg.com/leaflet@1.9.4/dist/leaflet.css"
                        integrity="sha256-p4NxAoJBhIIN+hmNHrzRCf9tD/miZyoHS5obTRR9BMY="
                        crossorigin="" />
                    <script src="https://unpkg.com/leaflet@1.9.4/dist/leaflet.js"
                        integrity="sha256-20nQCchB9co0qIjJZRGuk2/Z9VM+kNiyxNV1lvTlZBo="
                        crossorigin=""></script>
                    <script type="application/json" id="initial-journeys" inner_html=journeys_json></script>
                    <script type="module" src="/static/map.js"></script>
                }.into_any()
            } else {
                view! {
                    <main class="container-wide">
                        <section class="card">
                            <div class="empty-state">
                                <div class="empty-state-icon">{"\u{1F30D}"}</div>
                                <p>"No journeys yet. Connect TripIt in " <a href="/settings">"Settings"</a> " and sync to see your travel data."</p>
                            </div>
                        </section>
                    </main>
                }.into_any()
            }}
        </Shell>
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
    async fn dashboard_without_journeys_renders_empty_state_and_nav() {
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
        assert!(body.contains("No journeys yet"));
        assert!(body.contains("href=\"/settings\""));
        assert!(body.contains("<nav"));
        assert!(body.contains("Dashboard"));
        assert!(body.contains("Settings"));
        assert!(body.contains("action=\"/auth/logout\""));
    }

    #[tokio::test]
    async fn dashboard_with_journeys_renders_map_controls_and_script() {
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
        assert!(body.contains("id=\"travel-type-filter\""));
        assert!(body.contains("id=\"filter-date-from\""));
        assert!(body.contains("map-legend"));
        assert!(body.contains("id=\"initial-journeys\""));
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
    async fn dashboard_journeys_json_includes_both_past_and_future_dated_journeys() {
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
        .expect("insert past journey failed");

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
        .expect("insert future journey failed");

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
        assert!(body.contains("id=\"initial-journeys\""));
        assert!(
            body.contains("LHR"),
            "past journey origin should be in journeys JSON"
        );
        assert!(
            body.contains("SFO"),
            "future journey origin should be in journeys JSON"
        );
        assert!(
            body.contains("JFK"),
            "past journey dest should be in journeys JSON"
        );
        assert!(
            body.contains("NRT"),
            "future journey dest should be in journeys JSON"
        );
    }
}

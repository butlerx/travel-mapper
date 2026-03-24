use crate::{
    db::{
        self,
        hops::{StatsRow, TravelType},
    },
    server::{
        AppState,
        components::{NavBar, Shell},
        extractors::AuthUser,
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
use std::fmt::Write;

#[derive(Deserialize, Default)]
pub struct StatsQuery {
    pub year: Option<String>,
}

#[derive(Default, Clone)]
pub struct CountedItem {
    pub name: String,
    pub count: usize,
}

#[derive(Default, Clone)]
pub struct DetailedStats {
    pub total_journeys: usize,
    pub total_flights: usize,
    pub total_rail: usize,
    pub total_boat: usize,
    pub total_transport: usize,
    pub total_distance_km: u64,
    pub unique_airports: usize,
    pub unique_countries: usize,
    pub top_airlines: Vec<CountedItem>,
    pub top_aircraft: Vec<CountedItem>,
    pub top_routes: Vec<CountedItem>,
    pub cabin_class_breakdown: Vec<CountedItem>,
    pub seat_type_breakdown: Vec<CountedItem>,
    pub flight_reason_breakdown: Vec<CountedItem>,
    pub countries: Vec<CountedItem>,
    pub available_years: Vec<String>,
    pub selected_year: Option<String>,
    pub first_year: Option<String>,
    pub last_year: Option<String>,
}

fn haversine_km(lat1: f64, lng1: f64, lat2: f64, lng2: f64) -> f64 {
    let r = 6371.0_f64;
    let d_lat = (lat2 - lat1).to_radians();
    let d_lng = (lng2 - lng1).to_radians();
    let a = (d_lat / 2.0).sin().powi(2)
        + lat1.to_radians().cos() * lat2.to_radians().cos() * (d_lng / 2.0).sin().powi(2);
    r * 2.0 * a.sqrt().atan2((1.0 - a).sqrt())
}

/// Earth distances cap out well below `u64::MAX`, so truncation is safe.
#[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn positive_km_to_u64(km: f64) -> u64 {
    km.max(0.0) as u64
}

fn top_n(map: &HashMap<String, usize>, n: usize) -> Vec<CountedItem> {
    let mut items: Vec<CountedItem> = map
        .iter()
        .map(|(name, &count)| CountedItem {
            name: name.clone(),
            count,
        })
        .collect();
    items.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.name.cmp(&b.name)));
    items.truncate(n);
    items
}

fn increment(map: &mut HashMap<String, usize>, key: &str) {
    if !key.is_empty() {
        *map.entry(key.to_owned()).or_insert(0) += 1;
    }
}

fn extract_year(date: &str) -> Option<&str> {
    if date.len() >= 4 {
        Some(&date[..4])
    } else {
        None
    }
}

fn count_journey_countries(row: &StatsRow, countries: &mut HashMap<String, usize>) {
    let mut journey_countries: HashSet<String> = HashSet::new();
    if let Some(c) = &row.origin_country
        && !c.is_empty()
    {
        journey_countries.insert(c.to_uppercase());
    }
    if let Some(c) = &row.dest_country
        && !c.is_empty()
    {
        journey_countries.insert(c.to_uppercase());
    }
    for code in &journey_countries {
        *countries.entry(code.clone()).or_insert(0) += 1;
    }
}

pub fn compute_detailed_stats(all_rows: &[StatsRow], year_filter: Option<&str>) -> DetailedStats {
    let mut year_set: HashSet<String> = HashSet::new();
    for row in all_rows {
        if let Some(y) = extract_year(&row.start_date) {
            year_set.insert(y.to_owned());
        }
    }
    let mut available_years: Vec<String> = year_set.into_iter().collect();
    available_years.sort_unstable();

    let rows: Vec<&StatsRow> = if let Some(y) = year_filter {
        all_rows
            .iter()
            .filter(|r| extract_year(&r.start_date) == Some(y))
            .collect()
    } else {
        all_rows.iter().collect()
    };

    let mut stats = DetailedStats {
        total_journeys: rows.len(),
        available_years,
        selected_year: year_filter.map(str::to_owned),
        ..Default::default()
    };

    let mut airports: HashSet<String> = HashSet::new();
    let mut countries: HashMap<String, usize> = HashMap::new();
    let mut airlines: HashMap<String, usize> = HashMap::new();
    let mut aircraft: HashMap<String, usize> = HashMap::new();
    let mut routes: HashMap<String, usize> = HashMap::new();
    let mut cabin_classes: HashMap<String, usize> = HashMap::new();
    let mut seat_types: HashMap<String, usize> = HashMap::new();
    let mut flight_reasons: HashMap<String, usize> = HashMap::new();
    let mut years: Vec<&str> = Vec::new();

    for row in &rows {
        match row.travel_type {
            TravelType::Air => stats.total_flights += 1,
            TravelType::Rail => stats.total_rail += 1,
            TravelType::Boat => stats.total_boat += 1,
            TravelType::Transport => stats.total_transport += 1,
        }

        if row.origin_lat != 0.0
            || row.origin_lng != 0.0
            || row.dest_lat != 0.0
            || row.dest_lng != 0.0
        {
            let km = haversine_km(row.origin_lat, row.origin_lng, row.dest_lat, row.dest_lng);
            if km.is_finite() && km > 0.0 {
                stats.total_distance_km += positive_km_to_u64(km);
            }
        }

        if row.travel_type == TravelType::Air {
            airports.insert(row.origin_name.clone());
            airports.insert(row.dest_name.clone());
        }

        count_journey_countries(row, &mut countries);

        if let Some(a) = &row.airline {
            increment(&mut airlines, a);
        }
        if let Some(a) = &row.aircraft_type {
            increment(&mut aircraft, a);
        }
        if let Some(c) = &row.cabin_class {
            increment(&mut cabin_classes, c);
        }
        if let Some(s) = &row.seat_type {
            increment(&mut seat_types, s);
        }
        if let Some(r) = &row.flight_reason {
            increment(&mut flight_reasons, r);
        }

        let route = format!("{}\u{2192}{}", row.origin_name, row.dest_name);
        *routes.entry(route).or_insert(0) += 1;

        if let Some(y) = extract_year(&row.start_date) {
            years.push(y);
        }
    }

    stats.unique_airports = airports.len();
    stats.unique_countries = countries.len();
    stats.countries = top_n(&countries, 100);
    stats.top_airlines = top_n(&airlines, 10);
    stats.top_aircraft = top_n(&aircraft, 10);
    stats.top_routes = top_n(&routes, 10);
    stats.cabin_class_breakdown = top_n(&cabin_classes, 10);
    stats.seat_type_breakdown = top_n(&seat_types, 10);
    stats.flight_reason_breakdown = top_n(&flight_reasons, 10);

    years.sort_unstable();
    stats.first_year = years.first().map(|y| (*y).to_owned());
    stats.last_year = years.last().map(|y| (*y).to_owned());

    stats
}

pub async fn page(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(query): Query<StatsQuery>,
) -> Response {
    let all_rows = db::hops::GetAllForStats {
        user_id: auth.user_id,
    }
    .execute(&state.db)
    .await
    .unwrap_or_default();

    let detailed = compute_detailed_stats(&all_rows, query.year.as_deref());

    let html = view! {
        <StatsPage stats=detailed />
    };
    (StatusCode::OK, axum::response::Html(html.to_html())).into_response()
}

fn format_distance(km: u64) -> String {
    if km >= 1_000_000 {
        let whole = km / 1_000_000;
        let frac = (km % 1_000_000) / 100_000;
        format!("{whole}.{frac}M km")
    } else if km >= 10_000 {
        format!("{}k km", km / 1_000)
    } else {
        format!("{km} km")
    }
}

fn format_year_range(first: Option<&String>, last: Option<&String>) -> String {
    match (first, last) {
        (Some(f), Some(l)) if f == l => f.clone(),
        (Some(f), Some(l)) => format!("{f}\u{2013}{l}"),
        _ => "\u{2014}".to_owned(),
    }
}

/// Serialize country counts as a JSON object: `{"US":5,"GB":3,...}`.
fn countries_json(countries: &[CountedItem]) -> String {
    let mut buf = String::from('{');
    for (i, item) in countries.iter().enumerate() {
        if i > 0 {
            buf.push(',');
        }
        let _ = write!(buf, "\"{}\":{}", item.name, item.count);
    }
    buf.push('}');
    buf
}

#[component]
fn OverviewCards(stats: DetailedStats, distance: String, year_range: String) -> impl IntoView {
    view! {
        <div class="stats-overview">
            <div class="stat-row">
                <div class="stat-card">
                    <div class="stat-label">"Total Journeys"</div>
                    <div class="stat-value">{stats.total_journeys}</div>
                </div>
                <div class="stat-card">
                    <div class="stat-label">"Flights"</div>
                    <div class="stat-value">{stats.total_flights}</div>
                </div>
                <div class="stat-card">
                    <div class="stat-label">"Rail"</div>
                    <div class="stat-value">{stats.total_rail}</div>
                </div>
                <div class="stat-card">
                    <div class="stat-label">"Distance"</div>
                    <div class="stat-value">{distance}</div>
                </div>
                <div class="stat-card">
                    <div class="stat-label">"Airports"</div>
                    <div class="stat-value">{stats.unique_airports}</div>
                </div>
                <div class="stat-card">
                    <div class="stat-label">"Countries"</div>
                    <div class="stat-value">{stats.unique_countries}</div>
                </div>
                <div class="stat-card">
                    <div class="stat-label">"Years"</div>
                    <div class="stat-value">{year_range}</div>
                </div>
            </div>
        </div>
    }
}

#[component]
fn TopList(title: &'static str, items: Vec<CountedItem>) -> impl IntoView {
    let max_count = items.first().map_or(1, |i| i.count.max(1));

    view! {
        <section class="stats-section">
            <h3 class="stats-section-title">{title}</h3>
            {if items.is_empty() {
                view! { <p class="stats-empty">"No data"</p> }.into_any()
            } else {
                view! {
                    <ul class="stats-top-list">
                        {items.into_iter().map(|item| {
                            let pct = item.count * 100 / max_count;
                            let width = format!("width: {pct}%");
                            view! {
                                <li class="stats-top-item">
                                    <div class="stats-top-bar" style=width></div>
                                    <span class="stats-top-name">{item.name}</span>
                                    <span class="stats-top-count">{item.count}</span>
                                </li>
                            }
                        }).collect::<Vec<_>>()}
                    </ul>
                }.into_any()
            }}
        </section>
    }
}

#[component]
fn YearFilter(available_years: Vec<String>, selected_year: Option<String>) -> impl IntoView {
    if available_years.is_empty() {
        return ().into_any();
    }

    view! {
        <form method="get" action="/stats" class="stats-year-filter">
            <label for="year-filter">"Filter by year: "</label>
            <select name="year" id="year-filter" onchange="this.form.submit()">
                <option value="" selected=selected_year.is_none()>"All years"</option>
                {available_years.into_iter().rev().map(|y| {
                    let is_selected = selected_year.as_ref() == Some(&y);
                    let display = y.clone();
                    view! {
                        <option value={y} selected=is_selected>{display}</option>
                    }
                }).collect::<Vec<_>>()}
            </select>
        </form>
    }
    .into_any()
}

#[allow(clippy::must_use_candidate, clippy::needless_pass_by_value)]
#[component]
fn StatsPage(stats: DetailedStats) -> impl IntoView {
    let has_data = stats.total_journeys > 0;
    let distance = format_distance(stats.total_distance_km);
    let year_range = format_year_range(stats.first_year.as_ref(), stats.last_year.as_ref());

    let available_years = stats.available_years.clone();
    let selected_year = stats.selected_year.clone();
    let top_airlines = stats.top_airlines.clone();
    let top_aircraft = stats.top_aircraft.clone();
    let top_routes = stats.top_routes.clone();
    let cabin_class = stats.cabin_class_breakdown.clone();
    let seat_type = stats.seat_type_breakdown.clone();
    let flight_reason = stats.flight_reason_breakdown.clone();
    let countries = stats.countries.clone();
    let country_counts_json = countries_json(&countries);

    view! {
        <Shell title="Stats".to_owned() body_class="stats-layout">
            <NavBar current="stats" />

            {if has_data {
                view! {
                    <main class="stats-page">
                        <YearFilter available_years=available_years selected_year=selected_year />
                        <OverviewCards stats=stats distance=distance year_range=year_range />
                        <div class="stats-grid">
                            <TopList title="Top Airlines" items=top_airlines />
                            <TopList title="Top Aircraft" items=top_aircraft />
                            <TopList title="Top Routes" items=top_routes />
                            <TopList title="Cabin Class" items=cabin_class />
                            <TopList title="Seat Type" items=seat_type />
                            <TopList title="Flight Reason" items=flight_reason />
                            <TopList title="Countries Visited" items=countries />
                        </div>
                        <section class="stats-section stats-map-section">
                            <h3 class="stats-section-title">"Country Map"</h3>
                            <div id="stats-map"></div>
                        </section>
                        <link rel="stylesheet" href="https://unpkg.com/leaflet@1.9.4/dist/leaflet.css"
                            integrity="sha256-p4NxAoJBhIIN+hmNHrzRCf9tD/miZyoHS5obTRR9BMY="
                            crossorigin="" />
                        <script src="https://unpkg.com/leaflet@1.9.4/dist/leaflet.js"
                            integrity="sha256-20nQCchB9co0qIjJZRGuk2/Z9VM+kNiyxNV1lvTlZBo="
                            crossorigin=""></script>
                        <script src="https://cdn.jsdelivr.net/npm/topojson-client@3"></script>
                        <script type="application/json" id="country-counts" inner_html=country_counts_json></script>
                        <script type="module" src="/static/stats-map.js"></script>
                    </main>
                }.into_any()
            } else {
                view! {
                    <main class="container-wide">
                        <section class="card">
                            <div class="empty-state">
                                <div class="empty-state-icon">{"\u{1F4CA}"}</div>
                                <p>"No travel data yet. Add flights or sync from TripIt to see your stats."</p>
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
    use super::*;
    use crate::{
        db,
        db::hops::{FlightDetail, TravelType},
        server::{create_router, test_helpers::*},
    };
    use axum::{
        body::Body,
        http::{Request, StatusCode, header},
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn stats_page_requires_auth() {
        let pool = test_pool().await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/stats")
                    .header(header::ACCEPT, "text/html")
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn stats_page_renders_empty_state() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/stats")
                    .header(header::COOKIE, cookie)
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_text(response).await;
        assert!(body.contains("Stats"));
        assert!(body.contains("<nav"));
    }

    #[tokio::test]
    async fn stats_page_renders_data_with_journeys() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let user = db::users::GetByUsername { username: "alice" }
            .execute(&pool)
            .await
            .expect("lookup failed")
            .expect("missing user");

        let mut journey = sample_hop(TravelType::Air, "DUB", "LHR", "2024-06-15", "2024-06-15");
        journey.flight_detail = Some(FlightDetail {
            airline: "Aer Lingus".to_string(),
            flight_number: "EI154".to_string(),
            aircraft_type: "A320".to_string(),
            cabin_class: "Economy".to_string(),
            seat: "12A".to_string(),
            pnr: "ABC".to_string(),
        });
        db::hops::Create {
            trip_id: "trip-1",
            user_id: user.id,
            hops: &[journey],
        }
        .execute(&pool)
        .await
        .expect("insert failed");

        let app = create_router(test_app_state(pool));
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/stats")
                    .header(header::COOKIE, cookie)
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_text(response).await;
        assert!(body.contains("Aer Lingus"), "should show airline");
        assert!(body.contains("A320"), "should show aircraft");
        assert!(body.contains("Economy"), "should show cabin class");
    }

    #[tokio::test]
    async fn stats_page_filters_by_year() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let user = db::users::GetByUsername { username: "alice" }
            .execute(&pool)
            .await
            .expect("lookup failed")
            .expect("missing user");

        let mut journey_2024 =
            sample_hop(TravelType::Air, "DUB", "LHR", "2024-06-15", "2024-06-15");
        journey_2024.flight_detail = Some(FlightDetail {
            airline: "Aer Lingus".to_string(),
            ..Default::default()
        });
        let mut journey_2023 =
            sample_hop(TravelType::Air, "SFO", "NRT", "2023-03-01", "2023-03-01");
        journey_2023.flight_detail = Some(FlightDetail {
            airline: "United".to_string(),
            ..Default::default()
        });

        db::hops::Create {
            trip_id: "trip-1",
            user_id: user.id,
            hops: &[journey_2024],
        }
        .execute(&pool)
        .await
        .expect("insert 2024 failed");
        db::hops::Create {
            trip_id: "trip-2",
            user_id: user.id,
            hops: &[journey_2023],
        }
        .execute(&pool)
        .await
        .expect("insert 2023 failed");

        let app = create_router(test_app_state(pool));
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/stats?year=2024")
                    .header(header::COOKIE, cookie)
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_text(response).await;
        assert!(
            body.contains("Aer Lingus"),
            "2024 airline should be present"
        );
        assert!(
            !body.contains("United"),
            "2023 airline should be filtered out"
        );
    }

    #[test]
    fn compute_detailed_stats_empty_input() {
        let stats = compute_detailed_stats(&[], None);
        assert_eq!(stats.total_journeys, 0);
        assert_eq!(stats.total_flights, 0);
        assert_eq!(stats.unique_airports, 0);
        assert!(stats.top_airlines.is_empty());
    }

    #[test]
    fn compute_detailed_stats_counts_travel_types() {
        let rows = vec![
            StatsRow {
                travel_type: TravelType::Air,
                origin_name: "DUB".to_string(),
                origin_lat: 53.4,
                origin_lng: -6.3,
                origin_country: Some("ie".to_string()),
                dest_name: "LHR".to_string(),
                dest_lat: 51.5,
                dest_lng: -0.5,
                dest_country: Some("gb".to_string()),
                start_date: "2024-06-01".to_string(),
                end_date: "2024-06-01".to_string(),
                airline: Some("Aer Lingus".to_string()),
                aircraft_type: Some("A320".to_string()),
                cabin_class: Some("Economy".to_string()),
                seat_type: Some("Window".to_string()),
                flight_reason: Some("Leisure".to_string()),
            },
            StatsRow {
                travel_type: TravelType::Rail,
                origin_name: "Paris".to_string(),
                origin_lat: 48.9,
                origin_lng: 2.3,
                origin_country: Some("fr".to_string()),
                dest_name: "London".to_string(),
                dest_lat: 51.5,
                dest_lng: -0.1,
                dest_country: Some("gb".to_string()),
                start_date: "2024-07-01".to_string(),
                end_date: "2024-07-01".to_string(),
                airline: None,
                aircraft_type: None,
                cabin_class: None,
                seat_type: None,
                flight_reason: None,
            },
        ];

        let stats = compute_detailed_stats(&rows, None);
        assert_eq!(stats.total_journeys, 2);
        assert_eq!(stats.total_flights, 1);
        assert_eq!(stats.total_rail, 1);
        assert_eq!(stats.unique_airports, 2);
        assert_eq!(stats.unique_countries, 3);
        assert_eq!(stats.countries.len(), 3);
        // GB appears in both journeys (DUB→LHR and Paris→London), IE in 1, FR in 1
        assert_eq!(stats.countries[0].name, "GB");
        assert_eq!(stats.countries[0].count, 2);
        assert_eq!(stats.top_airlines.len(), 1);
        assert_eq!(stats.top_airlines[0].name, "Aer Lingus");
    }

    #[test]
    fn compute_detailed_stats_deduplicates_same_country_journey() {
        let rows = vec![StatsRow {
            travel_type: TravelType::Air,
            origin_name: "SFO".to_string(),
            origin_lat: 37.6,
            origin_lng: -122.4,
            origin_country: Some("us".to_string()),
            dest_name: "LAX".to_string(),
            dest_lat: 33.9,
            dest_lng: -118.4,
            dest_country: Some("us".to_string()),
            start_date: "2024-06-01".to_string(),
            end_date: "2024-06-01".to_string(),
            airline: None,
            aircraft_type: None,
            cabin_class: None,
            seat_type: None,
            flight_reason: None,
        }];

        let stats = compute_detailed_stats(&rows, None);
        assert_eq!(stats.unique_countries, 1);
        assert_eq!(stats.countries.len(), 1);
        assert_eq!(stats.countries[0].name, "US");
        assert_eq!(stats.countries[0].count, 1);
    }
}

use crate::server::components::{
    format_utils::{format_distance, format_year_range},
    overview_cards::OverviewCards,
    stats_filters::StatsFilters,
    top_list::{TopList, miles_section_view, optional_top_list, spending_section_view},
};
pub use crate::server::components::{overview_cards::DetailedStats, top_list::CountedItem};
use crate::{
    db::hops::{StatsRow, TravelType},
    distance::{haversine_km, haversine_miles},
    server::components::{NavBar, Shell},
};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use leptos::prelude::*;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::fmt::Write;

/// Query parameters for filtering the stats page by year.
#[derive(Deserialize, Default, schemars::JsonSchema)]
pub struct StatsQuery {
    pub year: Option<String>,
    pub travel_type: Option<String>,
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

fn summarize_spending(spending: HashMap<String, f64>) -> Vec<String> {
    let mut spending_vec: Vec<(String, f64)> = spending.into_iter().collect();
    spending_vec.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    spending_vec
        .into_iter()
        .map(|(currency, total)| format!("{total:.2} {currency}"))
        .collect()
}

fn summarize_miles(miles_map: HashMap<String, f64>) -> Vec<String> {
    let mut miles_vec: Vec<(String, f64)> = miles_map.into_iter().collect();
    miles_vec.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    miles_vec
        .into_iter()
        .filter(|(_, total)| total.is_finite() && *total > 0.0)
        .map(|(program, total)| format!("{total:.0} mi ({program})"))
        .collect()
}

fn sorted_available_years(all_rows: &[StatsRow]) -> Vec<String> {
    let mut year_set: HashSet<String> = HashSet::new();
    for row in all_rows {
        if let Some(y) = extract_year(&row.start_date) {
            year_set.insert(y.to_owned());
        }
    }
    let mut available_years: Vec<String> = year_set.into_iter().collect();
    available_years.sort_unstable();
    available_years
}

fn matches_travel_type_filter(row: &StatsRow, travel_type_filter: &str) -> bool {
    matches!(
        (&row.travel_type, travel_type_filter),
        (TravelType::Air, "air")
            | (TravelType::Rail, "rail")
            | (TravelType::Boat, "boat")
            | (TravelType::Transport, "transport")
    )
}

fn selected_rows<'a>(
    all_rows: &'a [StatsRow],
    year_filter: Option<&str>,
    travel_type_filter: Option<&str>,
) -> Vec<&'a StatsRow> {
    let rows: Vec<&StatsRow> = if let Some(y) = year_filter {
        all_rows
            .iter()
            .filter(|r| extract_year(&r.start_date) == Some(y))
            .collect()
    } else {
        all_rows.iter().collect()
    };

    if let Some(t) = travel_type_filter {
        rows.into_iter()
            .filter(|r| matches_travel_type_filter(r, t))
            .collect()
    } else {
        rows
    }
}

fn add_row_miles(miles_by_program: &mut HashMap<String, f64>, row: &StatsRow) {
    if let Some(miles) = row.miles_earned {
        if miles.is_finite() && miles > 0.0 {
            let program = row
                .loyalty_program
                .clone()
                .unwrap_or_else(|| "Unassigned".to_owned());
            *miles_by_program.entry(program).or_insert(0.0) += miles;
        }
    } else if row.travel_type == TravelType::Air
        && (row.origin_lat != 0.0
            || row.origin_lng != 0.0
            || row.dest_lat != 0.0
            || row.dest_lng != 0.0)
    {
        let miles = haversine_miles(row.origin_lat, row.origin_lng, row.dest_lat, row.dest_lng);
        if miles.is_finite() && miles > 0.0 {
            *miles_by_program
                .entry("Unassigned".to_owned())
                .or_insert(0.0) += miles;
        }
    }
}

#[derive(Default)]
struct Accumulators<'a> {
    airports: HashSet<String>,
    stations: HashSet<String>,
    countries: HashMap<String, usize>,
    airlines: HashMap<String, usize>,
    aircraft: HashMap<String, usize>,
    routes: HashMap<String, usize>,
    cabin_classes: HashMap<String, usize>,
    seat_types: HashMap<String, usize>,
    flight_reasons: HashMap<String, usize>,
    rail_carriers: HashMap<String, usize>,
    train_numbers: HashMap<String, usize>,
    rail_service_classes: HashMap<String, usize>,
    ships: HashMap<String, usize>,
    boat_cabin_types: HashMap<String, usize>,
    transport_carriers: HashMap<String, usize>,
    vehicle_types: HashMap<String, usize>,
    spending: HashMap<String, f64>,
    miles_by_program: HashMap<String, f64>,
    years: Vec<&'a str>,
}

fn accumulate_row<'a>(acc: &mut Accumulators<'a>, row: &'a StatsRow, distance_km: &mut u64) {
    if row.origin_lat != 0.0 || row.origin_lng != 0.0 || row.dest_lat != 0.0 || row.dest_lng != 0.0
    {
        let km = haversine_km(row.origin_lat, row.origin_lng, row.dest_lat, row.dest_lng);
        if km.is_finite() && km > 0.0 {
            *distance_km += positive_km_to_u64(km);
        }
    }

    match row.travel_type {
        TravelType::Air => {
            acc.airports.insert(row.origin_name.clone());
            acc.airports.insert(row.dest_name.clone());
        }
        TravelType::Rail => {
            acc.stations.insert(row.origin_name.clone());
            acc.stations.insert(row.dest_name.clone());
            if let Some(v) = &row.rail_carrier {
                increment(&mut acc.rail_carriers, v);
            }
            if let Some(v) = &row.train_number {
                increment(&mut acc.train_numbers, v);
            }
            if let Some(v) = &row.service_class {
                increment(&mut acc.rail_service_classes, v);
            }
        }
        TravelType::Boat => {
            if let Some(v) = &row.ship_name {
                increment(&mut acc.ships, v);
            }
            if let Some(v) = &row.boat_cabin_type {
                increment(&mut acc.boat_cabin_types, v);
            }
        }
        TravelType::Transport => {
            if let Some(v) = &row.transport_carrier {
                increment(&mut acc.transport_carriers, v);
            }
            if let Some(v) = &row.vehicle_description {
                increment(&mut acc.vehicle_types, v);
            }
        }
    }

    count_journey_countries(row, &mut acc.countries);

    if let Some(a) = &row.airline {
        increment(&mut acc.airlines, a);
    }
    if let Some(a) = &row.aircraft_type {
        increment(&mut acc.aircraft, a);
    }
    if let Some(c) = &row.cabin_class {
        increment(&mut acc.cabin_classes, c);
    }
    if let Some(s) = &row.seat_type {
        increment(&mut acc.seat_types, s);
    }
    if let Some(r) = &row.flight_reason {
        increment(&mut acc.flight_reasons, r);
    }

    if let Some(amount) = row.cost_amount {
        let currency = row.cost_currency.clone().unwrap_or_default();
        let key = if currency.is_empty() {
            "???".to_owned()
        } else {
            currency
        };
        *acc.spending.entry(key).or_insert(0.0) += amount;
    }

    add_row_miles(&mut acc.miles_by_program, row);

    let route = format!("{}\u{2192}{}", row.origin_name, row.dest_name);
    *acc.routes.entry(route).or_insert(0) += 1;

    if let Some(y) = extract_year(&row.start_date) {
        acc.years.push(y);
    }
}

fn finalize_stats(acc: Accumulators<'_>, stats: &mut DetailedStats) {
    stats.unique_airports = acc.airports.len();
    stats.unique_stations = acc.stations.len();
    stats.unique_countries = acc.countries.len();
    stats.countries = top_n(&acc.countries, 100);
    stats.top_airlines = top_n(&acc.airlines, 10);
    stats.top_aircraft = top_n(&acc.aircraft, 10);
    stats.top_routes = top_n(&acc.routes, 10);
    stats.cabin_class_breakdown = top_n(&acc.cabin_classes, 10);
    stats.seat_type_breakdown = top_n(&acc.seat_types, 10);
    stats.flight_reason_breakdown = top_n(&acc.flight_reasons, 10);
    stats.top_rail_carriers = top_n(&acc.rail_carriers, 10);
    stats.top_train_numbers = top_n(&acc.train_numbers, 10);
    stats.rail_service_class_breakdown = top_n(&acc.rail_service_classes, 10);
    stats.top_ships = top_n(&acc.ships, 10);
    stats.boat_cabin_type_breakdown = top_n(&acc.boat_cabin_types, 10);
    stats.top_transport_carriers = top_n(&acc.transport_carriers, 10);
    stats.transport_vehicle_breakdown = top_n(&acc.vehicle_types, 10);

    let mut years = acc.years;
    years.sort_unstable();
    stats.first_year = years.first().map(|y| (*y).to_owned());
    stats.last_year = years.last().map(|y| (*y).to_owned());
    stats.spending_summary = summarize_spending(acc.spending);
    let mut miles_entries: Vec<(String, f64)> = acc.miles_by_program.into_iter().collect();
    miles_entries.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    stats.miles_by_program.clone_from(&miles_entries);
    stats.miles_summary = summarize_miles(miles_entries.into_iter().collect::<HashMap<_, _>>());
}

pub fn compute_detailed_stats(
    all_rows: &[StatsRow],
    year_filter: Option<&str>,
    travel_type_filter: Option<&str>,
) -> DetailedStats {
    let year_filter = year_filter.filter(|v| !v.is_empty());
    let travel_type_filter = travel_type_filter.filter(|v| !v.is_empty());
    let available_years = sorted_available_years(all_rows);
    let rows = selected_rows(all_rows, year_filter, travel_type_filter);

    let mut stats = DetailedStats {
        total_journeys: rows.len(),
        available_years,
        selected_year: year_filter.map(str::to_owned),
        selected_travel_type: travel_type_filter.map(str::to_owned),
        ..Default::default()
    };

    let mut acc = Accumulators::default();

    for row in &rows {
        match row.travel_type {
            TravelType::Air => stats.total_flights += 1,
            TravelType::Rail => stats.total_rail += 1,
            TravelType::Boat => stats.total_boat += 1,
            TravelType::Transport => stats.total_transport += 1,
        }
        accumulate_row(&mut acc, row, &mut stats.total_distance_km);
    }

    finalize_stats(acc, &mut stats);
    stats
}

pub(crate) fn render_page(stats: DetailedStats) -> Response {
    let html = view! {
        <StatsPage stats=stats />
    };
    (StatusCode::OK, axum::response::Html(html.to_html())).into_response()
}

/// Render the public share stats page (no navbar, with OG meta tags).
pub(crate) fn render_share_page(stats: DetailedStats, token: &str) -> Response {
    let html = view! {
        <ShareStatsPage stats=stats token=token.to_owned() />
    };
    (StatusCode::OK, axum::response::Html(html.to_html())).into_response()
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

#[allow(clippy::must_use_candidate, clippy::needless_pass_by_value)]
#[component]
fn StatsPage(stats: DetailedStats) -> impl IntoView {
    let has_data = stats.total_journeys > 0;
    let distance = format_distance(stats.total_distance_km);
    let year_range = format_year_range(stats.first_year.as_ref(), stats.last_year.as_ref());

    let available_years = stats.available_years.clone();
    let selected_year = stats.selected_year.clone();
    let selected_travel_type = stats.selected_travel_type.clone();
    let top_airlines = stats.top_airlines.clone();
    let top_aircraft = stats.top_aircraft.clone();
    let top_routes = stats.top_routes.clone();
    let cabin_class = stats.cabin_class_breakdown.clone();
    let seat_type = stats.seat_type_breakdown.clone();
    let flight_reason = stats.flight_reason_breakdown.clone();
    let top_rail_carriers = stats.top_rail_carriers.clone();
    let top_train_numbers = stats.top_train_numbers.clone();
    let rail_service_class = stats.rail_service_class_breakdown.clone();
    let top_ships = stats.top_ships.clone();
    let boat_cabin_types = stats.boat_cabin_type_breakdown.clone();
    let top_transport_carriers = stats.top_transport_carriers.clone();
    let transport_vehicle_types = stats.transport_vehicle_breakdown.clone();
    let countries = stats.countries.clone();
    let spending_summary = stats.spending_summary.clone();
    let miles_summary = stats.miles_summary.clone();
    let country_counts_json = countries_json(&countries);
    let filters_selected_year = selected_year.clone();
    let filters_selected_travel_type = selected_travel_type.clone();

    view! {
        <Shell title="Stats".to_owned() body_class="stats-layout">
            <NavBar current="stats" />

            {if has_data {
                view! {
                    <main class="stats-page">
                        <OverviewCards stats=stats distance=distance year_range=year_range />
                        <StatsFilters
                            available_years=available_years
                            selected_year=filters_selected_year
                            selected_travel_type=filters_selected_travel_type
                        />
                        <div class="stats-grid">
                            {optional_top_list("Top Airlines", top_airlines)}
                            {optional_top_list("Top Aircraft", top_aircraft)}
                            {optional_top_list("Top Routes", top_routes)}
                            {optional_top_list("Cabin Class", cabin_class)}
                            {optional_top_list("Seat Type", seat_type)}
                            {optional_top_list("Flight Reason", flight_reason)}
                            {optional_top_list("Top Rail Carriers", top_rail_carriers)}
                            {optional_top_list("Top Train Numbers", top_train_numbers)}
                            {optional_top_list("Rail Service Class", rail_service_class)}
                            {optional_top_list("Top Ships", top_ships)}
                            {optional_top_list("Boat Cabin Type", boat_cabin_types)}
                            {optional_top_list("Top Transport Carriers", top_transport_carriers)}
                            {optional_top_list("Transport Vehicle Types", transport_vehicle_types)}
                            <TopList title="Countries Visited" items=countries />
                            {spending_section_view(spending_summary)}
                            {miles_section_view(miles_summary)}
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

fn build_og_description(stats: &DetailedStats) -> String {
    let parts = [
        (stats.total_flights, "flight"),
        (stats.total_rail, "rail journey"),
        (stats.total_boat, "boat trip"),
    ];
    let mut segments: Vec<String> = Vec::new();
    for (count, label) in &parts {
        if *count > 0 {
            let plural = if *count == 1 { "" } else { "s" };
            segments.push(format!("{count} {label}{plural}"));
        }
    }
    if segments.is_empty() {
        return "Travel stats".to_owned();
    }
    let distance = format_distance(stats.total_distance_km);
    format!(
        "{} across {} countries \u{00b7} {}",
        segments.join(", "),
        stats.unique_countries,
        distance
    )
}

#[allow(clippy::must_use_candidate, clippy::needless_pass_by_value)]
#[component]
fn ShareStatsPage(stats: DetailedStats, token: String) -> impl IntoView {
    let has_data = stats.total_journeys > 0;
    let og_description = build_og_description(&stats);
    let share_action = format!("/share/{token}");
    let og_meta = format!(
        r#"<meta property="og:title" content="Travel Stats"><meta property="og:description" content="{}"><meta property="og:url" content="{}">"#,
        og_description.replace('"', "&quot;"),
        share_action.replace('"', "&quot;"),
    );
    let distance = format_distance(stats.total_distance_km);
    let year_range = format_year_range(stats.first_year.as_ref(), stats.last_year.as_ref());

    let available_years = stats.available_years.clone();
    let selected_year = stats.selected_year.clone();
    let selected_travel_type = stats.selected_travel_type.clone();
    let top_airlines = stats.top_airlines.clone();
    let top_aircraft = stats.top_aircraft.clone();
    let top_routes = stats.top_routes.clone();
    let cabin_class = stats.cabin_class_breakdown.clone();
    let seat_type = stats.seat_type_breakdown.clone();
    let flight_reason = stats.flight_reason_breakdown.clone();
    let top_rail_carriers = stats.top_rail_carriers.clone();
    let top_train_numbers = stats.top_train_numbers.clone();
    let rail_service_class = stats.rail_service_class_breakdown.clone();
    let top_ships = stats.top_ships.clone();
    let boat_cabin_types = stats.boat_cabin_type_breakdown.clone();
    let top_transport_carriers = stats.top_transport_carriers.clone();
    let transport_vehicle_types = stats.transport_vehicle_breakdown.clone();
    let countries = stats.countries.clone();
    let spending_summary = stats.spending_summary.clone();
    let miles_summary = stats.miles_summary.clone();
    let country_counts_json = countries_json(&countries);
    let filters_selected_year = selected_year.clone();
    let filters_selected_travel_type = selected_travel_type.clone();

    view! {
        <Shell
            title="Shared Travel Stats".to_owned()
            body_class="stats-layout"
            og_meta=og_meta
        >
            {if has_data {
                view! {
                    <main class="stats-page">
                        <StatsFilters
                            available_years=available_years
                            selected_year=filters_selected_year
                            selected_travel_type=filters_selected_travel_type
                            action=share_action
                        />
                        <OverviewCards stats=stats distance=distance year_range=year_range />
                        <div class="stats-grid">
                            {optional_top_list("Top Airlines", top_airlines)}
                            {optional_top_list("Top Aircraft", top_aircraft)}
                            {optional_top_list("Top Routes", top_routes)}
                            {optional_top_list("Cabin Class", cabin_class)}
                            {optional_top_list("Seat Type", seat_type)}
                            {optional_top_list("Flight Reason", flight_reason)}
                            {optional_top_list("Top Rail Carriers", top_rail_carriers)}
                            {optional_top_list("Top Train Numbers", top_train_numbers)}
                            {optional_top_list("Rail Service Class", rail_service_class)}
                            {optional_top_list("Top Ships", top_ships)}
                            {optional_top_list("Boat Cabin Type", boat_cabin_types)}
                            {optional_top_list("Top Transport Carriers", top_transport_carriers)}
                            {optional_top_list("Transport Vehicle Types", transport_vehicle_types)}
                            <TopList title="Countries Visited" items=countries />
                            {spending_section_view(spending_summary)}
                            {miles_section_view(miles_summary)}
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
                                <p>"No travel data to display."</p>
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
                    .header(header::ACCEPT, "text/html")
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
                    .header(header::ACCEPT, "text/html")
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
                    .header(header::ACCEPT, "text/html")
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

    #[tokio::test]
    async fn stats_page_filters_by_travel_type() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let user = db::users::GetByUsername { username: "alice" }
            .execute(&pool)
            .await
            .expect("lookup failed")
            .expect("missing user");

        let mut air_journey = sample_hop(TravelType::Air, "DUB", "LHR", "2024-06-15", "2024-06-15");
        air_journey.flight_detail = Some(FlightDetail {
            airline: "Aer Lingus".to_string(),
            ..Default::default()
        });

        let rail_journey = sample_hop(
            TravelType::Rail,
            "Paris",
            "London",
            "2024-07-01",
            "2024-07-01",
        );

        db::hops::Create {
            trip_id: "trip-air",
            user_id: user.id,
            hops: &[air_journey],
        }
        .execute(&pool)
        .await
        .expect("insert air failed");

        db::hops::Create {
            trip_id: "trip-rail",
            user_id: user.id,
            hops: &[rail_journey],
        }
        .execute(&pool)
        .await
        .expect("insert rail failed");

        let app = create_router(test_app_state(pool));
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/stats?travel_type=rail")
                    .header(header::COOKIE, cookie)
                    .header(header::ACCEPT, "text/html")
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_text(response).await;
        assert!(!body.contains("Aer Lingus"));
        assert!(body.contains("Stations"));
    }

    #[test]
    fn compute_detailed_stats_empty_input() {
        let stats = compute_detailed_stats(&[], None, None);
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
                rail_carrier: None,
                train_number: None,
                service_class: None,
                ship_name: None,
                boat_cabin_type: None,
                transport_carrier: None,
                vehicle_description: None,
                cost_amount: None,
                cost_currency: None,
                loyalty_program: None,
                miles_earned: None,
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
                rail_carrier: Some("Eurostar".to_string()),
                train_number: Some("ES9026".to_string()),
                service_class: Some("Standard Premier".to_string()),
                ship_name: None,
                boat_cabin_type: None,
                transport_carrier: None,
                vehicle_description: None,
                cost_amount: None,
                cost_currency: None,
                loyalty_program: None,
                miles_earned: None,
            },
        ];

        let stats = compute_detailed_stats(&rows, None, None);
        assert_eq!(stats.total_journeys, 2);
        assert_eq!(stats.total_flights, 1);
        assert_eq!(stats.total_rail, 1);
        assert_eq!(stats.unique_airports, 2);
        assert_eq!(stats.unique_stations, 2);
        assert_eq!(stats.unique_countries, 3);
        assert_eq!(stats.countries.len(), 3);
        // GB appears in both journeys (DUB→LHR and Paris→London), IE in 1, FR in 1
        assert_eq!(stats.countries[0].name, "GB");
        assert_eq!(stats.countries[0].count, 2);
        assert_eq!(stats.top_airlines.len(), 1);
        assert_eq!(stats.top_airlines[0].name, "Aer Lingus");
        assert_eq!(stats.top_rail_carriers.len(), 1);
        assert_eq!(stats.top_rail_carriers[0].name, "Eurostar");
        assert_eq!(stats.top_train_numbers[0].name, "ES9026");
        assert_eq!(
            stats.rail_service_class_breakdown[0].name,
            "Standard Premier"
        );
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
            rail_carrier: None,
            train_number: None,
            service_class: None,
            ship_name: None,
            boat_cabin_type: None,
            transport_carrier: None,
            vehicle_description: None,
            cost_amount: None,
            cost_currency: None,
            loyalty_program: None,
            miles_earned: None,
        }];

        let stats = compute_detailed_stats(&rows, None, None);
        assert_eq!(stats.unique_countries, 1);
        assert_eq!(stats.countries.len(), 1);
        assert_eq!(stats.countries[0].name, "US");
        assert_eq!(stats.countries[0].count, 1);
    }

    #[test]
    fn compute_detailed_stats_filters_by_travel_type() {
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
                aircraft_type: None,
                cabin_class: None,
                seat_type: None,
                flight_reason: None,
                rail_carrier: None,
                train_number: None,
                service_class: None,
                ship_name: None,
                boat_cabin_type: None,
                transport_carrier: None,
                vehicle_description: None,
                cost_amount: None,
                cost_currency: None,
                loyalty_program: None,
                miles_earned: None,
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
                rail_carrier: Some("Eurostar".to_string()),
                train_number: None,
                service_class: None,
                ship_name: None,
                boat_cabin_type: None,
                transport_carrier: None,
                vehicle_description: None,
                cost_amount: None,
                cost_currency: None,
                loyalty_program: None,
                miles_earned: None,
            },
        ];

        let stats = compute_detailed_stats(&rows, None, Some("rail"));
        assert_eq!(stats.total_journeys, 1);
        assert_eq!(stats.total_flights, 0);
        assert_eq!(stats.total_rail, 1);
        assert_eq!(stats.selected_travel_type.as_deref(), Some("rail"));
        assert_eq!(stats.top_rail_carriers.len(), 1);
    }
}

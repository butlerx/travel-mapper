use crate::{
    db,
    db::hops::{StatsRow, TravelType},
    server::{AppState, components::StatsPage, middleware::AuthUser},
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use leptos::prelude::*;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};

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
    pub total_hops: usize,
    pub total_flights: usize,
    pub total_rail: usize,
    pub total_cruise: usize,
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
        total_hops: rows.len(),
        available_years,
        selected_year: year_filter.map(str::to_owned),
        ..Default::default()
    };

    let mut airports: HashSet<String> = HashSet::new();
    let mut countries: HashSet<String> = HashSet::new();
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
            TravelType::Cruise => stats.total_cruise += 1,
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

        if let Some(c) = &row.origin_country
            && !c.is_empty()
        {
            countries.insert(c.to_uppercase());
        }
        if let Some(c) = &row.dest_country
            && !c.is_empty()
        {
            countries.insert(c.to_uppercase());
        }

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

pub async fn stats_page(
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        db,
        db::hops::{FlightDetail, TravelType},
        server::{create_router, test_helpers::helpers::*},
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
    async fn stats_page_renders_data_with_hops() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let user = db::users::GetByUsername { username: "alice" }
            .execute(&pool)
            .await
            .expect("lookup failed")
            .expect("missing user");

        let mut hop = sample_hop(TravelType::Air, "DUB", "LHR", "2024-06-15", "2024-06-15");
        hop.flight_detail = Some(FlightDetail {
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
            hops: &[hop],
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

        let mut hop_2024 = sample_hop(TravelType::Air, "DUB", "LHR", "2024-06-15", "2024-06-15");
        hop_2024.flight_detail = Some(FlightDetail {
            airline: "Aer Lingus".to_string(),
            ..Default::default()
        });
        let mut hop_2023 = sample_hop(TravelType::Air, "SFO", "NRT", "2023-03-01", "2023-03-01");
        hop_2023.flight_detail = Some(FlightDetail {
            airline: "United".to_string(),
            ..Default::default()
        });

        db::hops::Create {
            trip_id: "trip-1",
            user_id: user.id,
            hops: &[hop_2024],
        }
        .execute(&pool)
        .await
        .expect("insert 2024 failed");
        db::hops::Create {
            trip_id: "trip-2",
            user_id: user.id,
            hops: &[hop_2023],
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
        assert_eq!(stats.total_hops, 0);
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
        assert_eq!(stats.total_hops, 2);
        assert_eq!(stats.total_flights, 1);
        assert_eq!(stats.total_rail, 1);
        assert_eq!(stats.unique_airports, 2);
        assert_eq!(stats.unique_countries, 3);
        assert_eq!(stats.top_airlines.len(), 1);
        assert_eq!(stats.top_airlines[0].name, "Aer Lingus");
    }
}

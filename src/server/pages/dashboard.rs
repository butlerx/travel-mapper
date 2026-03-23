use crate::{
    db,
    server::{AppState, components::DashboardPage, middleware::AuthUser, routes::HopResponse},
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use leptos::prelude::*;
use serde::Deserialize;
use std::collections::HashSet;

#[derive(Deserialize, Default)]
pub struct DashboardFeedback {
    pub error: Option<String>,
}

/// Stats computed from a user's travel hops.
#[derive(Default, Clone)]
pub struct TravelStats {
    pub total_journeys: usize,
    pub total_flights: usize,
    pub total_rail: usize,
    pub total_distance_km: u64,
    pub airports_visited: usize,
    pub cities_visited: usize,
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

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn positive_km_to_u64(km: f64) -> u64 {
    km.max(0.0).trunc() as u64
}

fn compute_stats(hops: &[HopResponse]) -> TravelStats {
    let mut stats = TravelStats {
        total_journeys: hops.len(),
        ..Default::default()
    };

    let mut places: HashSet<String> = HashSet::new();
    let mut years: Vec<&str> = Vec::new();

    for hop in hops {
        match hop.travel_type.as_str() {
            "air" => stats.total_flights += 1,
            "rail" => stats.total_rail += 1,
            _ => {}
        }

        if hop.origin_lat != 0.0
            || hop.origin_lng != 0.0
            || hop.dest_lat != 0.0
            || hop.dest_lng != 0.0
        {
            let km = haversine_km(hop.origin_lat, hop.origin_lng, hop.dest_lat, hop.dest_lng);
            if km.is_finite() && km > 0.0 {
                stats.total_distance_km += positive_km_to_u64(km);
            }
        }

        places.insert(hop.origin_name.clone());
        places.insert(hop.dest_name.clone());

        if !hop.start_date.is_empty()
            && let Some(y) = hop.start_date.get(..4)
        {
            years.push(y);
        }
    }

    stats.cities_visited = places.len();
    // For airport count, count only places referenced by air hops
    let mut airports: HashSet<String> = HashSet::new();
    for hop in hops {
        if hop.travel_type.as_str() == "air" {
            airports.insert(hop.origin_name.clone());
            airports.insert(hop.dest_name.clone());
        }
    }
    stats.airports_visited = airports.len();

    years.sort_unstable();
    stats.first_year = years.first().map(|y| (*y).to_owned());
    stats.last_year = years.last().map(|y| (*y).to_owned());

    stats
}

pub async fn dashboard_page(
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

#[cfg(test)]
mod tests {
    use crate::{
        db,
        db::hops::TravelType,
        server::{create_router, test_helpers::helpers::*},
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

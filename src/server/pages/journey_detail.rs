/// Boat travel detail section component.
mod boat_section;
/// Edit form component for modifying journey details.
mod edit_form;
/// Flight detail section component.
mod flight_section;
/// Rail travel detail section component.
mod rail_section;
/// Ground transport detail section component.
mod transport_section;

use crate::{
    db::hops::{DetailRow, TravelType},
    server::components::{CarrierIcon, NavBar, Shell},
};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use boat_section::BoatSection;
use edit_form::EditForm;
use flight_section::FlightSection;
use leptos::prelude::*;
use rail_section::RailSection;
use schemars::JsonSchema;
use serde::Deserialize;
use transport_section::TransportSection;

#[derive(Deserialize, Default, JsonSchema)]
pub struct JourneyDetailFeedback {
    /// Error message to display after a failed form submission.
    pub error: Option<String>,
    /// Success message to display after a successful form submission.
    pub success: Option<String>,
}

/// Render the journey detail HTML page from a [`DetailRow`] and optional feedback.
pub fn render_page(journey: DetailRow, feedback: JourneyDetailFeedback) -> Response {
    let html = view! {
        <JourneyDetailPage
            error_msg=feedback.error
            success_msg=feedback.success
            journey=journey
        />
    };
    (StatusCode::OK, axum::response::Html(html.to_html())).into_response()
}

fn travel_type_class(tt: &TravelType) -> &'static str {
    match tt {
        TravelType::Air => "air",
        TravelType::Rail => "rail",
        TravelType::Boat => "boat",
        TravelType::Transport => "transport",
    }
}

fn detail_row_view(label: &'static str, value: &str) -> impl IntoView + use<> {
    if value.is_empty() {
        ().into_any()
    } else {
        let v = value.to_owned();
        view! {
            <div class="journey-detail-label">{label}</div>
            <div class="journey-detail-value">{v}</div>
        }
        .into_any()
    }
}

fn timing_row(phase: &'static str, scheduled: &str, actual: &str) -> Option<impl IntoView + use<>> {
    if scheduled.is_empty() && actual.is_empty() {
        return None;
    }
    let s = scheduled.to_owned();
    let a = actual.to_owned();
    Some(view! {
        <tr>
            <td>{phase}</td>
            <td>{s}</td>
            <td>{a}</td>
        </tr>
    })
}

#[component]
fn JourneyDetailPage(
    #[prop(optional_no_strip)] error_msg: Option<String>,
    #[prop(optional_no_strip)] success_msg: Option<String>,
    journey: DetailRow,
) -> impl IntoView {
    let edit_journey = journey.clone();

    let carrier = journey
        .flight_detail
        .as_ref()
        .map(|d| &d.airline)
        .or_else(|| journey.rail_detail.as_ref().map(|d| &d.carrier))
        .or_else(|| journey.boat_detail.as_ref().map(|d| &d.ship_name))
        .or_else(|| journey.transport_detail.as_ref().map(|d| &d.carrier_name))
        .filter(|s| !s.is_empty())
        .cloned()
        .unwrap_or_default();

    let travel_type_str = journey.travel_type.to_string();
    let emoji = journey.travel_type.emoji();
    let badge_class = format!(
        "journey-detail-badge {}",
        travel_type_class(&journey.travel_type)
    );
    let type_label = travel_type_str.clone();

    let dates = if journey.start_date == journey.end_date {
        journey.start_date.clone()
    } else {
        format!("{} \u{2013} {}", journey.start_date, journey.end_date)
    };

    let countries_view = match (&journey.origin_country, &journey.dest_country) {
        (Some(orig), Some(dest)) if orig == dest => {
            let c = orig.clone();
            view! { <p class="journey-detail-countries">{c}</p> }.into_any()
        }
        (Some(orig), Some(dest)) => {
            let text = format!("{orig} \u{2192} {dest}");
            view! { <p class="journey-detail-countries">{text}</p> }.into_any()
        }
        _ => ().into_any(),
    };

    let origin_lat = journey.origin_lat.to_string();
    let origin_lng = journey.origin_lng.to_string();
    let dest_lat = journey.dest_lat.to_string();
    let dest_lng = journey.dest_lng.to_string();

    let detail_section = match journey.travel_type {
        TravelType::Air => journey.flight_detail.map_or_else(
            || ().into_any(),
            |d| view! { <FlightSection detail=d /> }.into_any(),
        ),
        TravelType::Rail => journey.rail_detail.map_or_else(
            || ().into_any(),
            |d| view! { <RailSection detail=d /> }.into_any(),
        ),
        TravelType::Boat => journey.boat_detail.map_or_else(
            || ().into_any(),
            |d| view! { <BoatSection detail=d /> }.into_any(),
        ),
        TravelType::Transport => journey.transport_detail.map_or_else(
            || ().into_any(),
            |d| view! { <TransportSection detail=d /> }.into_any(),
        ),
    };

    view! {
        <Shell title="Journey Detail".to_owned() body_class="journey-detail-layout">
            <NavBar current="" />
            <main class="journey-detail-page">
                <a href="/dashboard" class="journey-detail-back">"\u{2190} Dashboard"</a>

                {error_msg.map(|e| view! {
                    <div class="alert alert-error" role="alert">{e}</div>
                })}
                {success_msg.filter(|v| v == "1").map(|_| view! {
                    <div class="alert alert-success" role="status">"Journey updated successfully!"</div>
                })}

                <header class="journey-detail-header">
                    <h1 class="journey-detail-route">
                        <span>{emoji}</span>
                        <CarrierIcon carrier=carrier travel_type=travel_type_str size=24 />
                        " "
                        {journey.origin_name}
                        " \u{2192} "
                        {journey.dest_name}
                    </h1>
                    <p class="journey-detail-dates">{dates}</p>
                    <span class=badge_class>{type_label}</span>
                    {countries_view}
                </header>

                <div
                    id="journey-map"
                    data-origin-lat=origin_lat
                    data-origin-lng=origin_lng
                    data-dest-lat=dest_lat
                    data-dest-lng=dest_lng
                ></div>

                {detail_section}

                <button
                    class="btn btn-secondary"
                    type="button"
                    onclick="document.getElementById('edit-form').classList.add('open');document.getElementById('edit-backdrop').classList.add('open')"
                >"Edit"</button>

                <EditForm journey=edit_journey />

                <link rel="stylesheet" href="https://unpkg.com/leaflet@1.9.4/dist/leaflet.css"
                    integrity="sha256-p4NxAoJBhIIN+hmNHrzRCf9tD/miZyoHS5obTRR9BMY="
                    crossorigin="" />
                <script src="https://unpkg.com/leaflet@1.9.4/dist/leaflet.js"
                    integrity="sha256-20nQCchB9co0qIjJZRGuk2/Z9VM+kNiyxNV1lvTlZBo="
                    crossorigin=""></script>
                <script type="module" src="/static/journey-map.js"></script>
            </main>
        </Shell>
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        db::{
            self,
            hops::{Create, GetAll, TravelType},
        },
        server::{create_router, test_helpers::*},
    };
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    async fn insert_journey_for_user(pool: &sqlx::SqlitePool, username: &str) -> (String, i64) {
        let cookie = auth_cookie_for_user(pool, username).await;
        let user = db::users::GetByUsername { username }
            .execute(pool)
            .await
            .expect("lookup")
            .expect("user exists");

        let journey = sample_hop(
            TravelType::Air,
            "Dublin",
            "London Heathrow",
            "2024-06-01",
            "2024-06-01",
        );

        Create {
            trip_id: "trip-test",
            user_id: user.id,
            hops: &[journey],
        }
        .execute(pool)
        .await
        .expect("insert journey");

        let rows = GetAll {
            user_id: user.id,
            travel_type_filter: None,
        }
        .execute(pool)
        .await
        .expect("get journeys");

        (cookie, rows[0].id)
    }

    #[tokio::test]
    async fn journey_detail_page_renders_flight() {
        let pool = test_pool().await;
        let (cookie, journey_id) = insert_journey_for_user(&pool, "alice").await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/journeys/{journey_id}"))
                    .header("cookie", &cookie)
                    .header("accept", "text/html")
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_text(response).await;
        assert!(body.contains("Dublin"), "should contain origin name");
        assert!(body.contains("London Heathrow"), "should contain dest name");
        assert!(body.contains("journey-map"), "should contain map div");
    }

    #[tokio::test]
    async fn journey_detail_page_returns_404_for_missing() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/journeys/99999")
                    .header("cookie", &cookie)
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn journey_detail_page_redirects_without_auth() {
        let pool = test_pool().await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/journeys/1")
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::SEE_OTHER,
            "expected 401 or redirect, got {}",
            response.status()
        );
    }

    #[tokio::test]
    async fn journey_detail_page_returns_404_for_other_users_journey() {
        let pool = test_pool().await;
        let (_alice_cookie, journey_id) = insert_journey_for_user(&pool, "alice").await;
        let bob_cookie = auth_cookie_for_user(&pool, "bob").await;

        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/journeys/{journey_id}"))
                    .header("cookie", &bob_cookie)
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        // 404, not 403 — don't leak existence
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}

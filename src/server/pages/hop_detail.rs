/// Boat travel detail section component.
mod boat_section;
/// Edit form component for modifying hop details.
mod edit_form;
/// Flight detail section component.
mod flight_section;
/// Rail travel detail section component.
mod rail_section;
/// Ground transport detail section component.
mod transport_section;

use crate::{
    db::{
        self,
        hops::{DetailRow, TravelType},
    },
    server::{
        AppState,
        components::{NavBar, Shell},
        middleware::AuthUser,
    },
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use boat_section::BoatSection;
use edit_form::EditForm;
use flight_section::FlightSection;
use leptos::prelude::*;
use rail_section::RailSection;
use serde::Deserialize;
use transport_section::TransportSection;

#[derive(Deserialize, Default)]
pub struct HopDetailFeedback {
    pub error: Option<String>,
    pub success: Option<String>,
}

pub async fn page(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i64>,
    Query(feedback): Query<HopDetailFeedback>,
) -> Response {
    match (db::hops::GetById {
        id,
        user_id: auth.user_id,
    })
    .execute(&state.db)
    .await
    {
        Ok(Some(hop)) => {
            let html = view! {
                <HopDetailPage
                    error_msg=feedback.error
                    success_msg=feedback.success
                    hop=hop
                />
            };
            (StatusCode::OK, axum::response::Html(html.to_html())).into_response()
        }
        Ok(None) => super::not_found::page().await,
        Err(err) => {
            tracing::error!(hop_id = id, %err, "failed to fetch hop detail");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::response::Html(super::render_error_page(
                    "500",
                    "Server Error",
                    "Something went wrong loading this hop.",
                    "/dashboard",
                    "Back to Dashboard",
                )),
            )
                .into_response()
        }
    }
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
            <div class="hop-detail-label">{label}</div>
            <div class="hop-detail-value">{v}</div>
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
fn HopDetailPage(
    #[prop(optional_no_strip)] error_msg: Option<String>,
    #[prop(optional_no_strip)] success_msg: Option<String>,
    hop: DetailRow,
) -> impl IntoView {
    let edit_hop = hop.clone();

    let emoji = hop.travel_type.emoji();
    let badge_class = format!("hop-detail-badge {}", travel_type_class(&hop.travel_type));
    let type_label = hop.travel_type.to_string();

    let dates = if hop.start_date == hop.end_date {
        hop.start_date.clone()
    } else {
        format!("{} \u{2013} {}", hop.start_date, hop.end_date)
    };

    let countries_view = match (&hop.origin_country, &hop.dest_country) {
        (Some(orig), Some(dest)) if orig == dest => {
            let c = orig.clone();
            view! { <p class="hop-detail-countries">{c}</p> }.into_any()
        }
        (Some(orig), Some(dest)) => {
            let text = format!("{orig} \u{2192} {dest}");
            view! { <p class="hop-detail-countries">{text}</p> }.into_any()
        }
        _ => ().into_any(),
    };

    let origin_lat = hop.origin_lat.to_string();
    let origin_lng = hop.origin_lng.to_string();
    let dest_lat = hop.dest_lat.to_string();
    let dest_lng = hop.dest_lng.to_string();

    let detail_section = match hop.travel_type {
        TravelType::Air => hop.flight_detail.map_or_else(
            || ().into_any(),
            |d| view! { <FlightSection detail=d /> }.into_any(),
        ),
        TravelType::Rail => hop.rail_detail.map_or_else(
            || ().into_any(),
            |d| view! { <RailSection detail=d /> }.into_any(),
        ),
        TravelType::Boat => hop.boat_detail.map_or_else(
            || ().into_any(),
            |d| view! { <BoatSection detail=d /> }.into_any(),
        ),
        TravelType::Transport => hop.transport_detail.map_or_else(
            || ().into_any(),
            |d| view! { <TransportSection detail=d /> }.into_any(),
        ),
    };

    let map_script = r"
(function() {
    var el = document.getElementById('hop-map');
    if (!el || typeof L === 'undefined') return;
    var oLat = parseFloat(el.dataset.originLat);
    var oLng = parseFloat(el.dataset.originLng);
    var dLat = parseFloat(el.dataset.destLat);
    var dLng = parseFloat(el.dataset.destLng);
    if (isNaN(oLat) || isNaN(dLat)) return;
    var map = L.map('hop-map', { scrollWheelZoom: false });
    L.tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png', {
        attribution: '&copy; OpenStreetMap contributors',
        maxZoom: 18
    }).addTo(map);
    L.marker([oLat, oLng]).addTo(map);
    L.marker([dLat, dLng]).addTo(map);
    L.polyline([[oLat, oLng], [dLat, dLng]], {
        color: '#4a90d9', weight: 3, dashArray: '8 4'
    }).addTo(map);
    map.fitBounds([[oLat, oLng], [dLat, dLng]], { padding: [40, 40] });
})();
";

    view! {
        <Shell title="Hop Detail".to_owned() body_class="hop-detail-layout">
            <NavBar current="" />
            <main class="hop-detail-page">
                <a href="/dashboard" class="hop-detail-back">"\u{2190} Dashboard"</a>

                {error_msg.map(|e| view! {
                    <div class="alert alert-error" role="alert">{e}</div>
                })}
                {success_msg.filter(|v| v == "1").map(|_| view! {
                    <div class="alert alert-success" role="status">"Hop updated successfully!"</div>
                })}

                <header class="hop-detail-header">
                    <h1 class="hop-detail-route">
                        <span>{emoji}</span>
                        " "
                        {hop.origin_name}
                        " \u{2192} "
                        {hop.dest_name}
                    </h1>
                    <p class="hop-detail-dates">{dates}</p>
                    <span class=badge_class>{type_label}</span>
                    {countries_view}
                </header>

                <div
                    id="hop-map"
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

                <EditForm hop=edit_hop />

                <link rel="stylesheet" href="https://unpkg.com/leaflet@1.9.4/dist/leaflet.css"
                    integrity="sha256-p4NxAoJBhIIN+hmNHrzRCf9tD/miZyoHS5obTRR9BMY="
                    crossorigin="" />
                <script src="https://unpkg.com/leaflet@1.9.4/dist/leaflet.js"
                    integrity="sha256-20nQCchB9co0qIjJZRGuk2/Z9VM+kNiyxNV1lvTlZBo="
                    crossorigin=""></script>
                <script inner_html=map_script></script>
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

    async fn insert_hop_for_user(pool: &sqlx::SqlitePool, username: &str) -> (String, i64) {
        let cookie = auth_cookie_for_user(pool, username).await;
        let user = db::users::GetByUsername { username }
            .execute(pool)
            .await
            .expect("lookup")
            .expect("user exists");

        let hop = sample_hop(
            TravelType::Air,
            "Dublin",
            "London Heathrow",
            "2024-06-01",
            "2024-06-01",
        );

        Create {
            trip_id: "trip-test",
            user_id: user.id,
            hops: &[hop],
        }
        .execute(pool)
        .await
        .expect("insert hop");

        let rows = GetAll {
            user_id: user.id,
            travel_type_filter: None,
        }
        .execute(pool)
        .await
        .expect("get hops");

        (cookie, rows[0].id)
    }

    #[tokio::test]
    async fn hop_detail_page_renders_flight() {
        let pool = test_pool().await;
        let (cookie, hop_id) = insert_hop_for_user(&pool, "alice").await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/hop/{hop_id}"))
                    .header("cookie", &cookie)
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_text(response).await;
        assert!(body.contains("Dublin"), "should contain origin name");
        assert!(body.contains("London Heathrow"), "should contain dest name");
        assert!(body.contains("hop-map"), "should contain map div");
    }

    #[tokio::test]
    async fn hop_detail_page_returns_404_for_missing_hop() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/hop/99999")
                    .header("cookie", &cookie)
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn hop_detail_page_redirects_without_auth() {
        let pool = test_pool().await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/hop/1")
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
    async fn hop_detail_page_returns_404_for_other_users_hop() {
        let pool = test_pool().await;
        let (_alice_cookie, hop_id) = insert_hop_for_user(&pool, "alice").await;
        let bob_cookie = auth_cookie_for_user(&pool, "bob").await;

        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/hop/{hop_id}"))
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

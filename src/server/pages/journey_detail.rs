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
    db::{
        self,
        hops::{DetailRow, TravelType},
        status_enrichments,
    },
    server::components::{CarrierIcon, NavBar, Shell},
};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use boat_section::BoatSection;
use edit_form::EditForm;
use flight_section::{FlightSection, FlightVerificationView};
use leptos::prelude::*;
use rail_section::{RailEnrichmentView, RailSection};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;
use transport_section::TransportSection;

#[derive(Deserialize, Default, JsonSchema)]
pub struct JourneyDetailFeedback {
    /// Error message to display after a failed form submission.
    pub error: Option<String>,
    /// Success message to display after a successful form submission.
    pub success: Option<String>,
}

/// Render the journey detail HTML page from a [`DetailRow`] and optional feedback.
pub fn render_page(
    journey: DetailRow,
    feedback: JourneyDetailFeedback,
    enrichment: Option<status_enrichments::Row>,
    attachments: Vec<db::attachments::Row>,
    opensky_verification: Option<status_enrichments::Row>,
) -> Response {
    let html = view! {
        <JourneyDetailPage
            error_msg=feedback.error
            success_msg=feedback.success
            journey=journey
            enrichment=enrichment
            attachments=attachments
            opensky_verification=opensky_verification
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

fn format_miles(miles: f64) -> Option<String> {
    if !miles.is_finite() || miles <= 0.0 {
        return None;
    }
    let rounded = miles.round();
    let text = format!("{rounded:.0}");
    let mut out = String::new();
    for (i, ch) in text.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    Some(format!("{} mi", out.chars().rev().collect::<String>()))
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

fn opensky_verification_view(raw_json: &str) -> Option<FlightVerificationView> {
    let parsed: Value = serde_json::from_str(raw_json).ok()?;
    let est_departure_airport = parsed
        .get("est_departure_airport")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned();
    let est_arrival_airport = parsed
        .get("est_arrival_airport")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned();
    let callsign = parsed
        .get("callsign")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned();
    if est_departure_airport.is_empty() && est_arrival_airport.is_empty() && callsign.is_empty() {
        None
    } else {
        Some(FlightVerificationView {
            est_departure_airport,
            est_arrival_airport,
            callsign,
        })
    }
}

#[component]
fn JourneyDetailPage(
    #[prop(optional_no_strip)] error_msg: Option<String>,
    #[prop(optional_no_strip)] success_msg: Option<String>,
    journey: DetailRow,
    #[prop(optional_no_strip)] enrichment: Option<status_enrichments::Row>,
    attachments: Vec<db::attachments::Row>,
    #[prop(optional_no_strip)] opensky_verification: Option<status_enrichments::Row>,
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

    let cost_view = journey.cost_amount.map(|amount| {
        let currency = journey.cost_currency.clone().unwrap_or_default();
        if currency.is_empty() {
            format!("{amount:.2}")
        } else {
            format!("{amount:.2} {currency}")
        }
    });
    let loyalty_view = journey.loyalty_program.clone().filter(|v| !v.is_empty());
    let miles_view = journey.miles_earned.and_then(format_miles);

    let origin_lat = journey.origin_lat.to_string();
    let origin_lng = journey.origin_lng.to_string();
    let dest_lat = journey.dest_lat.to_string();
    let dest_lng = journey.dest_lng.to_string();

    let flight_verification = if journey.travel_type == TravelType::Air {
        opensky_verification
            .as_ref()
            .and_then(|v| opensky_verification_view(&v.raw_json))
    } else {
        None
    };

    let rail_enrichment = if journey.travel_type == TravelType::Rail {
        enrichment.as_ref().map(|e| RailEnrichmentView {
            dep_platform: e.dep_platform.clone(),
            arr_platform: e.arr_platform.clone(),
            provider: e.provider.clone(),
        })
    } else {
        None
    };

    let detail_section = match journey.travel_type {
        TravelType::Air => journey.flight_detail.map_or_else(
            || ().into_any(),
            |d| view! { <FlightSection detail=d verification=flight_verification /> }.into_any(),
        ),
        TravelType::Rail => journey.rail_detail.map_or_else(
            || ().into_any(),
            |d| view! { <RailSection detail=d enrichment=rail_enrichment /> }.into_any(),
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

    let status_badge_view = enrichment
        .as_ref()
        .filter(|e| !e.status.is_empty())
        .map(|e| {
            let css = format!(
                "status-badge status-{}",
                e.status.to_lowercase().replace(' ', "-")
            );
            let label = match e.delay_minutes {
                Some(mins) if mins > 0 => format!("{} (+{}m)", e.status, mins),
                Some(mins) if mins < 0 => format!("{} ({}m)", e.status, mins),
                _ => e.status.clone(),
            };
            (css, label)
        });

    let verification_badge_view = opensky_verification.as_ref().map(|v| {
        if v.status == "verified" {
            (
                "status-badge status-connected",
                "✓ Route Verified".to_owned(),
            )
        } else {
            (
                "status-badge status-disconnected",
                "Route Unverified".to_owned(),
            )
        }
    });

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
                    {status_badge_view.map(|(css, label)| view! {
                        <span class=css>{label}</span>
                    })}
                    {verification_badge_view.map(|(css, label)| view! {
                        <span class=css>{label}</span>
                    })}
                    {countries_view}
                    {cost_view.map(|text| view! {
                        <p class="journey-detail-cost">{text}</p>
                    })}
                    {loyalty_view.map(|program| view! {
                        <p class="journey-detail-cost">{format!("Loyalty: {program}")}</p>
                    })}
                    {miles_view.map(|text| view! {
                        <p class="journey-detail-cost">{text}</p>
                    })}
                </header>

                <div
                    id="journey-map"
                    data-origin-lat=origin_lat
                    data-origin-lng=origin_lng
                    data-dest-lat=dest_lat
                    data-dest-lng=dest_lng
                ></div>

                {detail_section}

                <AttachmentGallery journey_id=journey.id attachments=attachments />

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

#[component]
fn AttachmentGallery(journey_id: i64, attachments: Vec<db::attachments::Row>) -> impl IntoView {
    let upload_action = format!("/journeys/{journey_id}/attachments");

    view! {
        <section class="attachment-gallery">
            <h2>"Photos"</h2>
            {if attachments.is_empty() {
                view! { <p class="attachment-empty">"No photos attached yet."</p> }.into_any()
            } else {
                let items: Vec<AnyView> = attachments
                    .iter()
                    .map(|att| {
                        let src = format!("/journeys/{journey_id}/attachments/{}", att.id);
                        let delete_action = format!("/journeys/{journey_id}/attachments/{}", att.id);
                        let alt = att.filename.clone();
                        let filename = att.filename.clone();
                        let href = src.clone();
                        let img_src = src;
                        view! {
                            <div class="attachment-card">
                                <a href=href target="_blank">
                                    <img src=img_src alt=alt class="attachment-thumb" loading="lazy" />
                                </a>
                                <div class="attachment-info">
                                    <span class="attachment-name">{filename}</span>
                                    <form method="post" action=delete_action class="attachment-delete-form">
                                        <input type="hidden" name="_method" value="DELETE" />
                                        <button type="submit" class="btn btn-danger btn-sm">"Remove"</button>
                                    </form>
                                </div>
                            </div>
                        }
                        .into_any()
                    })
                    .collect();
                view! { <div class="attachment-grid">{items}</div> }.into_any()
            }}
            <form
                method="post"
                action=upload_action
                enctype="multipart/form-data"
                class="attachment-upload-form"
            >
                <label class="btn btn-secondary">
                    "Add Photos"
                    <input type="file" name="file" accept="image/*" multiple=true hidden=true />
                </label>
            </form>
        </section>
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

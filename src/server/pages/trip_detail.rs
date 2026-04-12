use super::FormFeedback;
use crate::{
    db,
    server::{
        components::{CarrierIcon, NavBar, Shell},
        routes::journeys::JourneyTravelType,
    },
};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use leptos::prelude::*;
use serde_json;

pub(crate) struct TripHopRow {
    pub(crate) id: i64,
    pub(crate) travel_type: String,
    pub(crate) origin_name: String,
    pub(crate) origin_lat: f64,
    pub(crate) origin_lng: f64,
    pub(crate) dest_name: String,
    pub(crate) dest_lat: f64,
    pub(crate) dest_lng: f64,
    pub(crate) start_date: String,
    pub(crate) carrier: Option<String>,
}

impl From<db::hops::SummaryRow> for TripHopRow {
    fn from(row: db::hops::SummaryRow) -> Self {
        Self {
            id: row.id,
            travel_type: row.travel_type,
            origin_name: row.origin_name,
            origin_lat: row.origin_lat,
            origin_lng: row.origin_lng,
            dest_name: row.dest_name,
            dest_lat: row.dest_lat,
            dest_lng: row.dest_lng,
            start_date: row.start_date,
            carrier: row.carrier,
        }
    }
}

pub(crate) struct UnassignedHopRow {
    pub(crate) id: i64,
    pub(crate) travel_type: String,
    pub(crate) origin_name: String,
    pub(crate) dest_name: String,
    pub(crate) start_date: String,
    pub(crate) carrier: Option<String>,
}

impl From<db::hops::SummaryRow> for UnassignedHopRow {
    fn from(row: db::hops::SummaryRow) -> Self {
        Self {
            id: row.id,
            travel_type: row.travel_type,
            origin_name: row.origin_name,
            dest_name: row.dest_name,
            start_date: row.start_date,
            carrier: row.carrier,
        }
    }
}

pub(crate) fn render_page(
    trip: db::trips::Row,
    trip_journeys: Vec<TripHopRow>,
    unassigned_journeys: Vec<UnassignedHopRow>,
    feedback: FormFeedback,
) -> Response {
    let html = view! {
        <TripDetailPage
            trip=trip
            trip_journeys=trip_journeys
            unassigned_journeys=unassigned_journeys
            error=feedback.error
            success=feedback.success
        />
    };
    (StatusCode::OK, axum::response::Html(html.to_html())).into_response()
}

fn travel_type_emoji(travel_type: &str) -> &'static str {
    match travel_type {
        "air" => JourneyTravelType::Air.emoji(),
        "rail" => JourneyTravelType::Rail.emoji(),
        "boat" => JourneyTravelType::Boat.emoji(),
        "transport" => JourneyTravelType::Transport.emoji(),
        _ => "",
    }
}

#[component]
fn TripDetailPage(
    trip: db::trips::Row,
    trip_journeys: Vec<TripHopRow>,
    unassigned_journeys: Vec<UnassignedHopRow>,
    #[prop(optional_no_strip)] error: Option<String>,
    #[prop(optional_no_strip)] success: Option<String>,
) -> impl IntoView {
    let date_range = match (&trip.start_date, &trip.end_date) {
        (Some(start), Some(end)) if start == end => start.clone(),
        (Some(start), Some(end)) => format!("{start} – {end}"),
        (Some(start), None) => start.clone(),
        (None, Some(end)) => end.clone(),
        (None, None) => "No dates yet".to_string(),
    };

    let trip_id = trip.id;
    let trip_id_str = trip_id.to_string();

    let legs_json = serde_json::to_string(
        &trip_journeys
            .iter()
            .map(|h| {
                serde_json::json!({
                    "oLat": h.origin_lat,
                    "oLng": h.origin_lng,
                    "dLat": h.dest_lat,
                    "dLng": h.dest_lng,
                })
            })
            .collect::<Vec<_>>(),
    )
    .unwrap_or_default();

    let remove_entries: Vec<(String, String, String)> = trip_journeys
        .iter()
        .map(|h| {
            (
                h.id.to_string(),
                format!("{} → {}", h.origin_name, h.dest_name),
                h.start_date.clone(),
            )
        })
        .collect();

    view! {
        <Shell title="Trip Detail".to_owned() body_class="journey-detail-layout">
            <NavBar current="trips" />
            <main class="journey-detail-page" data-trip-id=trip_id_str>
                <a href="/trips" class="journey-detail-back">"\u{2190} Trips"</a>

                {error.map(|e| view! {
                    <div class="alert alert-error" role="alert">{e}</div>
                })}
                {success.filter(|v| v == "1").map(|_| view! {
                    <div class="alert alert-success" role="status">"Trip action completed."</div>
                })}

                <header class="page-header">
                    <div class="page-title-row">
                        <h1>{trip.name.clone()}</h1>
                        <button class="btn btn-secondary btn-sm" type="button" data-edit-open>"Edit"</button>
                    </div>
                    <p class="muted">{date_range}</p>
                    <p class="muted">{format!("{} journeys", trip.hop_count)}</p>
                </header>

                <div id="trip-map" data-legs=legs_json></div>

                <section class="content-section">
                    <h2>"Journeys"</h2>
                    {if trip_journeys.is_empty() {
                        view! { <p class="muted">"No journeys assigned yet."</p> }.into_any()
                    } else {
                        view! {
                            <ul class="trip-journey-list">
                                {trip_journeys.into_iter().map(|row| {
                                    let journey_id = row.id;
                                    let emoji = travel_type_emoji(&row.travel_type);
                                    let carrier = row.carrier.unwrap_or_default();
                                    let travel_type = row.travel_type.clone();
                                    view! {
                                        <li class="trip-journey-item">
                                            <a href={format!("/journeys/{journey_id}")} class="trip-journey-link">
                                                <span class="trip-journey-type">{emoji}</span>
                                                <CarrierIcon carrier=carrier travel_type=travel_type size=20 />
                                                <span class="trip-journey-route">{format!("{} → {}", row.origin_name, row.dest_name)}</span>
                                                <span class="trip-journey-date muted">{row.start_date}</span>
                                            </a>
                                        </li>
                                    }
                                }).collect::<Vec<_>>()}
                            </ul>
                        }.into_any()
                    }}
                </section>

                <div id="edit-backdrop" class="edit-panel-backdrop"></div>
                <section id="edit-form" class="journey-edit-form">
                    <h3>
                        "Edit Trip"
                        <button class="edit-panel-close" type="button" data-edit-close>"\u{2715}"</button>
                    </h3>

                    <form method="post" action={format!("/trips/{trip_id}")}>
                        <div class="form-group">
                            <label for="trip-name">"Trip name"</label>
                            <input id="trip-name" name="name" type="text" value={trip.name} required />
                        </div>
                        <button class="btn btn-primary" type="submit">"Rename Trip"</button>
                    </form>

                    <hr class="trip-edit-hr" />

                    <h4>"Add journey"</h4>
                    <form method="post" action={format!("/trips/{trip_id}/journeys")}>
                        <div class="form-group">
                            <label for="journey-id">"Unassigned journey"</label>
                            <select id="journey-id" name="hop_id" required>
                                <option value="">"Select journey"</option>
                                {unassigned_journeys.into_iter().map(|row| {
                                    let journey_id = row.id;
                                    let emoji = travel_type_emoji(&row.travel_type);
                                    let carrier_str = row.carrier.as_deref().unwrap_or("");
                                    let carrier_suffix = if carrier_str.is_empty() {
                                        String::new()
                                    } else {
                                        format!(" [{carrier_str}]")
                                    };
                                    let label = format!("{emoji} {} — {} → {} (#{journey_id}){carrier_suffix}", row.start_date, row.origin_name, row.dest_name);
                                    view! {
                                        <option value={journey_id.to_string()}>{label}</option>
                                    }
                                }).collect::<Vec<_>>()}
                            </select>
                        </div>
                        <button class="btn btn-primary" type="submit">"Assign Journey"</button>
                    </form>

                    <hr class="trip-edit-hr" />

                    <h4>"Remove journeys"</h4>
                    {if remove_entries.is_empty() {
                        view! { <p class="muted">"No journeys to remove."</p> }.into_any()
                    } else {
                        view! {
                            <ul class="trip-remove-list">
                                {remove_entries.into_iter().map(|(jid, route, date)| view! {
                                    <li class="trip-remove-item">
                                        <span>{route}" "{date}</span>
                                        <button class="btn btn-danger btn-sm" type="button"
                                            data-remove-journey=jid
                                        >"Remove"</button>
                                    </li>
                                }).collect::<Vec<_>>()}
                            </ul>
                        }.into_any()
                    }}

                    <hr class="trip-edit-hr" />

                    <button
                        class="btn btn-danger btn-block"
                        type="button"
                        data-delete-trip
                    >"Delete Trip"</button>
                </section>

                <link rel="stylesheet" href="https://unpkg.com/leaflet@1.9.4/dist/leaflet.css"
                    integrity="sha256-p4NxAoJBhIIN+hmNHrzRCf9tD/miZyoHS5obTRR9BMY="
                    crossorigin="" />
                <script src="https://unpkg.com/leaflet@1.9.4/dist/leaflet.js"
                    integrity="sha256-20nQCchB9co0qIjJZRGuk2/Z9VM+kNiyxNV1lvTlZBo="
                    crossorigin=""></script>
                <script type="module" src="/static/trip-map.js"></script>
                <script src="/static/edit-panel.js" defer></script>
                <script src="/static/trip-detail.js" defer></script>
            </main>
        </Shell>
    }
}

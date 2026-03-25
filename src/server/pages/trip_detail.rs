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

pub(crate) struct TripHopRow {
    pub(crate) id: i64,
    pub(crate) travel_type: String,
    pub(crate) origin_name: String,
    pub(crate) dest_name: String,
    pub(crate) start_date: String,
    pub(crate) carrier: Option<String>,
}

impl From<db::hops::SummaryRow> for TripHopRow {
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
    let delete_click = format!(
        "if (confirm('Delete this trip? Journeys will remain but be unassigned.')) {{ fetch('/trips/{trip_id}', {{ method: 'DELETE' }}).then(function(r) {{ if (r.ok) window.location = '/trips'; else window.location.reload(); }}); }}"
    );

    view! {
        <Shell title="Trip Detail".to_owned()>
            <NavBar current="trips" />
            <main class="container-wide">
                <a href="/trips" class="journey-detail-back">"← Trips"</a>

                {error.map(|e| view! {
                    <div class="alert alert-error" role="alert">{e}</div>
                })}
                {success.filter(|v| v == "1").map(|_| view! {
                    <div class="alert alert-success" role="status">"Trip action completed."</div>
                })}

                <section class="card">
                    <h1>{trip.name.clone()}</h1>
                    <p class="muted">{date_range}</p>
                    <p class="muted">{format!("{} journeys", trip.hop_count)}</p>

                    <form method="post" action={format!("/trips/{trip_id}")}>
                        <div class="form-group">
                            <label for="trip-name">"Trip name"</label>
                            <input id="trip-name" name="name" type="text" value={trip.name} required />
                        </div>
                        <button class="btn btn-secondary" type="submit">"Rename Trip"</button>
                    </form>

                    <button
                        class="btn btn-danger"
                        type="button"
                        style="margin-top:1rem;"
                        onclick=delete_click
                    >"Delete Trip"</button>
                </section>

                <section class="card" style="margin-top:1rem;">
                    <h2>"Journeys in this trip"</h2>
                    {if trip_journeys.is_empty() {
                        view! { <p class="muted">"No journeys assigned yet."</p> }.into_any()
                    } else {
                        view! {
                            <ul class="stats-top-list">
                                {trip_journeys.into_iter().map(|row| {
                                    let journey_id = row.id;
                                    let emoji = travel_type_emoji(&row.travel_type);
                                    let carrier = row.carrier.unwrap_or_default();
                                    let travel_type = row.travel_type.clone();
                                    view! {
                                        <li class="stats-top-item" style="display:flex;justify-content:space-between;align-items:center;gap:1rem;">
                                            <div>
                                                <span>{emoji}</span>
                                                <CarrierIcon carrier=carrier travel_type=travel_type size=20 />
                                                " "
                                                <a href={format!("/journeys/{journey_id}")}>{format!("{} → {}", row.origin_name, row.dest_name)}</a>
                                                " "
                                                <span class="muted">{row.start_date}</span>
                                            </div>
                                            <button class="btn btn-secondary" type="button"
                                                onclick={format!(
                                                    "fetch('/trips/{trip_id}/journeys/{journey_id}', {{ method: 'DELETE' }}).then(function(r) {{ if (r.ok) window.location.reload(); }});"
                                                )}>
                                                "Remove"
                                            </button>
                                        </li>
                                    }
                                }).collect::<Vec<_>>()}
                            </ul>
                        }.into_any()
                    }}
                </section>

                <section class="card" style="margin-top:1rem;">
                    <h2>"Add journey to trip"</h2>
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
                </section>
            </main>
        </Shell>
    }
}

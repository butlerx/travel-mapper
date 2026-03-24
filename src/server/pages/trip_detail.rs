use super::FormFeedback;
use crate::{
    db,
    server::{
        AppState,
        components::{NavBar, Shell},
        extractors::AuthUser,
        routes::hops::HopTravelType,
    },
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use leptos::prelude::*;

struct TripHopRow {
    id: i64,
    travel_type: String,
    origin_name: String,
    dest_name: String,
    start_date: String,
}

struct UnassignedHopRow {
    id: i64,
    travel_type: String,
    origin_name: String,
    dest_name: String,
    start_date: String,
}

pub async fn page(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i64>,
    Query(feedback): Query<FormFeedback>,
) -> Response {
    let Some(trip) = (db::trips::GetById {
        id,
        user_id: auth.user_id,
    })
    .execute(&state.db)
    .await
    .unwrap_or_default() else {
        return super::not_found::page().await;
    };

    let trip_hops = sqlx::query!(
        r#"SELECT
               id as "id!: i64",
               travel_type as "travel_type!: String",
               origin_name as "origin_name!: String",
               dest_name as "dest_name!: String",
               start_date as "start_date!: String"
           FROM hops
           WHERE user_id = ? AND user_trip_id = ?
           ORDER BY start_date ASC, id ASC"#,
        auth.user_id,
        id,
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default()
    .into_iter()
    .map(|row| TripHopRow {
        id: row.id,
        travel_type: row.travel_type,
        origin_name: row.origin_name,
        dest_name: row.dest_name,
        start_date: row.start_date,
    })
    .collect();

    let unassigned_hops = sqlx::query!(
        r#"SELECT
               id as "id!: i64",
               travel_type as "travel_type!: String",
               origin_name as "origin_name!: String",
               dest_name as "dest_name!: String",
               start_date as "start_date!: String"
           FROM hops
           WHERE user_id = ? AND user_trip_id IS NULL
           ORDER BY start_date DESC, id DESC
           LIMIT 200"#,
        auth.user_id,
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default()
    .into_iter()
    .map(|row| UnassignedHopRow {
        id: row.id,
        travel_type: row.travel_type,
        origin_name: row.origin_name,
        dest_name: row.dest_name,
        start_date: row.start_date,
    })
    .collect();

    let html = view! {
        <TripDetailPage
            trip=trip
            trip_hops=trip_hops
            unassigned_hops=unassigned_hops
            error=feedback.error
            success=feedback.success
        />
    };
    (StatusCode::OK, axum::response::Html(html.to_html())).into_response()
}

fn travel_type_emoji(travel_type: &str) -> &'static str {
    match travel_type {
        "air" => HopTravelType::Air.emoji(),
        "rail" => HopTravelType::Rail.emoji(),
        "boat" => HopTravelType::Boat.emoji(),
        "transport" => HopTravelType::Transport.emoji(),
        _ => "",
    }
}

#[component]
fn TripDetailPage(
    trip: db::trips::Row,
    trip_hops: Vec<TripHopRow>,
    unassigned_hops: Vec<UnassignedHopRow>,
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
                <a href="/trips" class="hop-detail-back">"← Trips"</a>

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
                    {if trip_hops.is_empty() {
                        view! { <p class="muted">"No journeys assigned yet."</p> }.into_any()
                    } else {
                        view! {
                            <ul class="stats-top-list">
                                {trip_hops.into_iter().map(|row| {
                                    let hop_id = row.id;
                                    let emoji = travel_type_emoji(&row.travel_type);
                                    view! {
                                        <li class="stats-top-item" style="display:flex;justify-content:space-between;align-items:center;gap:1rem;">
                                            <div>
                                                <span>{emoji}</span>
                                                " "
                                                <a href={format!("/journey/{hop_id}")}>{format!("{} → {}", row.origin_name, row.dest_name)}</a>
                                                " "
                                                <span class="muted">{row.start_date}</span>
                                            </div>
                                            <form method="post" action={format!("/trips/{trip_id}/journeys/{hop_id}")}>
                                                <button class="btn btn-secondary" type="submit">"Remove"</button>
                                            </form>
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
                            <label for="hop-id">"Unassigned journey"</label>
                            <select id="hop-id" name="hop_id" required>
                                <option value="">"Select journey"</option>
                                {unassigned_hops.into_iter().map(|row| {
                                    let hop_id = row.id;
                                    let emoji = travel_type_emoji(&row.travel_type);
                                    let label = format!("{emoji} {} — {} → {} (#{hop_id})", row.start_date, row.origin_name, row.dest_name);
                                    view! {
                                        <option value={hop_id.to_string()}>{label}</option>
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

use super::FormFeedback;
use crate::{
    db,
    server::components::{NavBar, Shell},
};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use leptos::prelude::*;

pub(crate) fn render_page(trips: Vec<db::trips::Row>, feedback: FormFeedback) -> Response {
    let html = view! {
        <TripsPage trips=trips error=feedback.error success=feedback.success />
    };
    (StatusCode::OK, axum::response::Html(html.to_html())).into_response()
}

#[component]
fn TripsPage(
    trips: Vec<db::trips::Row>,
    #[prop(optional_no_strip)] error: Option<String>,
    #[prop(optional_no_strip)] success: Option<String>,
) -> impl IntoView {
    view! {
        <Shell title="Trips".to_owned()>
            <NavBar current="trips" />
            <main class="container-wide">
                {error.map(|e| view! {
                    <div class="alert alert-error" role="alert">{e}</div>
                })}
                {success.filter(|v| v == "1").map(|_| view! {
                    <div class="alert alert-success" role="status">"Trip action completed."</div>
                })}

                <section class="card">
                    <h2>"Create Trip"</h2>
                    <form method="post" action="/trips">
                        <div class="form-group">
                            <label for="trip-name">"Trip Name"</label>
                            <input id="trip-name" name="name" type="text" required placeholder="Europe 2024" />
                        </div>
                        <button class="btn btn-primary" type="submit">"Create"</button>
                    </form>
                </section>

                <section class="card" style="margin-top:1rem;">
                    <h2>"Auto Group"</h2>
                    <form method="post" action="/trips/auto-group">
                        <div class="form-group">
                            <label for="gap-days">"Gap days"</label>
                            <input id="gap-days" name="gap_days" type="number" min="0" value="3" />
                        </div>
                        <button class="btn btn-secondary" type="submit">"Auto-Group Unassigned Journeys"</button>
                    </form>
                </section>

                <section class="card" style="margin-top:1rem;">
                    <h2>"Trips"</h2>
                    {if trips.is_empty() {
                        view! {
                            <p class="muted">"No trips yet. Create one or auto-group your journeys."</p>
                        }.into_any()
                    } else {
                        view! {
                            <div class="data-card-list">
                                {trips.into_iter().map(|trip| {
                                    let range = match (&trip.start_date, &trip.end_date) {
                                        (Some(start), Some(end)) if start == end => start.clone(),
                                        (Some(start), Some(end)) => format!("{start} – {end}"),
                                        (Some(start), None) => start.clone(),
                                        (None, Some(end)) => end.clone(),
                                        (None, None) => "No dates yet".to_string(),
                                    };
                                    view! {
                                        <a href={format!("/trips/{}", trip.id)} class="journey-card-link">
                                            <div class="data-card journey-card">
                                                <div class="journey-card-route">{trip.name}</div>
                                                <div class="journey-card-meta">
                                                    <span class="journey-card-date">{range}</span>
                                                    <span class="journey-card-badge">{format!("{} journeys", trip.hop_count)}</span>
                                                </div>
                                            </div>
                                        </a>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        }.into_any()
                    }}
                </section>
            </main>
        </Shell>
    }
}

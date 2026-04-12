use super::FormFeedback;
use crate::server::{
    components::{NavBar, Shell},
    routes::trips::{TripResponse, TripSort},
};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use leptos::prelude::*;

pub(crate) fn render_page(
    trips: Vec<TripResponse>,
    sort: TripSort,
    feedback: FormFeedback,
) -> Response {
    let html = view! {
        <TripsPage trips=trips sort=sort error=feedback.error success=feedback.success />
    };
    (StatusCode::OK, axum::response::Html(html.to_html())).into_response()
}

#[component]
fn TripsPage(
    trips: Vec<TripResponse>,
    sort: TripSort,
    #[prop(optional_no_strip)] error: Option<String>,
    #[prop(optional_no_strip)] success: Option<String>,
) -> impl IntoView {
    let trip_count = trips.len();

    let sort_options: Vec<AnyView> = TripSort::all()
        .iter()
        .map(|&variant| {
            let value = variant.as_str();
            let label = variant.label();
            let selected = variant == sort;
            view! { <option value=value selected=selected>{label}</option> }.into_any()
        })
        .collect();

    view! {
        <Shell title="Trips".to_owned()>
            <NavBar current="trips" />
            <main class="data-page">
                <div class="data-page-header">
                    <h1>"Trips"</h1>
                    <span class="data-page-count">{format!("{trip_count} trips")}</span>
                    <form class="journey-controls" method="get" action="/trips">
                        <div class="journey-control">
                            <label for="sort-select" class="control-label">"Sort"</label>
                            <select id="sort-select" name="sort" data-auto-submit>
                                {sort_options}
                            </select>
                        </div>
                        <button class="btn btn-secondary btn-sm" type="button" data-edit-open>"Actions"</button>
                    </form>
                </div>

                {error.map(|e| view! {
                    <div class="alert alert-error" role="alert">{e}</div>
                })}
                {success.filter(|v| v == "1").map(|_| view! {
                    <div class="alert alert-success" role="status">"Trip action completed."</div>
                })}

                {if trips.is_empty() {
                    view! {
                        <p class="muted">"No trips yet. Open Actions to create one or auto-group your journeys."</p>
                    }.into_any()
                } else {
                    let mut groups: Vec<(String, Vec<AnyView>)> = Vec::new();
                    for trip in &trips {
                        let key = sort.group_key(trip);
                        let range = match (&trip.start_date, &trip.end_date) {
                            (Some(start), Some(end)) if start == end => start.clone(),
                            (Some(start), Some(end)) => format!("{start} \u{2013} {end}"),
                            (Some(start), None) => start.clone(),
                            (None, Some(end)) => end.clone(),
                            (None, None) => "No dates yet".to_string(),
                        };
                        let card = view! {
                            <a href={format!("/trips/{}", trip.id)} class="journey-card-link">
                                <div class="data-card journey-card">
                                    <div class="journey-card-route">{trip.name.clone()}</div>
                                    <div class="journey-card-meta">
                                        <span class="journey-card-date">{range}</span>
                                        <span class="journey-card-badge">{format!("{} journeys", trip.hop_count)}</span>
                                    </div>
                                </div>
                            </a>
                        }.into_any();
                        if groups.last().is_some_and(|(k, _)| *k == key) {
                            groups.last_mut().unwrap().1.push(card);
                        } else {
                            groups.push((key, vec![card]));
                        }
                    }
                    let sections: Vec<AnyView> = groups
                        .into_iter()
                        .map(|(heading_key, cards)| {
                            let count = cards.len();
                            let heading = format!("{heading_key} ({count})");
                            view! {
                                <section class="journey-year-section">
                                    <h2 class="journey-year-heading">{heading}</h2>
                                    <div class="data-card-list">{cards}</div>
                                </section>
                            }
                            .into_any()
                        })
                        .collect();
                    view! { <div>{sections}</div> }.into_any()
                }}

                <div id="edit-backdrop" class="edit-panel-backdrop"></div>
                <section id="edit-form" class="journey-edit-form">
                    <h3>
                        "Actions"
                        <button class="edit-panel-close" type="button" data-edit-close>"\u{2715}"</button>
                    </h3>

                    <h4>"Create Trip"</h4>
                    <form method="post" action="/trips">
                        <div class="form-group">
                            <label for="trip-name">"Trip name"</label>
                            <input id="trip-name" name="name" type="text" required placeholder="Europe 2024" />
                        </div>
                        <button class="btn btn-primary" type="submit">"Create"</button>
                    </form>

                    <hr class="trip-edit-hr" />

                    <h4>"Auto Group"</h4>
                    <p class="muted">"Automatically group unassigned journeys into trips based on date proximity."</p>
                    <form method="post" action="/trips/auto-group">
                        <div class="form-group">
                            <label for="gap-days">"Gap days"</label>
                            <input id="gap-days" name="gap_days" type="number" min="0" value="3" />
                        </div>
                        <button class="btn btn-secondary" type="submit">"Auto-Group Unassigned Journeys"</button>
                    </form>
                </section>

                <script src="/static/auto-submit.js" defer></script>
                <script src="/static/edit-panel.js" defer></script>
            </main>
        </Shell>
    }
}

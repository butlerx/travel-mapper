use crate::server::{
    AppState,
    components::{NavBar, Shell},
    middleware::AuthUser,
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use leptos::prelude::*;
use serde::Deserialize;

#[derive(Deserialize, Default)]
pub struct AddFlightFeedback {
    pub error: Option<String>,
    pub success: Option<String>,
}

pub async fn page(
    State(_state): State<AppState>,
    _auth: AuthUser,
    Query(feedback): Query<AddFlightFeedback>,
) -> Response {
    let html = view! {
        <AddFlight error=feedback.error success=feedback.success />
    };
    (StatusCode::OK, axum::response::Html(html.to_html())).into_response()
}

#[component]
fn AddFlight(
    #[prop(optional_no_strip)] error: Option<String>,
    #[prop(optional_no_strip)] success: Option<String>,
) -> impl IntoView {
    view! {
        <Shell title="Add Flight".to_owned()>
            <NavBar current="add-flight" />
            <main class="container">
                {error.map(|e| view! {
                    <div class="alert alert-error" role="alert">{e}</div>
                })}
                {success.filter(|v| v == "1").map(|_| view! {
                    <div class="alert alert-success" role="status">"Flight added successfully!"</div>
                })}

                <section class="card">
                    <h2>"Add Flight"</h2>
                    <form method="post" action="/hops">
                        <div class="form-group">
                            <label for="origin">"Origin (IATA code)"</label>
                            <input
                                type="text"
                                id="origin"
                                name="origin"
                                required
                                maxlength="4"
                                placeholder="LHR"
                                style="text-transform: uppercase"
                            />
                        </div>
                        <div class="form-group">
                            <label for="destination">"Destination (IATA code)"</label>
                            <input
                                type="text"
                                id="destination"
                                name="destination"
                                required
                                maxlength="4"
                                placeholder="JFK"
                                style="text-transform: uppercase"
                            />
                        </div>
                        <div class="form-group">
                            <label for="date">"Date"</label>
                            <input type="date" id="date" name="date" required />
                        </div>
                        <div class="form-group">
                            <label for="airline">"Airline"</label>
                            <input type="text" id="airline" name="airline" placeholder="British Airways" />
                        </div>
                        <div class="form-group">
                            <label for="flight_number">"Flight Number"</label>
                            <input type="text" id="flight_number" name="flight_number" placeholder="BA117" />
                        </div>
                        <div class="form-group">
                            <label for="aircraft_type">"Aircraft Type"</label>
                            <input type="text" id="aircraft_type" name="aircraft_type" placeholder="Boeing 777-300ER" />
                        </div>
                        <div class="form-group">
                            <label for="cabin_class">"Cabin Class"</label>
                            <input type="text" id="cabin_class" name="cabin_class" placeholder="Economy" />
                        </div>
                        <div class="form-group">
                            <label for="seat">"Seat"</label>
                            <input type="text" id="seat" name="seat" placeholder="14A" />
                        </div>
                        <div class="form-group">
                            <label for="pnr">"Booking Reference (PNR)"</label>
                            <input type="text" id="pnr" name="pnr" placeholder="ABC123" />
                        </div>
                        <button class="btn btn-primary btn-full" type="submit">"Add Flight"</button>
                    </form>
                </section>
            </main>
        </Shell>
    }
}

use crate::server::{
    AppState,
    components::{NavBar, Shell},
    extractors::AuthUser,
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use leptos::prelude::*;
use serde::Deserialize;

#[derive(Deserialize, Default)]
pub struct AddHopFeedback {
    pub error: Option<String>,
    pub success: Option<String>,
}

pub async fn page(
    State(_state): State<AppState>,
    _auth: AuthUser,
    Query(feedback): Query<AddHopFeedback>,
) -> Response {
    let html = view! {
        <AddHop error=feedback.error success=feedback.success />
    };
    (StatusCode::OK, axum::response::Html(html.to_html())).into_response()
}

#[component]
fn AddHop(
    #[prop(optional_no_strip)] error: Option<String>,
    #[prop(optional_no_strip)] success: Option<String>,
) -> impl IntoView {
    view! {
        <Shell title="Add Journey".to_owned()>
            <NavBar current="add-journey" />
            <main class="container">
                {error.map(|e| view! {
                    <div class="alert alert-error" role="alert">{e}</div>
                })}
                {success.filter(|v| v == "1").map(|_| view! {
                    <div class="alert alert-success" role="status">"Journey added successfully!"</div>
                })}

                <section class="card">
                    <h2>"Add Journey"</h2>
                    <form method="post" action="/journeys">
                        <div class="form-group">
                            <label for="travel_type">"Travel Type"</label>
                            <select id="travel_type" name="travel_type">
                                <option value="air" selected>"✈️ Flight"</option>
                                <option value="rail">"🚆 Rail"</option>
                                <option value="boat">"🚢 Boat"</option>
                                <option value="transport">"🚗 Transport"</option>
                            </select>
                        </div>

                        <div class="form-group">
                            <label for="origin" id="origin-label">"Origin"</label>
                            <input type="text" id="origin" name="origin" required placeholder="LHR" />
                        </div>
                        <div class="form-group">
                            <label for="destination" id="destination-label">"Destination"</label>
                            <input type="text" id="destination" name="destination" required placeholder="JFK" />
                        </div>
                        <div class="form-group">
                            <label for="date">"Date"</label>
                            <input type="date" id="date" name="date" required />
                        </div>

                        <div id="fields-air" class="type-fields">
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
                        </div>

                        <div id="fields-rail" class="type-fields" style="display:none">
                            <div class="form-group">
                                <label for="rail_carrier">"Carrier"</label>
                                <input type="text" id="rail_carrier" name="rail_carrier" placeholder="Eurostar" />
                            </div>
                            <div class="form-group">
                                <label for="train_number">"Train Number"</label>
                                <input type="text" id="train_number" name="train_number" placeholder="9024" />
                            </div>
                            <div class="form-group">
                                <label for="service_class">"Class"</label>
                                <input type="text" id="service_class" name="service_class" placeholder="Standard Premier" />
                            </div>
                            <div class="form-group">
                                <label for="coach_number">"Coach"</label>
                                <input type="text" id="coach_number" name="coach_number" placeholder="6" />
                            </div>
                            <div class="form-group">
                                <label for="rail_seats">"Seats"</label>
                                <input type="text" id="rail_seats" name="rail_seats" placeholder="23A, 23B" />
                            </div>
                            <div class="form-group">
                                <label for="rail_confirmation">"Confirmation Number"</label>
                                <input type="text" id="rail_confirmation" name="rail_confirmation" placeholder="ABC123" />
                            </div>
                            <div class="form-group">
                                <label for="rail_booking_site">"Booking Site"</label>
                                <input type="text" id="rail_booking_site" name="rail_booking_site" placeholder="eurostar.com" />
                            </div>
                            <div class="form-group">
                                <label for="rail_notes">"Notes"</label>
                                <textarea id="rail_notes" name="rail_notes" rows="3"></textarea>
                            </div>
                        </div>

                        <div id="fields-boat" class="type-fields" style="display:none">
                            <div class="form-group">
                                <label for="ship_name">"Ship Name"</label>
                                <input type="text" id="ship_name" name="ship_name" placeholder="MS Nordnorge" />
                            </div>
                            <div class="form-group">
                                <label for="cabin_type">"Cabin Type"</label>
                                <input type="text" id="cabin_type" name="cabin_type" placeholder="Outside" />
                            </div>
                            <div class="form-group">
                                <label for="cabin_number">"Cabin Number"</label>
                                <input type="text" id="cabin_number" name="cabin_number" placeholder="412" />
                            </div>
                            <div class="form-group">
                                <label for="boat_confirmation">"Confirmation Number"</label>
                                <input type="text" id="boat_confirmation" name="boat_confirmation" placeholder="ABC123" />
                            </div>
                            <div class="form-group">
                                <label for="boat_booking_site">"Booking Site"</label>
                                <input type="text" id="boat_booking_site" name="boat_booking_site" placeholder="hurtigruten.com" />
                            </div>
                            <div class="form-group">
                                <label for="boat_notes">"Notes"</label>
                                <textarea id="boat_notes" name="boat_notes" rows="3"></textarea>
                            </div>
                        </div>

                        <div id="fields-transport" class="type-fields" style="display:none">
                            <div class="form-group">
                                <label for="transport_carrier">"Carrier"</label>
                                <input type="text" id="transport_carrier" name="transport_carrier" placeholder="Greyhound" />
                            </div>
                            <div class="form-group">
                                <label for="vehicle_description">"Vehicle Description"</label>
                                <input type="text" id="vehicle_description" name="vehicle_description" placeholder="Coach bus" />
                            </div>
                            <div class="form-group">
                                <label for="transport_confirmation">"Confirmation Number"</label>
                                <input type="text" id="transport_confirmation" name="transport_confirmation" placeholder="ABC123" />
                            </div>
                            <div class="form-group">
                                <label for="transport_notes">"Notes"</label>
                                <textarea id="transport_notes" name="transport_notes" rows="3"></textarea>
                            </div>
                        </div>

                        <div class="form-row">
                            <div class="form-group">
                                <label for="cost_amount">"Cost"</label>
                                <input type="number" id="cost_amount" name="cost_amount" step="0.01" min="0" placeholder="0.00" />
                            </div>
                            <div class="form-group">
                                <label for="cost_currency">"Currency"</label>
                                <input type="text" id="cost_currency" name="cost_currency" placeholder="EUR" maxlength="3" style="text-transform:uppercase" />
                            </div>
                        </div>

                        <div class="form-row">
                            <div class="form-group">
                                <label for="loyalty_program">"Loyalty Program"</label>
                                <input type="text" id="loyalty_program" name="loyalty_program" placeholder="e.g. Delta SkyMiles" />
                            </div>
                            <div class="form-group">
                                <label for="miles_earned">"Miles Earned"</label>
                                <input type="number" id="miles_earned" name="miles_earned" step="1" placeholder="auto-calculated for flights" />
                            </div>
                        </div>

                        <button class="btn btn-primary btn-full" type="submit">"Add Journey"</button>
                    </form>
                </section>

                <script type="module" src="/static/add-journey.js"></script>
            </main>
        </Shell>
    }
}

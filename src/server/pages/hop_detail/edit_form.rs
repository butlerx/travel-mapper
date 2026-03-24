use crate::db::hops::{DetailRow, TravelType};
use leptos::prelude::*;

fn form_field(
    id: &'static str,
    name: &'static str,
    label: &'static str,
    value: &str,
    input_type: &'static str,
) -> impl IntoView + use<> {
    let v = value.to_owned();
    view! {
        <div class="form-group">
            <label for=id>{label}</label>
            <input type=input_type id=id name=name value=v />
        </div>
    }
}

fn hidden_field(name: &'static str, value: String) -> impl IntoView + use<> {
    view! { <input type="hidden" name=name value=value /> }
}

#[component]
fn FlightEditFields(detail: crate::db::hops::FullFlightDetail) -> impl IntoView {
    view! {
        <fieldset class="edit-form-fieldset">
            <legend>"Flight Details"</legend>
            {form_field("edit-airline", "airline", "Airline", &detail.airline, "text")}
            {form_field("edit-flight-number", "flight_number", "Flight Number", &detail.flight_number, "text")}
            {form_field("edit-aircraft-type", "aircraft_type", "Aircraft Type", &detail.aircraft_type, "text")}
            {form_field("edit-tail-number", "tail_number", "Tail Number", &detail.tail_number, "text")}
            {form_field("edit-dep-terminal", "dep_terminal", "Departure Terminal", &detail.dep_terminal, "text")}
            {form_field("edit-dep-gate", "dep_gate", "Departure Gate", &detail.dep_gate, "text")}
            {form_field("edit-arr-terminal", "arr_terminal", "Arrival Terminal", &detail.arr_terminal, "text")}
            {form_field("edit-arr-gate", "arr_gate", "Arrival Gate", &detail.arr_gate, "text")}
            {form_field("edit-cabin-class", "cabin_class", "Cabin Class", &detail.cabin_class, "text")}
            {form_field("edit-seat", "seat", "Seat", &detail.seat, "text")}
            {form_field("edit-seat-type", "seat_type", "Seat Type", &detail.seat_type, "text")}
            {form_field("edit-pnr", "pnr", "Booking Reference", &detail.pnr, "text")}
            {form_field("edit-flight-reason", "flight_reason", "Flight Reason", &detail.flight_reason, "text")}
            {form_field("edit-flight-notes", "flight_notes", "Notes", &detail.notes, "text")}
            {form_field("edit-gate-dep-sched", "gate_dep_scheduled", "Gate Dep. Scheduled", &detail.gate_dep_scheduled, "text")}
            {form_field("edit-gate-dep-actual", "gate_dep_actual", "Gate Dep. Actual", &detail.gate_dep_actual, "text")}
            {form_field("edit-takeoff-sched", "takeoff_scheduled", "Takeoff Scheduled", &detail.takeoff_scheduled, "text")}
            {form_field("edit-takeoff-actual", "takeoff_actual", "Takeoff Actual", &detail.takeoff_actual, "text")}
            {form_field("edit-landing-sched", "landing_scheduled", "Landing Scheduled", &detail.landing_scheduled, "text")}
            {form_field("edit-landing-actual", "landing_actual", "Landing Actual", &detail.landing_actual, "text")}
            {form_field("edit-gate-arr-sched", "gate_arr_scheduled", "Gate Arr. Scheduled", &detail.gate_arr_scheduled, "text")}
            {form_field("edit-gate-arr-actual", "gate_arr_actual", "Gate Arr. Actual", &detail.gate_arr_actual, "text")}
            {form_field("edit-diverted-to", "diverted_to", "Diverted To", &detail.diverted_to, "text")}
        </fieldset>
    }
}

#[component]
fn RailEditFields(detail: crate::db::hops::RailDetail) -> impl IntoView {
    view! {
        <fieldset class="edit-form-fieldset">
            <legend>"Rail Details"</legend>
            {form_field("edit-rail-carrier", "rail_carrier", "Carrier", &detail.carrier, "text")}
            {form_field("edit-train-number", "train_number", "Train Number", &detail.train_number, "text")}
            {form_field("edit-service-class", "service_class", "Service Class", &detail.service_class, "text")}
            {form_field("edit-coach-number", "coach_number", "Coach Number", &detail.coach_number, "text")}
            {form_field("edit-rail-seats", "rail_seats", "Seats", &detail.seats, "text")}
            {form_field("edit-rail-confirmation", "rail_confirmation", "Confirmation", &detail.confirmation_num, "text")}
            {form_field("edit-rail-booking-site", "rail_booking_site", "Booking Site", &detail.booking_site, "text")}
            {form_field("edit-rail-notes", "rail_notes", "Notes", &detail.notes, "text")}
        </fieldset>
    }
}

#[component]
fn BoatEditFields(detail: crate::db::hops::BoatDetail) -> impl IntoView {
    view! {
        <fieldset class="edit-form-fieldset">
            <legend>"Boat Details"</legend>
            {form_field("edit-ship-name", "ship_name", "Ship Name", &detail.ship_name, "text")}
            {form_field("edit-cabin-type", "cabin_type", "Cabin Type", &detail.cabin_type, "text")}
            {form_field("edit-cabin-number", "cabin_number", "Cabin Number", &detail.cabin_number, "text")}
            {form_field("edit-boat-confirmation", "boat_confirmation", "Confirmation", &detail.confirmation_num, "text")}
            {form_field("edit-boat-booking-site", "boat_booking_site", "Booking Site", &detail.booking_site, "text")}
            {form_field("edit-boat-notes", "boat_notes", "Notes", &detail.notes, "text")}
        </fieldset>
    }
}

#[component]
fn TransportEditFields(detail: crate::db::hops::TransportDetail) -> impl IntoView {
    view! {
        <fieldset class="edit-form-fieldset">
            <legend>"Transport Details"</legend>
            {form_field("edit-transport-carrier", "transport_carrier", "Carrier", &detail.carrier_name, "text")}
            {form_field("edit-vehicle-desc", "vehicle_description", "Vehicle", &detail.vehicle_description, "text")}
            {form_field("edit-transport-confirmation", "transport_confirmation", "Confirmation", &detail.confirmation_num, "text")}
            {form_field("edit-transport-notes", "transport_notes", "Notes", &detail.notes, "text")}
        </fieldset>
    }
}

#[component]
pub(super) fn EditForm(hop: DetailRow) -> impl IntoView {
    let action = format!("/hops/{}", hop.id);
    let travel_type_str = hop.travel_type.to_string();
    let origin_lat = hop.origin_lat.to_string();
    let origin_lng = hop.origin_lng.to_string();
    let dest_lat = hop.dest_lat.to_string();
    let dest_lng = hop.dest_lng.to_string();
    let origin_country = hop.origin_country.clone().unwrap_or_default();
    let dest_country = hop.dest_country.clone().unwrap_or_default();

    let detail_fields = match hop.travel_type {
        TravelType::Air => hop.flight_detail.map_or_else(
            || ().into_any(),
            |d| view! { <FlightEditFields detail=d /> }.into_any(),
        ),
        TravelType::Rail => hop.rail_detail.map_or_else(
            || ().into_any(),
            |d| view! { <RailEditFields detail=d /> }.into_any(),
        ),
        TravelType::Boat => hop.boat_detail.map_or_else(
            || ().into_any(),
            |d| view! { <BoatEditFields detail=d /> }.into_any(),
        ),
        TravelType::Transport => hop.transport_detail.map_or_else(
            || ().into_any(),
            |d| view! { <TransportEditFields detail=d /> }.into_any(),
        ),
    };

    view! {
        <div id="edit-backdrop" class="edit-panel-backdrop"
            onclick="document.getElementById('edit-form').classList.remove('open');this.classList.remove('open')">
        </div>
        <section id="edit-form" class="hop-edit-form">
            <h3>
                "Edit Hop"
                <button class="edit-panel-close" type="button"
                    onclick="document.getElementById('edit-form').classList.remove('open');document.getElementById('edit-backdrop').classList.remove('open')"
                >"\u{2715}"</button>
            </h3>
            <form method="post" action=action>
                {hidden_field("travel_type", travel_type_str)}
                {hidden_field("origin_lat", origin_lat)}
                {hidden_field("origin_lng", origin_lng)}
                {hidden_field("dest_lat", dest_lat)}
                {hidden_field("dest_lng", dest_lng)}
                {hidden_field("origin_country", origin_country)}
                {hidden_field("dest_country", dest_country)}

                {form_field("edit-origin", "origin_name", "Origin", &hop.origin_name, "text")}
                {form_field("edit-dest", "dest_name", "Destination", &hop.dest_name, "text")}
                {form_field("edit-start-date", "start_date", "Start Date", &hop.start_date, "date")}
                {form_field("edit-end-date", "end_date", "End Date", &hop.end_date, "date")}

                {detail_fields}

                <div class="edit-form-actions">
                    <button class="btn btn-primary" type="submit">"Save Changes"</button>
                    <button class="btn btn-secondary" type="button"
                        onclick="document.getElementById('edit-form').classList.remove('open');document.getElementById('edit-backdrop').classList.remove('open')"
                    >"Cancel"</button>
                </div>
            </form>
        </section>
    }
}

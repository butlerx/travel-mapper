use super::{detail_row_view, timing_row};
use crate::db::hops::FullFlightDetail;
use leptos::prelude::*;

pub(super) struct FlightVerificationView {
    pub est_departure_airport: String,
    pub est_arrival_airport: String,
    pub callsign: String,
}

#[component]
pub(super) fn FlightSection(
    detail: FullFlightDetail,
    #[prop(optional_no_strip)] verification: Option<FlightVerificationView>,
) -> impl IntoView {
    let timing_rows: Vec<_> = [
        timing_row(
            "Gate Departure",
            &detail.gate_dep_scheduled,
            &detail.gate_dep_actual,
        ),
        timing_row("Takeoff", &detail.takeoff_scheduled, &detail.takeoff_actual),
        timing_row("Landing", &detail.landing_scheduled, &detail.landing_actual),
        timing_row(
            "Gate Arrival",
            &detail.gate_arr_scheduled,
            &detail.gate_arr_actual,
        ),
    ]
    .into_iter()
    .flatten()
    .collect();

    let has_timing = !timing_rows.is_empty();
    let adsb_route = verification.as_ref().and_then(|v| {
        if v.est_departure_airport.is_empty() || v.est_arrival_airport.is_empty() {
            None
        } else {
            Some(format!(
                "{} → {}",
                v.est_departure_airport, v.est_arrival_airport
            ))
        }
    });
    let adsb_callsign = verification
        .as_ref()
        .and_then(|v| (!v.callsign.is_empty()).then_some(v.callsign.clone()));

    view! {
        <section class="journey-detail-section">
            <h3>"Flight Info"</h3>
            <div class="journey-detail-grid">
                {detail_row_view("Airline", &detail.airline)}
                {detail_row_view("Flight", &detail.flight_number)}
                {detail_row_view("Aircraft", &detail.aircraft_type)}
                {detail_row_view("Tail Number", &detail.tail_number)}
                {detail_row_view("Departure Terminal", &detail.dep_terminal)}
                {detail_row_view("Departure Gate", &detail.dep_gate)}
                {detail_row_view("Arrival Terminal", &detail.arr_terminal)}
                {detail_row_view("Arrival Gate", &detail.arr_gate)}
                {if detail.canceled { view! {
                    <div class="journey-detail-label">"Status"</div>
                    <div class="journey-detail-value journey-detail-canceled">"Canceled"</div>
                }.into_any() } else { ().into_any() }}
                {detail_row_view("Diverted To", &detail.diverted_to)}
                {adsb_route.map(|route| view! {
                    <div class="journey-detail-label">"ADS-B Route"</div>
                    <div class="journey-detail-value">{route}</div>
                }.into_any())}
                {adsb_callsign
                    .as_ref()
                    .map(|callsign| detail_row_view("Callsign", callsign))}
            </div>
        </section>

        {if has_timing { view! {
            <section class="journey-detail-section">
                <h3>"Timing"</h3>
                <table class="journey-detail-timing">
                    <thead>
                        <tr>
                            <th>"Phase"</th>
                            <th>"Scheduled"</th>
                            <th>"Actual"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {timing_rows}
                    </tbody>
                </table>
            </section>
        }.into_any() } else { ().into_any() }}

        <section class="journey-detail-section">
            <h3>"Seat & Booking"</h3>
            <div class="journey-detail-grid">
                {detail_row_view("Cabin Class", &detail.cabin_class)}
                {detail_row_view("Seat", &detail.seat)}
                {detail_row_view("Seat Type", &detail.seat_type)}
                {detail_row_view("Booking Ref", &detail.pnr)}
                {detail_row_view("Flight Reason", &detail.flight_reason)}
            </div>
        </section>

        {if detail.notes.is_empty() { ().into_any() } else {
            let notes = detail.notes.clone();
            view! {
                <section class="journey-detail-section">
                    <h3>"Notes"</h3>
                    <p class="journey-detail-notes">{notes}</p>
                </section>
            }.into_any()
        }}
    }
}

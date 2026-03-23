use super::{navbar::NavBar, shell::Shell};
use crate::db::hops::{DetailRow, FullFlightDetail, TravelType};
use leptos::prelude::*;

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
            <div class="hop-detail-label">{label}</div>
            <div class="hop-detail-value">{v}</div>
        }
        .into_any()
    }
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

#[component]
fn FlightSection(detail: FullFlightDetail) -> impl IntoView {
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

    view! {
        <section class="hop-detail-section">
            <h3>"Flight Info"</h3>
            <div class="hop-detail-grid">
                {detail_row_view("Airline", &detail.airline)}
                {detail_row_view("Flight", &detail.flight_number)}
                {detail_row_view("Aircraft", &detail.aircraft_type)}
                {detail_row_view("Tail Number", &detail.tail_number)}
                {detail_row_view("Departure Terminal", &detail.dep_terminal)}
                {detail_row_view("Departure Gate", &detail.dep_gate)}
                {detail_row_view("Arrival Terminal", &detail.arr_terminal)}
                {detail_row_view("Arrival Gate", &detail.arr_gate)}
                {if detail.canceled { view! {
                    <div class="hop-detail-label">"Status"</div>
                    <div class="hop-detail-value hop-detail-canceled">"Canceled"</div>
                }.into_any() } else { ().into_any() }}
                {detail_row_view("Diverted To", &detail.diverted_to)}
            </div>
        </section>

        {if has_timing { view! {
            <section class="hop-detail-section">
                <h3>"Timing"</h3>
                <table class="hop-detail-timing">
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

        <section class="hop-detail-section">
            <h3>"Seat & Booking"</h3>
            <div class="hop-detail-grid">
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
                <section class="hop-detail-section">
                    <h3>"Notes"</h3>
                    <p class="hop-detail-notes">{notes}</p>
                </section>
            }.into_any()
        }}
    }
}

#[component]
fn RailSection(detail: crate::db::hops::RailDetail) -> impl IntoView {
    view! {
        <section class="hop-detail-section">
            <h3>"Rail Details"</h3>
            <div class="hop-detail-grid">
                {detail_row_view("Carrier", &detail.carrier)}
                {detail_row_view("Train", &detail.train_number)}
                {detail_row_view("Class", &detail.service_class)}
                {detail_row_view("Coach", &detail.coach_number)}
                {detail_row_view("Seats", &detail.seats)}
                {detail_row_view("Confirmation", &detail.confirmation_num)}
                {detail_row_view("Booking Site", &detail.booking_site)}
            </div>
        </section>
        {if detail.notes.is_empty() { ().into_any() } else {
            let notes = detail.notes.clone();
            view! {
                <section class="hop-detail-section">
                    <h3>"Notes"</h3>
                    <p class="hop-detail-notes">{notes}</p>
                </section>
            }.into_any()
        }}
    }
}

#[component]
fn BoatSection(detail: crate::db::hops::BoatDetail) -> impl IntoView {
    view! {
        <section class="hop-detail-section">
            <h3>"Boat Details"</h3>
            <div class="hop-detail-grid">
                {detail_row_view("Ship", &detail.ship_name)}
                {detail_row_view("Cabin Type", &detail.cabin_type)}
                {detail_row_view("Cabin Number", &detail.cabin_number)}
                {detail_row_view("Confirmation", &detail.confirmation_num)}
                {detail_row_view("Booking Site", &detail.booking_site)}
            </div>
        </section>
        {if detail.notes.is_empty() { ().into_any() } else {
            let notes = detail.notes.clone();
            view! {
                <section class="hop-detail-section">
                    <h3>"Notes"</h3>
                    <p class="hop-detail-notes">{notes}</p>
                </section>
            }.into_any()
        }}
    }
}

#[component]
fn TransportSection(detail: crate::db::hops::TransportDetail) -> impl IntoView {
    view! {
        <section class="hop-detail-section">
            <h3>"Transport Details"</h3>
            <div class="hop-detail-grid">
                {detail_row_view("Carrier", &detail.carrier_name)}
                {detail_row_view("Vehicle", &detail.vehicle_description)}
                {detail_row_view("Confirmation", &detail.confirmation_num)}
            </div>
        </section>
        {if detail.notes.is_empty() { ().into_any() } else {
            let notes = detail.notes.clone();
            view! {
                <section class="hop-detail-section">
                    <h3>"Notes"</h3>
                    <p class="hop-detail-notes">{notes}</p>
                </section>
            }.into_any()
        }}
    }
}

#[component]
pub fn HopDetailPage(hop: DetailRow) -> impl IntoView {
    let emoji = hop.travel_type.emoji();
    let badge_class = format!("hop-detail-badge {}", travel_type_class(&hop.travel_type));
    let type_label = hop.travel_type.to_string();

    let dates = if hop.start_date == hop.end_date {
        hop.start_date.clone()
    } else {
        format!("{} \u{2013} {}", hop.start_date, hop.end_date)
    };

    let countries_view = match (&hop.origin_country, &hop.dest_country) {
        (Some(orig), Some(dest)) if orig == dest => {
            let c = orig.clone();
            view! { <p class="hop-detail-countries">{c}</p> }.into_any()
        }
        (Some(orig), Some(dest)) => {
            let text = format!("{orig} \u{2192} {dest}");
            view! { <p class="hop-detail-countries">{text}</p> }.into_any()
        }
        _ => ().into_any(),
    };

    let origin_lat = hop.origin_lat.to_string();
    let origin_lng = hop.origin_lng.to_string();
    let dest_lat = hop.dest_lat.to_string();
    let dest_lng = hop.dest_lng.to_string();

    let detail_section = match hop.travel_type {
        TravelType::Air => hop.flight_detail.map_or_else(
            || ().into_any(),
            |d| view! { <FlightSection detail=d /> }.into_any(),
        ),
        TravelType::Rail => hop.rail_detail.map_or_else(
            || ().into_any(),
            |d| view! { <RailSection detail=d /> }.into_any(),
        ),
        TravelType::Boat => hop.boat_detail.map_or_else(
            || ().into_any(),
            |d| view! { <BoatSection detail=d /> }.into_any(),
        ),
        TravelType::Transport => hop.transport_detail.map_or_else(
            || ().into_any(),
            |d| view! { <TransportSection detail=d /> }.into_any(),
        ),
    };

    let map_script = r"
(function() {
    var el = document.getElementById('hop-map');
    if (!el || typeof L === 'undefined') return;
    var oLat = parseFloat(el.dataset.originLat);
    var oLng = parseFloat(el.dataset.originLng);
    var dLat = parseFloat(el.dataset.destLat);
    var dLng = parseFloat(el.dataset.destLng);
    if (isNaN(oLat) || isNaN(dLat)) return;
    var map = L.map('hop-map', { scrollWheelZoom: false });
    L.tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png', {
        attribution: '&copy; OpenStreetMap contributors',
        maxZoom: 18
    }).addTo(map);
    L.marker([oLat, oLng]).addTo(map);
    L.marker([dLat, dLng]).addTo(map);
    L.polyline([[oLat, oLng], [dLat, dLng]], {
        color: '#4a90d9', weight: 3, dashArray: '8 4'
    }).addTo(map);
    map.fitBounds([[oLat, oLng], [dLat, dLng]], { padding: [40, 40] });
})();
";

    view! {
        <Shell title="Hop Detail".to_owned() body_class="hop-detail-layout">
            <NavBar current="" />
            <main class="hop-detail-page">
                <a href="/dashboard" class="hop-detail-back">"\u{2190} Dashboard"</a>

                <header class="hop-detail-header">
                    <h1 class="hop-detail-route">
                        <span>{emoji}</span>
                        " "
                        {hop.origin_name}
                        " \u{2192} "
                        {hop.dest_name}
                    </h1>
                    <p class="hop-detail-dates">{dates}</p>
                    <span class=badge_class>{type_label}</span>
                    {countries_view}
                </header>

                <div
                    id="hop-map"
                    data-origin-lat=origin_lat
                    data-origin-lng=origin_lng
                    data-dest-lat=dest_lat
                    data-dest-lng=dest_lng
                ></div>

                {detail_section}

                <link rel="stylesheet" href="https://unpkg.com/leaflet@1.9.4/dist/leaflet.css"
                    integrity="sha256-p4NxAoJBhIIN+hmNHrzRCf9tD/miZyoHS5obTRR9BMY="
                    crossorigin="" />
                <script src="https://unpkg.com/leaflet@1.9.4/dist/leaflet.js"
                    integrity="sha256-20nQCchB9co0qIjJZRGuk2/Z9VM+kNiyxNV1lvTlZBo="
                    crossorigin=""></script>
                <script inner_html=map_script></script>
            </main>
        </Shell>
    }
}

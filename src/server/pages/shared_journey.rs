use crate::{
    db::{hops::DetailRow, status_enrichments},
    server::{components::Shell, routes::journeys::JourneyTravelType},
};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use leptos::prelude::*;

/// Render the public shared-journey page without auth-only controls.
pub fn render_page(
    token_hash: &str,
    journey: DetailRow,
    enrichment: Option<status_enrichments::Row>,
) -> Response {
    let html = view! {
        <SharedJourneyPage token_hash=token_hash.to_owned() journey=journey enrichment=enrichment />
    };
    (StatusCode::OK, axum::response::Html(html.to_html())).into_response()
}

fn travel_type_class(tt: &crate::db::hops::TravelType) -> &'static str {
    match tt {
        crate::db::hops::TravelType::Air => "air",
        crate::db::hops::TravelType::Rail => "rail",
        crate::db::hops::TravelType::Boat => "boat",
        crate::db::hops::TravelType::Transport => "transport",
    }
}

#[component]
fn SharedJourneyPage(
    token_hash: String,
    journey: DetailRow,
    #[prop(optional_no_strip)] enrichment: Option<status_enrichments::Row>,
) -> impl IntoView {
    let page_title = format!(
        "{} → {} — Live Status",
        journey.origin_name, journey.dest_name
    );
    let travel_type = JourneyTravelType::from(journey.travel_type.clone());
    let badge_class = format!(
        "journey-detail-badge {}",
        travel_type_class(&journey.travel_type)
    );

    let status_badge = enrichment
        .as_ref()
        .filter(|e| !e.status.is_empty())
        .map(|e| {
            let css = format!(
                "status-badge status-{}",
                e.status.to_lowercase().replace(' ', "-")
            );
            let label = match e.delay_minutes {
                Some(mins) if mins > 0 => format!("{} (+{}m)", e.status, mins),
                Some(mins) if mins < 0 => format!("{} ({}m)", e.status, mins),
                _ => e.status.clone(),
            };
            (css, label)
        });

    let meta_chips: Vec<String> = enrichment.as_ref().map_or_else(Vec::new, |e| {
        [
            (!e.dep_gate.is_empty()).then(|| format!("Gate {}", e.dep_gate)),
            (!e.dep_terminal.is_empty()).then(|| format!("Terminal {}", e.dep_terminal)),
            (!e.arr_gate.is_empty()).then(|| format!("Arr. gate {}", e.arr_gate)),
            (!e.arr_terminal.is_empty()).then(|| format!("Arr. terminal {}", e.arr_terminal)),
            (!e.dep_platform.is_empty()).then(|| format!("Platform {}", e.dep_platform)),
            (!e.arr_platform.is_empty()).then(|| format!("Arr. platform {}", e.arr_platform)),
            (!e.aircraft_reg.is_empty()).then(|| format!("Aircraft {}", e.aircraft_reg)),
        ]
        .into_iter()
        .flatten()
        .collect()
    });

    let origin_lat = journey.origin_lat.to_string();
    let origin_lng = journey.origin_lng.to_string();
    let dest_lat = journey.dest_lat.to_string();
    let dest_lng = journey.dest_lng.to_string();
    let enrichment_url = format!("/share/journey/{token_hash}/enrichments");

    view! {
        <Shell title=page_title body_class="journey-detail-layout">
            <main class="journey-detail-page" data-enrichment-url=enrichment_url>
                <header class="page-header">
                    <div class="page-title-row">
                        <h1 class="journey-detail-route">
                            <span>{travel_type.emoji()}</span>
                            " "
                            {journey.origin_name}
                            " → "
                            {journey.dest_name}
                        </h1>
                    </div>
                    <p class="journey-detail-dates">{journey.start_date.clone()}</p>
                    <span class=badge_class>{travel_type.to_string()}</span>
                    <span data-shared-status>
                        {status_badge.map(|(css, label)| view! { <span class=css>{label}</span> })}
                    </span>
                    <div class="journey-detail-meta">
                        {meta_chips
                            .into_iter()
                            .map(|chip| view! { <span class="journey-detail-meta-item">{chip}</span> })
                            .collect::<Vec<_>>()}
                    </div>
                </header>

                <div
                    id="journey-map"
                    data-origin-lat=origin_lat
                    data-origin-lng=origin_lng
                    data-dest-lat=dest_lat
                    data-dest-lng=dest_lng
                    data-type=journey.travel_type.to_string()
                ></div>

                <link rel="stylesheet" href="https://unpkg.com/maplibre-gl@5/dist/maplibre-gl.css" crossorigin="" />
                <script src="https://unpkg.com/pmtiles@3/dist/pmtiles.js"></script>
                <script src="https://unpkg.com/maplibre-gl@5/dist/maplibre-gl.js"></script>
                <script src="https://unpkg.com/@protomaps/basemaps@5/dist/basemaps.js"></script>
                <script type="module" src="/static/map-core.js"></script>
                <script type="module" src="/static/journey-map.js"></script>
                <script type="module" src="/static/shared-journey-poll.js"></script>
            </main>
        </Shell>
    }
}

use crate::server::components::{
    Shell, format_utils::format_distance, overview_cards::DetailedStats, top_list::CountedItem,
};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use leptos::prelude::*;

/// Equatorial circumference of the Earth, for the "times around the world" stat.
const EARTH_CIRCUMFERENCE_KM: u64 = 40_075;

/// Render the public, shareable Year-in-Review recap for a single year.
pub(crate) fn render(stats: DetailedStats, token: &str, year: &str) -> Response {
    let html = view! {
        <YearInReviewPage stats=stats token=token.to_owned() year=year.to_owned() />
    };
    (StatusCode::OK, axum::response::Html(html.to_html())).into_response()
}

fn top_name(items: &[CountedItem]) -> Option<String> {
    items
        .first()
        .map(|item| format!("{} ({})", item.name, item.count))
}

fn highlight(label: &'static str, value: Option<String>) -> Option<impl IntoView + use<>> {
    let value = value?;
    Some(view! {
        <div class="yir-highlight">
            <span class="yir-highlight-label">{label}</span>
            <span class="yir-highlight-value">{value}</span>
        </div>
    })
}

fn stat_tile(value: String, label: &'static str) -> impl IntoView + use<> {
    view! {
        <div class="yir-tile">
            <span class="yir-tile-value">{value}</span>
            <span class="yir-tile-label">{label}</span>
        </div>
    }
}

/// "N.N× around the world" derived from total distance, or `None` when the
/// year's travel is under a tenth of a lap. Computed in integer tenths to avoid
/// floating-point distance maths.
fn laps_around_world(distance_km: u64) -> Option<String> {
    let tenths = distance_km.saturating_mul(10) / EARTH_CIRCUMFERENCE_KM;
    (tenths >= 1).then(|| format!("{}.{}\u{00d7} around the world", tenths / 10, tenths % 10))
}

#[component]
fn YearInReviewPage(stats: DetailedStats, token: String, year: String) -> impl IntoView {
    let has_data = stats.total_journeys > 0;
    let distance = format_distance(stats.total_distance_km);

    let og_meta = format!(
        concat!(
            r#"<meta property="og:title" content="{year} Year in Review">"#,
            r#"<meta property="og:description" content="{journeys} journeys and {distance} across {countries} countries.">"#,
            r#"<meta property="og:url" content="/share/{token}/review">"#,
        ),
        year = year,
        journeys = stats.total_journeys,
        distance = distance,
        countries = stats.unique_countries,
        token = token,
    );

    // Mode tiles for whichever travel types appear this year.
    let mode_tiles: Vec<_> = [
        (stats.total_flights, "Flights"),
        (stats.total_rail, "Rail trips"),
        (stats.total_boat, "Boat trips"),
        (stats.total_transport, "Transport legs"),
    ]
    .into_iter()
    .filter(|(count, _)| *count > 0)
    .map(|(count, label)| stat_tile(count.to_string(), label))
    .collect();

    let highlights: Vec<_> = [
        highlight("Top airline", top_name(&stats.top_airlines)),
        highlight("Top route", top_name(&stats.top_routes)),
        highlight("Top aircraft", top_name(&stats.top_aircraft)),
        highlight("Most visited", top_name(&stats.countries)),
        highlight(
            "Around the world",
            laps_around_world(stats.total_distance_km),
        ),
    ]
    .into_iter()
    .flatten()
    .collect();

    // Links to the other years available behind this share token.
    let token_for_years = token.clone();
    let current_year = year.clone();
    let year_links: Vec<_> = stats
        .available_years
        .iter()
        .rev()
        .map(|y| {
            let href = format!("/share/{token_for_years}/review?year={y}");
            let is_current = *y == current_year;
            view! {
                <a
                    href=href
                    class="yir-year-link"
                    class:yir-year-link--active=is_current
                >{y.clone()}</a>
            }
        })
        .collect();

    let full_stats_url = format!("/share/{token}");

    view! {
        <Shell title=format!("{year} Year in Review") body_class="yir-layout" og_meta=og_meta>
            {if has_data {
                view! {
                    <main class="yir-page">
                        <header class="yir-header">
                            <p class="yir-eyebrow">"Year in Review"</p>
                            <h1 class="yir-year">{year.clone()}</h1>
                        </header>

                        <div class="yir-tiles yir-tiles--hero">
                            {stat_tile(stats.total_journeys.to_string(), "Journeys")}
                            {stat_tile(distance, "Distance")}
                            {stat_tile(stats.unique_countries.to_string(), "Countries")}
                        </div>

                        {(!mode_tiles.is_empty()).then(|| view! {
                            <div class="yir-tiles">{mode_tiles}</div>
                        })}

                        {(!highlights.is_empty()).then(|| view! {
                            <section class="yir-highlights">{highlights}</section>
                        })}

                        <nav class="yir-years" aria-label="Other years">{year_links}</nav>

                        <a class="yir-full-link" href=full_stats_url>"View full stats \u{2192}"</a>
                    </main>
                }.into_any()
            } else {
                view! {
                    <main class="yir-page">
                        <header class="yir-header">
                            <p class="yir-eyebrow">"Year in Review"</p>
                            <h1 class="yir-year">{year.clone()}</h1>
                        </header>
                        <p class="yir-empty">"No journeys recorded for this year."</p>
                        <a class="yir-full-link" href=full_stats_url>"View full stats \u{2192}"</a>
                    </main>
                }.into_any()
            }}
        </Shell>
    }
}

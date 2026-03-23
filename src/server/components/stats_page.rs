use super::{navbar::NavBar, shell::Shell};
use crate::server::pages::stats::{CountedItem, DetailedStats};
use leptos::prelude::*;
use std::fmt::Write;

fn format_distance(km: u64) -> String {
    if km >= 1_000_000 {
        let whole = km / 1_000_000;
        let frac = (km % 1_000_000) / 100_000;
        format!("{whole}.{frac}M km")
    } else if km >= 10_000 {
        format!("{}k km", km / 1_000)
    } else {
        format!("{km} km")
    }
}

fn format_year_range(first: Option<&String>, last: Option<&String>) -> String {
    match (first, last) {
        (Some(f), Some(l)) if f == l => f.clone(),
        (Some(f), Some(l)) => format!("{f}\u{2013}{l}"),
        _ => "\u{2014}".to_owned(),
    }
}

#[component]
fn OverviewCards(stats: DetailedStats, distance: String, year_range: String) -> impl IntoView {
    view! {
        <div class="stats-overview">
            <div class="stat-row">
                <div class="stat-card">
                    <div class="stat-label">"Total Journeys"</div>
                    <div class="stat-value">{stats.total_hops}</div>
                </div>
                <div class="stat-card">
                    <div class="stat-label">"Flights"</div>
                    <div class="stat-value">{stats.total_flights}</div>
                </div>
                <div class="stat-card">
                    <div class="stat-label">"Rail"</div>
                    <div class="stat-value">{stats.total_rail}</div>
                </div>
                <div class="stat-card">
                    <div class="stat-label">"Distance"</div>
                    <div class="stat-value">{distance}</div>
                </div>
                <div class="stat-card">
                    <div class="stat-label">"Airports"</div>
                    <div class="stat-value">{stats.unique_airports}</div>
                </div>
                <div class="stat-card">
                    <div class="stat-label">"Countries"</div>
                    <div class="stat-value">{stats.unique_countries}</div>
                </div>
                <div class="stat-card">
                    <div class="stat-label">"Years"</div>
                    <div class="stat-value">{year_range}</div>
                </div>
            </div>
        </div>
    }
}

#[component]
fn TopList(title: &'static str, items: Vec<CountedItem>) -> impl IntoView {
    let max_count = items.first().map_or(1, |i| i.count.max(1));

    view! {
        <section class="stats-section">
            <h3 class="stats-section-title">{title}</h3>
            {if items.is_empty() {
                view! { <p class="stats-empty">"No data"</p> }.into_any()
            } else {
                view! {
                    <ul class="stats-top-list">
                        {items.into_iter().map(|item| {
                            let pct = item.count * 100 / max_count;
                            let width = format!("width: {pct}%");
                            view! {
                                <li class="stats-top-item">
                                    <div class="stats-top-bar" style=width></div>
                                    <span class="stats-top-name">{item.name}</span>
                                    <span class="stats-top-count">{item.count}</span>
                                </li>
                            }
                        }).collect::<Vec<_>>()}
                    </ul>
                }.into_any()
            }}
        </section>
    }
}

#[component]
fn YearFilter(available_years: Vec<String>, selected_year: Option<String>) -> impl IntoView {
    if available_years.is_empty() {
        return ().into_any();
    }

    view! {
        <form method="get" action="/stats" class="stats-year-filter">
            <label for="year-filter">"Filter by year: "</label>
            <select name="year" id="year-filter" onchange="this.form.submit()">
                <option value="" selected=selected_year.is_none()>"All years"</option>
                {available_years.into_iter().rev().map(|y| {
                    let is_selected = selected_year.as_ref() == Some(&y);
                    let display = y.clone();
                    view! {
                        <option value={y} selected=is_selected>{display}</option>
                    }
                }).collect::<Vec<_>>()}
            </select>
        </form>
    }
    .into_any()
}

/// Serialize country counts as a JSON object: `{"US":5,"GB":3,...}`.
fn countries_json(countries: &[CountedItem]) -> String {
    let mut buf = String::from('{');
    for (i, item) in countries.iter().enumerate() {
        if i > 0 {
            buf.push(',');
        }
        let _ = write!(buf, "\"{}\":{}", item.name, item.count);
    }
    buf.push('}');
    buf
}

#[component]
pub fn StatsPage(stats: DetailedStats) -> impl IntoView {
    let has_data = stats.total_hops > 0;
    let distance = format_distance(stats.total_distance_km);
    let year_range = format_year_range(stats.first_year.as_ref(), stats.last_year.as_ref());

    let available_years = stats.available_years.clone();
    let selected_year = stats.selected_year.clone();
    let top_airlines = stats.top_airlines.clone();
    let top_aircraft = stats.top_aircraft.clone();
    let top_routes = stats.top_routes.clone();
    let cabin_class = stats.cabin_class_breakdown.clone();
    let seat_type = stats.seat_type_breakdown.clone();
    let flight_reason = stats.flight_reason_breakdown.clone();
    let countries = stats.countries.clone();
    let countries_script = format!("window.countryCounts={};", countries_json(&countries));

    view! {
        <Shell title="Stats".to_owned() body_class="stats-layout">
            <NavBar current="stats" />

            {if has_data {
                view! {
                    <main class="stats-page">
                        <YearFilter available_years=available_years selected_year=selected_year />
                        <OverviewCards stats=stats distance=distance year_range=year_range />
                        <div class="stats-grid">
                            <TopList title="Top Airlines" items=top_airlines />
                            <TopList title="Top Aircraft" items=top_aircraft />
                            <TopList title="Top Routes" items=top_routes />
                            <TopList title="Cabin Class" items=cabin_class />
                            <TopList title="Seat Type" items=seat_type />
                            <TopList title="Flight Reason" items=flight_reason />
                            <TopList title="Countries Visited" items=countries />
                        </div>
                        <section class="stats-section stats-map-section">
                            <h3 class="stats-section-title">"Country Map"</h3>
                            <div id="stats-map"></div>
                        </section>
                        <link rel="stylesheet" href="https://unpkg.com/leaflet@1.9.4/dist/leaflet.css"
                            integrity="sha256-p4NxAoJBhIIN+hmNHrzRCf9tD/miZyoHS5obTRR9BMY="
                            crossorigin="" />
                        <script src="https://unpkg.com/leaflet@1.9.4/dist/leaflet.js"
                            integrity="sha256-20nQCchB9co0qIjJZRGuk2/Z9VM+kNiyxNV1lvTlZBo="
                            crossorigin=""></script>
                        <script src="https://cdn.jsdelivr.net/npm/topojson-client@3"></script>
                        <script inner_html=countries_script></script>
                        <script src="/static/stats-map.js"></script>
                    </main>
                }.into_any()
            } else {
                view! {
                    <main class="container-wide">
                        <section class="card">
                            <div class="empty-state">
                                <div class="empty-state-icon">{"\u{1F4CA}"}</div>
                                <p>"No travel data yet. Add flights or sync from TripIt to see your stats."</p>
                            </div>
                        </section>
                    </main>
                }.into_any()
            }}
        </Shell>
    }
}

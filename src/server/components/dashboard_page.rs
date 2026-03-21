mod map_controls;
mod stats_bar;

use super::{navbar::NavBar, shell::Shell};
use crate::server::pages::dashboard::TravelStats;
use leptos::prelude::*;
use map_controls::MapControls;
use stats_bar::StatsBar;

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
pub fn DashboardPage(
    hops_json: String,
    hop_count: usize,
    stats: TravelStats,
    #[prop(optional_no_strip)] error: Option<String>,
) -> impl IntoView {
    let has_hops = hop_count > 0;
    let hops_script = format!("window.allHops={hops_json};");

    let distance = format_distance(stats.total_distance_km);
    let year_range = format_year_range(stats.first_year.as_ref(), stats.last_year.as_ref());

    view! {
        <Shell title="Dashboard".to_owned() body_class="dashboard-layout">
            <NavBar current="dashboard" />
            {error.map(|e| view! {
                <div class="alert alert-error" role="alert">{e}</div>
            })}

            {if has_hops {
                view! {
                    <StatsBar stats=stats distance=distance year_range=year_range />
                    <div class="dashboard-main">
                        <div class="dashboard-map-col">
                            <div id="map"></div>
                            <MapControls hop_count=hop_count />
                        </div>
                        <aside id="journey-sidebar" class="journey-sidebar"></aside>
                    </div>
                    <link rel="stylesheet" href="https://unpkg.com/leaflet@1.9.4/dist/leaflet.css"
                        integrity="sha256-p4NxAoJBhIIN+hmNHrzRCf9tD/miZyoHS5obTRR9BMY="
                        crossorigin="" />
                    <script src="https://unpkg.com/leaflet@1.9.4/dist/leaflet.js"
                        integrity="sha256-20nQCchB9co0qIjJZRGuk2/Z9VM+kNiyxNV1lvTlZBo="
                        crossorigin=""></script>
                    <script inner_html=hops_script></script>
                    <script src="/static/map.js"></script>
                }.into_any()
            } else {
                view! {
                    <main class="container-wide">
                        <section class="card">
                            <div class="empty-state">
                                <div class="empty-state-icon">{"\u{1F30D}"}</div>
                                <p>"No hops yet. Connect TripIt in " <a href="/settings">"Settings"</a> " and sync to see your travel data."</p>
                            </div>
                        </section>
                    </main>
                }.into_any()
            }}
        </Shell>
    }
}

use super::{navbar::NavBar, shell::Shell};
use leptos::prelude::*;

#[component]
pub fn DashboardPage(
    hops_json: String,
    hop_count: usize,
    #[prop(optional_no_strip)] error: Option<String>,
) -> impl IntoView {
    let has_hops = hop_count > 0;
    let hops_script = format!("window.allHops={hops_json};");

    view! {
        <Shell title="Dashboard".to_owned()>
            <NavBar current="dashboard" />
            {error.map(|e| view! {
                <div class="alert alert-error" role="alert">{e}</div>
            })}

            {if has_hops {
                view! {
                    <div id="map"></div>
                    <div class="map-controls">
                        <div class="map-filters">
                            <label for="filter-type">{"\u{1F3F7}\u{FE0F} Type"}</label>
                            <select id="filter-type">
                                <option value="all">"All Types"</option>
                                <option value="air">{"\u{2708}\u{FE0F} Air"}</option>
                                <option value="rail">{"\u{1F686} Rail"}</option>
                                <option value="cruise">{"\u{1F6A2} Cruise"}</option>
                                <option value="transport">{"\u{1F697} Transport"}</option>
                            </select>
                            <label for="filter-year">{"\u{1F4C5} Year"}</label>
                            <select id="filter-year">
                                <option value="all">"All Years"</option>
                            </select>
                        </div>
                        <div class="map-legend">
                            <h3>{"\u{1F5FA}\u{FE0F} Routes"}</h3>
                            <div class="legend-item">
                                <div class="legend-swatch legend-air"></div>
                                <span>{"\u{2708}\u{FE0F} Air"}</span>
                            </div>
                            <div class="legend-item">
                                <div class="legend-swatch legend-rail"></div>
                                <span>{"\u{1F686} Rail"}</span>
                            </div>
                            <div class="legend-item">
                                <div class="legend-swatch legend-cruise"></div>
                                <span>{"\u{1F6A2} Cruise"}</span>
                            </div>
                            <div class="legend-item">
                                <div class="legend-swatch legend-transport"></div>
                                <span>{"\u{1F697} Transport"}</span>
                            </div>
                            <div class="legend-count" id="hop-count">{hop_count}" hops"</div>
                        </div>
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

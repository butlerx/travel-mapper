use leptos::prelude::*;

#[component]
pub fn MapControls(hop_count: usize) -> impl IntoView {
    view! {
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
                <div class="legend-count" id="hop-count">{hop_count}" journeys"</div>
            </div>
        </div>
    }
}

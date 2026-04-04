use leptos::prelude::*;

#[component]
pub fn MapControls(journey_count: usize) -> impl IntoView {
    view! {
        <div class="map-controls">
            <div class="map-toggles">
                <label class="map-toggle">
                    <input type="checkbox" id="toggle-routes" checked />
                    <span>{"\u{1F5FA}\u{FE0F} Routes"}</span>
                </label>
                <label class="map-toggle">
                    <input type="checkbox" id="toggle-airports" checked />
                    <span>{"\u{1F4CD} Airports"}</span>
                </label>
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
                    <div class="legend-swatch legend-boat"></div>
                    <span>{"\u{1F6A2} Boat"}</span>
                </div>
                <div class="legend-item">
                    <div class="legend-swatch legend-transport"></div>
                    <span>{"\u{1F697} Transport"}</span>
                </div>
                <div class="legend-count" id="journey-count">{journey_count}" journeys"</div>
            </div>
        </div>
    }
}

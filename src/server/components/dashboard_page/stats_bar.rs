use crate::server::pages::dashboard::TravelStats;
use leptos::prelude::*;

#[component]
pub fn StatsBar(stats: TravelStats, distance: String, year_range: String) -> impl IntoView {
    view! {
        <div class="dashboard-stats">
            <div class="stat-row">
                <div class="stat-card">
                    <div class="stat-label">"Journeys"</div>
                    <div class="stat-value">{stats.total_journeys}</div>
                </div>
                <div class="stat-card">
                    <div class="stat-label">"Routes"</div>
                    <div class="stat-value">{stats.total_flights + stats.total_rail}</div>
                </div>
                <div class="stat-card">
                    <div class="stat-label">"Distance"</div>
                    <div class="stat-value">{distance}</div>
                </div>
                <div class="stat-card">
                    <div class="stat-label">"Places"</div>
                    <div class="stat-value">{stats.cities_visited}</div>
                </div>
                <div class="stat-card">
                    <div class="stat-label">"Countries"</div>
                    <div class="stat-value">{stats.airports_visited}</div>
                </div>
                <div class="stat-card">
                    <div class="stat-label">"Years"</div>
                    <div class="stat-value">{year_range}</div>
                </div>
            </div>
        </div>
    }
}

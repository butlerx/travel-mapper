use crate::server::components::top_list::CountedItem;
use crate::server::routes::journeys::{JourneyResponse, JourneyTravelType};
use leptos::prelude::*;
use std::collections::HashSet;

/// Aggregated travel statistics computed from a user's journey history.
#[derive(Default, Clone)]
pub struct DetailedStats {
    pub total_journeys: usize,
    pub total_flights: usize,
    pub total_rail: usize,
    pub total_boat: usize,
    pub total_transport: usize,
    pub total_distance_km: u64,
    pub unique_airports: usize,
    pub unique_stations: usize,
    pub unique_countries: usize,
    pub top_airlines: Vec<CountedItem>,
    pub top_aircraft: Vec<CountedItem>,
    pub top_routes: Vec<CountedItem>,
    pub cabin_class_breakdown: Vec<CountedItem>,
    pub seat_type_breakdown: Vec<CountedItem>,
    pub flight_reason_breakdown: Vec<CountedItem>,
    pub top_rail_carriers: Vec<CountedItem>,
    pub top_train_numbers: Vec<CountedItem>,
    pub rail_service_class_breakdown: Vec<CountedItem>,
    pub top_ships: Vec<CountedItem>,
    pub boat_cabin_type_breakdown: Vec<CountedItem>,
    pub top_transport_carriers: Vec<CountedItem>,
    pub transport_vehicle_breakdown: Vec<CountedItem>,
    pub countries: Vec<CountedItem>,
    pub available_years: Vec<String>,
    pub selected_year: Option<String>,
    pub selected_travel_type: Option<String>,
    pub first_year: Option<String>,
    pub last_year: Option<String>,
    pub spending_summary: Vec<String>,
    pub miles_by_program: Vec<(String, f64)>,
    pub miles_summary: Vec<String>,
}

#[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn positive_km_to_u64(km: f64) -> u64 {
    km.max(0.0).trunc() as u64
}

impl From<&[JourneyResponse]> for DetailedStats {
    fn from(journeys: &[JourneyResponse]) -> Self {
        use crate::distance::haversine_km;

        let mut stats = Self {
            total_journeys: journeys.len(),
            ..Self::default()
        };

        let mut airports: HashSet<&str> = HashSet::new();
        let mut stations: HashSet<&str> = HashSet::new();
        let mut years: Vec<&str> = Vec::new();

        for journey in journeys {
            match journey.travel_type {
                JourneyTravelType::Air => {
                    stats.total_flights += 1;
                    airports.insert(&journey.origin_name);
                    airports.insert(&journey.dest_name);
                }
                JourneyTravelType::Rail => {
                    stats.total_rail += 1;
                    stations.insert(&journey.origin_name);
                    stations.insert(&journey.dest_name);
                }
                JourneyTravelType::Boat => stats.total_boat += 1,
                JourneyTravelType::Transport => stats.total_transport += 1,
            }

            if journey.origin_lat != 0.0
                || journey.origin_lng != 0.0
                || journey.dest_lat != 0.0
                || journey.dest_lng != 0.0
            {
                let km = haversine_km(
                    journey.origin_lat,
                    journey.origin_lng,
                    journey.dest_lat,
                    journey.dest_lng,
                );
                if km.is_finite() && km > 0.0 {
                    stats.total_distance_km += positive_km_to_u64(km);
                }
            }

            if !journey.start_date.is_empty()
                && let Some(y) = journey.start_date.get(..4)
            {
                years.push(y);
            }
        }

        stats.unique_airports = airports.len();
        stats.unique_stations = stations.len();

        years.sort_unstable();
        stats.first_year = years.first().map(|y| (*y).to_owned());
        stats.last_year = years.last().map(|y| (*y).to_owned());

        stats
    }
}

/// A single overview stat card: icon, label, and value. `hero` enlarges the
/// value to the display type scale for the dashboard's headline figures.
fn stat_card(icon: &'static str, label: &'static str, value: String, hero: bool) -> impl IntoView {
    let value_class = if hero {
        "stat-value stat-value-hero"
    } else {
        "stat-value"
    };
    view! {
        <div class="stat-card">
            <div class="stat-card-head">
                <span class="stat-icon" aria-hidden="true">{icon}</span>
                <div class="stat-label">{label}</div>
            </div>
            <div class=value_class>{value}</div>
        </div>
    }
}

/// Overview stat cards shown above the dashboard map and on the stats page.
///
/// When `compact` is true (dashboard), only four headline cards are shown
/// inline; the rest collapse into a "More stats" disclosure so the map stays
/// near the top of the viewport. When false (stats page), every card is shown.
#[component]
pub fn OverviewCards(
    stats: DetailedStats,
    distance: String,
    year_range: String,
    #[prop(default = false)] compact: bool,
) -> impl IntoView {
    let show_airports = stats.unique_airports > 0;
    let show_stations = stats.unique_stations > 0;

    let airports_card = show_airports.then(|| {
        stat_card(
            "\u{1F6EB}",
            "Airports",
            stats.unique_airports.to_string(),
            false,
        )
    });
    let stations_card = show_stations.then(|| {
        stat_card(
            "\u{1F687}",
            "Stations",
            stats.unique_stations.to_string(),
            false,
        )
    });

    if compact {
        view! {
            <div class="stats-overview stats-overview-compact">
                <div class="stat-row">
                    {stat_card("\u{1F9ED}", "Total Journeys", stats.total_journeys.to_string(), true)}
                    {stat_card("\u{2708}\u{FE0F}", "Flights", stats.total_flights.to_string(), true)}
                    {stat_card("\u{1F4CF}", "Distance", distance, true)}
                    {stat_card("\u{1F30D}", "Countries", stats.unique_countries.to_string(), true)}
                </div>
                <details class="stats-more">
                    <summary class="stats-more-summary">"More stats"</summary>
                    <div class="stat-row">
                        {stat_card("\u{1F686}", "Rail", stats.total_rail.to_string(), false)}
                        {airports_card}
                        {stations_card}
                        {stat_card("\u{1F4C5}", "Years", year_range, false)}
                    </div>
                </details>
            </div>
        }
        .into_any()
    } else {
        view! {
            <div class="stats-overview">
                <div class="stat-row">
                    {stat_card("\u{1F9ED}", "Total Journeys", stats.total_journeys.to_string(), false)}
                    {stat_card("\u{2708}\u{FE0F}", "Flights", stats.total_flights.to_string(), false)}
                    {stat_card("\u{1F686}", "Rail", stats.total_rail.to_string(), false)}
                    {stat_card("\u{1F4CF}", "Distance", distance, false)}
                    {airports_card}
                    {stations_card}
                    {stat_card("\u{1F30D}", "Countries", stats.unique_countries.to_string(), false)}
                    {stat_card("\u{1F4C5}", "Years", year_range, false)}
                </div>
            </div>
        }
        .into_any()
    }
}

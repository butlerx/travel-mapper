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

#[component]
pub fn OverviewCards(stats: DetailedStats, distance: String, year_range: String) -> impl IntoView {
    let show_airports = stats.unique_airports > 0;
    let show_stations = stats.unique_stations > 0;

    view! {
        <div class="stats-overview">
            <div class="stat-row">
                <div class="stat-card">
                    <div class="stat-label">"Total Journeys"</div>
                    <div class="stat-value">{stats.total_journeys}</div>
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
                {if show_airports {
                    view! {
                        <div class="stat-card">
                            <div class="stat-label">"Airports"</div>
                            <div class="stat-value">{stats.unique_airports}</div>
                        </div>
                    }
                    .into_any()
                } else {
                    ().into_any()
                }}
                {if show_stations {
                    view! {
                        <div class="stat-card">
                            <div class="stat-label">"Stations"</div>
                            <div class="stat-value">{stats.unique_stations}</div>
                        </div>
                    }
                    .into_any()
                } else {
                    ().into_any()
                }}
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

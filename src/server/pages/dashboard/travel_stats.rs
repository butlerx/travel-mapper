use crate::{distance::haversine_km, server::routes::JourneyResponse};
use std::collections::HashSet;

/// Stats computed from a user's travel journeys.
#[derive(Default, Clone)]
pub(super) struct TravelStats {
    pub(super) total_journeys: usize,
    pub(super) total_flights: usize,
    pub(super) total_rail: usize,
    pub(super) total_distance_km: u64,
    pub(super) airports_visited: usize,
    pub(super) cities_visited: usize,
    pub(super) first_year: Option<String>,
    pub(super) last_year: Option<String>,
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn positive_km_to_u64(km: f64) -> u64 {
    km.max(0.0).trunc() as u64
}

pub(super) fn compute_stats(journeys: &[JourneyResponse]) -> TravelStats {
    let mut stats = TravelStats {
        total_journeys: journeys.len(),
        ..Default::default()
    };

    let mut places: HashSet<String> = HashSet::new();
    let mut years: Vec<&str> = Vec::new();

    for journey in journeys {
        match journey.travel_type.as_str() {
            "air" => stats.total_flights += 1,
            "rail" => stats.total_rail += 1,
            _ => {}
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

        places.insert(journey.origin_name.clone());
        places.insert(journey.dest_name.clone());

        if !journey.start_date.is_empty()
            && let Some(y) = journey.start_date.get(..4)
        {
            years.push(y);
        }
    }

    stats.cities_visited = places.len();
    // For airport count, count only places referenced by air journeys
    let mut airports: HashSet<String> = HashSet::new();
    for journey in journeys {
        if journey.travel_type.as_str() == "air" {
            airports.insert(journey.origin_name.clone());
            airports.insert(journey.dest_name.clone());
        }
    }
    stats.airports_visited = airports.len();

    years.sort_unstable();
    stats.first_year = years.first().map(|y| (*y).to_owned());
    stats.last_year = years.last().map(|y| (*y).to_owned());

    stats
}

pub(super) fn format_distance(km: u64) -> String {
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

pub(super) fn format_year_range(first: Option<&String>, last: Option<&String>) -> String {
    match (first, last) {
        (Some(f), Some(l)) if f == l => f.clone(),
        (Some(f), Some(l)) => format!("{f}\u{2013}{l}"),
        _ => "\u{2014}".to_owned(),
    }
}

use crate::server::routes::HopResponse;
use std::collections::HashSet;

/// Stats computed from a user's travel hops.
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

fn haversine_km(lat1: f64, lng1: f64, lat2: f64, lng2: f64) -> f64 {
    let r = 6371.0_f64;
    let d_lat = (lat2 - lat1).to_radians();
    let d_lng = (lng2 - lng1).to_radians();
    let a = (d_lat / 2.0).sin().powi(2)
        + lat1.to_radians().cos() * lat2.to_radians().cos() * (d_lng / 2.0).sin().powi(2);
    r * 2.0 * a.sqrt().atan2((1.0 - a).sqrt())
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn positive_km_to_u64(km: f64) -> u64 {
    km.max(0.0).trunc() as u64
}

pub(super) fn compute_stats(hops: &[HopResponse]) -> TravelStats {
    let mut stats = TravelStats {
        total_journeys: hops.len(),
        ..Default::default()
    };

    let mut places: HashSet<String> = HashSet::new();
    let mut years: Vec<&str> = Vec::new();

    for hop in hops {
        match hop.travel_type.as_str() {
            "air" => stats.total_flights += 1,
            "rail" => stats.total_rail += 1,
            _ => {}
        }

        if hop.origin_lat != 0.0
            || hop.origin_lng != 0.0
            || hop.dest_lat != 0.0
            || hop.dest_lng != 0.0
        {
            let km = haversine_km(hop.origin_lat, hop.origin_lng, hop.dest_lat, hop.dest_lng);
            if km.is_finite() && km > 0.0 {
                stats.total_distance_km += positive_km_to_u64(km);
            }
        }

        places.insert(hop.origin_name.clone());
        places.insert(hop.dest_name.clone());

        if !hop.start_date.is_empty()
            && let Some(y) = hop.start_date.get(..4)
        {
            years.push(y);
        }
    }

    stats.cities_visited = places.len();
    // For airport count, count only places referenced by air hops
    let mut airports: HashSet<String> = HashSet::new();
    for hop in hops {
        if hop.travel_type.as_str() == "air" {
            airports.insert(hop.origin_name.clone());
            airports.insert(hop.dest_name.clone());
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

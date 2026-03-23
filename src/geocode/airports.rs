//! Embedded IATA airport database — coordinate and metadata lookups by
//! three-letter airport code.

use airport_data::AirportData;
use std::sync::LazyLock;

static DB: LazyLock<AirportData> = LazyLock::new(AirportData::new);

/// Enriched airport information from the embedded IATA database.
#[derive(Debug, Clone)]
pub struct Airport {
    pub iata: String,
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub country_code: String,
    pub timezone: String,
}

/// Look up enriched airport data by IATA code.
///
/// Returns `None` when the code is not found in the database.
#[must_use]
pub fn lookup_enriched(iata: &str) -> Option<Airport> {
    if iata.is_empty() {
        return None;
    }
    DB.get_airport_by_iata(iata).ok().map(|a| Airport {
        iata: a.iata.clone(),
        name: a.airport.clone(),
        latitude: a.latitude,
        longitude: a.longitude,
        country_code: a.country_code.to_lowercase(),
        timezone: a.timezone.clone(),
    })
}

/// Look up coordinates by IATA code.
///
/// TODO: rework this to return a custom error type instead of silently returning `None` for
/// invalid codes, and to avoid redundant lookups when both coordinates and metadata are needed.
/// Backward-compatible wrapper around [`lookup_enriched`] that returns only
/// the latitude/longitude pair.
#[must_use]
pub fn lookup(iata: &str) -> Option<(f64, f64)> {
    lookup_enriched(iata).map(|a| (a.latitude, a.longitude))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_airports_resolve() {
        let dub = lookup("DUB").expect("DUB not found");
        assert!((dub.0 - 53.4213).abs() < 0.1);
        assert!((dub.1 - (-6.2701)).abs() < 0.1);

        let jfk = lookup("JFK").expect("JFK not found");
        assert!((jfk.0 - 40.6413).abs() < 0.1);

        let lhr = lookup("LHR").expect("LHR not found");
        assert!((lhr.0 - 51.4706).abs() < 0.1);
    }

    #[test]
    fn unknown_code_returns_none() {
        assert!(lookup("ZZZ").is_none());
        assert!(lookup("").is_none());
    }

    #[test]
    fn enriched_lookup_returns_country_and_timezone() {
        let dub = lookup_enriched("DUB").expect("DUB not found");
        assert_eq!(dub.country_code, "ie");
        assert_eq!(dub.timezone, "Europe/Dublin");
        assert!(!dub.name.is_empty());

        let jfk = lookup_enriched("JFK").expect("JFK not found");
        assert_eq!(jfk.country_code, "us");
        assert!(jfk.timezone.starts_with("America/"));

        let nrt = lookup_enriched("NRT").expect("NRT not found");
        assert_eq!(nrt.country_code, "jp");
        assert_eq!(nrt.timezone, "Asia/Tokyo");
    }

    #[test]
    fn enriched_lookup_unknown_returns_none() {
        assert!(lookup_enriched("ZZZ").is_none());
        assert!(lookup_enriched("").is_none());
    }
}

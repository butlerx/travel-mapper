//! Embedded IATA airport database — coordinate and metadata lookups by
//! three-letter airport code.

use airport_data::AirportData;
use std::sync::LazyLock;

static DB: LazyLock<AirportData> = LazyLock::new(AirportData::new);

/// Enriched airport information from the embedded IATA database.
#[derive(Debug, Clone)]
#[allow(dead_code)] // fields populated from airport database; subset read by callers
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_airports_resolve() {
        let dub = lookup_enriched("DUB").expect("DUB not found");
        assert!((dub.latitude - 53.4213).abs() < 0.1);
        assert!((dub.longitude - (-6.2701)).abs() < 0.1);

        let jfk = lookup_enriched("JFK").expect("JFK not found");
        assert!((jfk.latitude - 40.6413).abs() < 0.1);

        let lhr = lookup_enriched("LHR").expect("LHR not found");
        assert!((lhr.latitude - 51.4706).abs() < 0.1);
    }

    #[test]
    fn unknown_code_returns_none() {
        assert!(lookup_enriched("ZZZ").is_none());
        assert!(lookup_enriched("").is_none());
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

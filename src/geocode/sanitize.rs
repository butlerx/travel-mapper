//! String-cleaning utilities for location names and timezone-to-country
//! mapping used by the geocoding pipeline.

use regex::Regex;
use std::sync::LazyLock;

static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\(([A-Z]{3})\)").unwrap());

// D12 AN22, A96 K7W4, etc.
static EIRCODE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b[A-Z]\d{2}\s?[A-Z]{2}\d{2}\b").unwrap());
// 08820, 08004, 12345, etc.
static POSTCODE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\b\d{4,5}\b").unwrap());
// "Terminal 2A", "Terminal 1"
static TERMINAL: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)\bTerminal\s+\w+\b").unwrap());

/// Extract a 3-letter IATA airport code from parenthesized text,
/// e.g. `"Marrakech Menara Airport (RAK)"` → `Some("RAK")`.
#[must_use]
pub(super) fn extract_iata_code(name: &str) -> Option<&str> {
    RE.captures(name).map(|c| c.get(1).unwrap().as_str())
}

/// Strip noisy substrings that confuse Nominatim: Irish eircodes, European
/// postcodes, and terminal identifiers.
#[must_use]
pub(super) fn sanitize_location_name(name: &str) -> String {
    let s = EIRCODE.replace_all(name, "");
    let s = POSTCODE.replace_all(&s, "");
    let s = TERMINAL.replace_all(&s, "");

    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Return the last comma-separated segment as a city fallback,
/// e.g. `"Carrer del Poeta 45, Barcelona"` → `Some("Barcelona")`.
#[must_use]
pub(super) fn extract_city_segment(name: &str) -> Option<&str> {
    let last = name.rsplit(',').next()?.trim();
    if last.is_empty() || last == name.trim() {
        None
    } else {
        Some(last)
    }
}

/// Map an IANA timezone to its primary ISO 3166-1 alpha-2 country code.
///
/// Covers the 29 timezones observed in our `TripIt` data. Returns `None` for
/// unknown or ambiguous zones (e.g. `UTC`).
#[must_use]
pub(super) fn tz_to_country(tz: &str) -> Option<&'static str> {
    match tz {
        "Africa/Casablanca" => Some("ma"),
        "America/Chicago" | "America/Denver" | "America/Los_Angeles" | "America/New_York" => {
            Some("us")
        }
        "America/Vancouver" => Some("ca"),
        "Asia/Dubai" => Some("ae"),
        "Asia/Nicosia" => Some("cy"),
        "Asia/Seoul" => Some("kr"),
        "Asia/Tokyo" => Some("jp"),
        "Europe/Amsterdam" => Some("nl"),
        "Europe/Athens" => Some("gr"),
        "Europe/Berlin" => Some("de"),
        "Europe/Bratislava" => Some("sk"),
        "Europe/Brussels" => Some("be"),
        "Europe/Copenhagen" => Some("dk"),
        "Europe/Dublin" => Some("ie"),
        "Europe/Istanbul" => Some("tr"),
        "Europe/Lisbon" => Some("pt"),
        "Europe/London" => Some("gb"),
        "Europe/Madrid" => Some("es"),
        "Europe/Malta" => Some("mt"),
        "Europe/Moscow" => Some("ru"),
        "Europe/Paris" => Some("fr"),
        "Europe/Prague" => Some("cz"),
        "Europe/Rome" => Some("it"),
        "Europe/Stockholm" => Some("se"),
        "Europe/Warsaw" => Some("pl"),
        "Europe/Zurich" => Some("ch"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_iata_code_finds_parenthesized_code() {
        assert_eq!(
            extract_iata_code("Marrakech Menara Airport (RAK)"),
            Some("RAK")
        );
        assert_eq!(
            extract_iata_code("Barcelona El Prat Airport (BCN) Terminal 2"),
            Some("BCN")
        );
    }

    #[test]
    fn extract_iata_code_returns_none_for_no_match() {
        assert_eq!(extract_iata_code("Hotel Lobby"), None);
        assert_eq!(extract_iata_code("DUB"), None);
        assert_eq!(extract_iata_code("(ab)"), None);
        assert_eq!(extract_iata_code("(ABCD)"), None);
    }

    #[test]
    fn sanitize_location_name_strips_eircode() {
        assert_eq!(
            sanitize_location_name("Greenmount Avenue 1, Dublin 12 D12 AN22"),
            "Greenmount Avenue 1, Dublin 12"
        );
    }

    #[test]
    fn sanitize_location_name_strips_postcode() {
        assert_eq!(
            sanitize_location_name("El Prat de Llobregat 08820"),
            "El Prat de Llobregat"
        );
        assert_eq!(
            sanitize_location_name("Carrer del Poeta Cabanyes 45, Barcelona 08004"),
            "Carrer del Poeta Cabanyes 45, Barcelona"
        );
    }

    #[test]
    fn sanitize_location_name_strips_terminal() {
        assert_eq!(
            sanitize_location_name("Barcelona El Prat Airport Terminal 2A"),
            "Barcelona El Prat Airport"
        );
    }

    #[test]
    fn sanitize_location_name_handles_combined_noise() {
        assert_eq!(
            sanitize_location_name(
                "Barcelona El Prat Airport Terminal 2A El Prat de Llobregat 08820"
            ),
            "Barcelona El Prat Airport El Prat de Llobregat"
        );
    }

    #[test]
    fn extract_city_segment_returns_last_comma_part() {
        assert_eq!(
            extract_city_segment("Carrer del Poeta Cabanyes 45, Barcelona 08004"),
            Some("Barcelona 08004")
        );
        assert_eq!(
            extract_city_segment("Dar Najat, 30 Derb Lalla Chacha"),
            Some("30 Derb Lalla Chacha")
        );
    }

    #[test]
    fn extract_city_segment_returns_none_without_comma() {
        assert_eq!(extract_city_segment("Hotel Lobby"), None);
        assert_eq!(extract_city_segment(""), None);
    }

    #[test]
    fn tz_to_country_maps_known_timezones() {
        assert_eq!(tz_to_country("Europe/Dublin"), Some("ie"));
        assert_eq!(tz_to_country("America/New_York"), Some("us"));
        assert_eq!(tz_to_country("Europe/London"), Some("gb"));
        assert_eq!(tz_to_country("Asia/Tokyo"), Some("jp"));
        assert_eq!(tz_to_country("Africa/Casablanca"), Some("ma"));
    }

    #[test]
    fn tz_to_country_returns_none_for_unknown() {
        assert_eq!(tz_to_country("UTC"), None);
        assert_eq!(tz_to_country("Pacific/Auckland"), None);
        assert_eq!(tz_to_country(""), None);
    }
}

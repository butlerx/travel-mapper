//! Parse an iCalendar (`.ics`) document into journeys for import.
//!
//! Round-trips the feed this app emits (`routes/feed.rs`): `LOCATION` carries
//! `"ORIGIN → DEST"`, and `DESCRIPTION` carries `Type:`/`Carrier:` lines. Other
//! calendars are best-effort — any event with a parseable `origin → dest` route
//! and a start date is imported; everything else is skipped.

use crate::db::hops::TravelType;
use icalendar::{Calendar, Component};
use std::str::FromStr;

/// A journey extracted from a calendar event, ready to create.
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedJourney {
    pub date: String,
    pub origin: String,
    pub destination: String,
    pub travel_type: TravelType,
    pub carrier: String,
}

/// Parse ICS text into importable journeys. Unparseable or non-travel events
/// are skipped rather than erroring, so a single bad event can't fail an import.
#[must_use]
pub fn parse_ics(text: &str) -> Vec<ParsedJourney> {
    let Ok(calendar) = Calendar::from_str(text) else {
        return Vec::new();
    };

    calendar.events().filter_map(parse_event).collect()
}

fn parse_event(event: &icalendar::Event) -> Option<ParsedJourney> {
    let date = event_date(event.property_value("DTSTART")?)?;

    // Prefer the clean LOCATION route; fall back to the summary.
    let (origin, destination) = event
        .property_value("LOCATION")
        .and_then(extract_route)
        .or_else(|| event.property_value("SUMMARY").and_then(extract_route))?;

    let description = event.property_value("DESCRIPTION").unwrap_or_default();
    let travel_type = description_field(description, "Type")
        .and_then(|t| parse_travel_type(&t))
        .unwrap_or_else(|| infer_travel_type(&origin, &destination));
    let carrier = description_field(description, "Carrier").unwrap_or_default();

    Some(ParsedJourney {
        date,
        origin,
        destination,
        travel_type,
        carrier,
    })
}

/// Extract `YYYY-MM-DD` from an ICS date/date-time value (e.g. `20240601` or
/// `20240601T130000Z`).
fn event_date(dtstart: &str) -> Option<String> {
    let digits: String = dtstart.chars().take_while(char::is_ascii_digit).collect();
    if digits.len() < 8 {
        return None;
    }
    let date = chrono::NaiveDate::parse_from_str(&digits[..8], "%Y%m%d").ok()?;
    Some(date.format("%Y-%m-%d").to_string())
}

/// Split a `"ORIGIN → DEST"` style string into trimmed endpoints. Supports the
/// arrow we emit plus a couple of common separators, and strips a leading
/// `"emoji carrier:"` prefix that summaries carry.
fn extract_route(raw: &str) -> Option<(String, String)> {
    let separator = ["→", " — ", " - ", " to "]
        .into_iter()
        .find(|sep| raw.contains(sep))?;
    let (left, right) = raw.split_once(separator)?;

    // A summary like "✈️ BA: DUB → LHR" leaves "✈️ BA: DUB" on the left; keep
    // only the text after the last colon.
    let origin = left.rsplit(':').next().unwrap_or(left);
    let origin = clean_endpoint(origin);
    let destination = clean_endpoint(right);
    if origin.is_empty() || destination.is_empty() {
        return None;
    }
    Some((origin, destination))
}

/// Trim whitespace and any leading non-alphanumeric noise (emoji, arrows).
fn clean_endpoint(s: &str) -> String {
    s.trim()
        .trim_start_matches(|c: char| !c.is_alphanumeric())
        .trim()
        .to_string()
}

/// Find a `"Label: value"` line within an ICS description, tolerating both real
/// newlines and the literal `\n` escape that some serializers leave in.
fn description_field(description: &str, label: &str) -> Option<String> {
    let normalized = description.replace("\\n", "\n");
    let needle = format!("{label}:");
    normalized.lines().find_map(|line| {
        let line = line.trim();
        line.strip_prefix(&needle)
            .map(|rest| rest.trim().to_string())
    })
}

fn parse_travel_type(s: &str) -> Option<TravelType> {
    match s.trim().to_ascii_lowercase().as_str() {
        "air" | "flight" => Some(TravelType::Air),
        "rail" | "train" => Some(TravelType::Rail),
        "boat" | "ferry" => Some(TravelType::Boat),
        "transport" => Some(TravelType::Transport),
        _ => None,
    }
}

/// With no explicit type, treat a 3-letter all-caps pair as airports (air),
/// otherwise ground transport.
fn infer_travel_type(origin: &str, destination: &str) -> TravelType {
    let looks_iata = |s: &str| s.len() == 3 && s.chars().all(|c| c.is_ascii_uppercase());
    if looks_iata(origin) && looks_iata(destination) {
        TravelType::Air
    } else {
        TravelType::Transport
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wrap(event_body: &str) -> String {
        format!(
            "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//test//EN\r\nBEGIN:VEVENT\r\nUID:1@test\r\n{event_body}\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n"
        )
    }

    #[test]
    fn parses_our_own_feed_format() {
        let ics = wrap(
            "DTSTART:20240601\r\nSUMMARY:\u{2708}\u{fe0f} BA: DUB \u{2192} LHR\r\nLOCATION:DUB \u{2192} LHR\r\nDESCRIPTION:Type: air\\nCarrier: BA",
        );
        let parsed = parse_ics(&ics);
        assert_eq!(parsed.len(), 1);
        let j = &parsed[0];
        assert_eq!(j.date, "2024-06-01");
        assert_eq!(j.origin, "DUB");
        assert_eq!(j.destination, "LHR");
        assert_eq!(j.travel_type, TravelType::Air);
        assert_eq!(j.carrier, "BA");
    }

    #[test]
    fn parses_rail_from_description_type() {
        let ics = wrap(
            "DTSTART:20240815T080000Z\r\nLOCATION:London Euston \u{2192} Manchester Piccadilly\r\nDESCRIPTION:Type: rail\\nCarrier: Avanti",
        );
        let parsed = parse_ics(&ics);
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].travel_type, TravelType::Rail);
        assert_eq!(parsed[0].origin, "London Euston");
        assert_eq!(parsed[0].destination, "Manchester Piccadilly");
        assert_eq!(parsed[0].carrier, "Avanti");
    }

    #[test]
    fn infers_air_for_iata_pair_without_type() {
        let ics = wrap("DTSTART:20240601\r\nSUMMARY:SFO \u{2192} JFK");
        let parsed = parse_ics(&ics);
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].travel_type, TravelType::Air);
        assert_eq!(parsed[0].origin, "SFO");
        assert_eq!(parsed[0].destination, "JFK");
    }

    #[test]
    fn skips_events_without_a_route() {
        let ics = wrap("DTSTART:20240601\r\nSUMMARY:Dentist appointment");
        assert!(parse_ics(&ics).is_empty());
    }

    #[test]
    fn skips_events_without_a_date() {
        let ics = wrap("SUMMARY:DUB \u{2192} LHR");
        assert!(parse_ics(&ics).is_empty());
    }

    #[test]
    fn invalid_ics_returns_empty() {
        assert!(parse_ics("not a calendar").is_empty());
    }
}

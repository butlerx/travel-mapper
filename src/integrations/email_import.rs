//! Parse booking confirmation emails into importable journeys.
//!
//! Three-tier extraction strategy:
//! 1. **JSON-LD** — schema.org `FlightReservation` / `TrainReservation` embedded
//!    in `<script type="application/ld+json">` tags (highest confidence).
//! 2. **ICS attachment** — `.ics` calendar attachments parsed via [`super::ics_import::parse_ics`].
//! 3. **Regex heuristics** — pattern-matching for IATA codes, flight numbers, and
//!    dates in the email text body (lowest confidence, best-effort).

use crate::db::hops::TravelType;
use crate::integrations::ics_import::{self, ParsedJourney};
use mail_parser::{Message, MessageParser, MimeHeaders};
use regex::Regex;
use std::sync::LazyLock;

/// Parse a raw MIME email into importable journeys.
///
/// Tries JSON-LD first, then ICS attachments, then regex heuristics.
/// Returns an empty vec if nothing extractable is found.
#[must_use]
pub fn parse_email(raw: &[u8]) -> Vec<ParsedJourney> {
    let Some(message) = MessageParser::default().parse(raw) else {
        return Vec::new();
    };
    parse_message(&message)
}

/// Parse an already-decoded email message.
#[must_use]
pub fn parse_message(message: &Message) -> Vec<ParsedJourney> {
    // Tier 1: JSON-LD from HTML body
    if let Some(html) = message.body_html(0) {
        let results = extract_json_ld(&html);
        if !results.is_empty() {
            return results;
        }
    }

    // Tier 2: ICS attachments
    let ics_results = extract_ics_attachments(message);
    if !ics_results.is_empty() {
        return ics_results;
    }

    // Tier 3: Regex heuristics from text body
    if let Some(text) = message.body_text(0) {
        let results = extract_regex(&text);
        if !results.is_empty() {
            return results;
        }
    }

    // Also try HTML-to-text fallback for regex if no plain text part
    if let Some(html) = message.body_html(0) {
        let text = strip_html_tags(&html);
        return extract_regex(&text);
    }

    Vec::new()
}

// ---------------------------------------------------------------------------
// Tier 1: JSON-LD extraction
// ---------------------------------------------------------------------------

/// Extract journeys from schema.org JSON-LD in HTML `<script>` tags.
#[must_use]
pub fn extract_json_ld(html: &str) -> Vec<ParsedJourney> {
    let mut results = Vec::new();

    for json_str in find_json_ld_blocks(html) {
        let Ok(value) = serde_json::from_str::<serde_json::Value>(&json_str) else {
            continue;
        };
        extract_from_json_ld_value(&value, &mut results);
    }

    results
}

fn find_json_ld_blocks(html: &str) -> Vec<String> {
    static RE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(
            r#"(?is)<script[^>]*type\s*=\s*["']application/ld\+json["'][^>]*>(.*?)</script>"#,
        )
        .expect("json-ld regex")
    });

    RE.captures_iter(html)
        .filter_map(|cap| cap.get(1).map(|m| m.as_str().trim().to_string()))
        .collect()
}

fn extract_from_json_ld_value(value: &serde_json::Value, results: &mut Vec<ParsedJourney>) {
    match value {
        serde_json::Value::Array(arr) => {
            for item in arr {
                extract_from_json_ld_value(item, results);
            }
        }
        serde_json::Value::Object(obj) => {
            // Handle @graph wrapper
            if let Some(graph) = obj.get("@graph") {
                extract_from_json_ld_value(graph, results);
                return;
            }

            let type_field = obj
                .get("@type")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default();

            match type_field {
                "FlightReservation" => {
                    if let Some(j) = parse_flight_reservation(obj) {
                        results.push(j);
                    }
                }
                "TrainReservation" => {
                    if let Some(j) = parse_train_reservation(obj) {
                        results.push(j);
                    }
                }
                "BusReservation" => {
                    if let Some(j) = parse_bus_reservation(obj) {
                        results.push(j);
                    }
                }
                _ => {}
            }
        }
        _ => {}
    }
}

fn parse_flight_reservation(
    obj: &serde_json::Map<String, serde_json::Value>,
) -> Option<ParsedJourney> {
    let flight = obj.get("reservationFor")?.as_object()?;

    let dep_airport = flight.get("departureAirport")?.as_object()?;
    let arr_airport = flight.get("arrivalAirport")?.as_object()?;

    let origin = dep_airport
        .get("iataCode")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_else(|| {
            dep_airport
                .get("name")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default()
        })
        .to_string();

    let destination = arr_airport
        .get("iataCode")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_else(|| {
            arr_airport
                .get("name")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default()
        })
        .to_string();

    let date = flight
        .get("departureTime")
        .and_then(serde_json::Value::as_str)
        .and_then(extract_date_from_iso)?;

    let carrier = flight
        .get("provider")
        .or_else(|| flight.get("airline"))
        .and_then(serde_json::Value::as_object)
        .and_then(|a| {
            a.get("name")
                .or_else(|| a.get("iataCode"))
                .and_then(serde_json::Value::as_str)
        })
        .unwrap_or_default()
        .to_string();

    if origin.is_empty() || destination.is_empty() {
        return None;
    }

    Some(ParsedJourney {
        date,
        origin,
        destination,
        travel_type: TravelType::Air,
        carrier,
    })
}

fn parse_train_reservation(
    obj: &serde_json::Map<String, serde_json::Value>,
) -> Option<ParsedJourney> {
    let trip = obj.get("reservationFor")?.as_object()?;

    let dep_station = trip.get("departureStation")?.as_object()?;
    let arr_station = trip.get("arrivalStation")?.as_object()?;

    let origin = dep_station
        .get("name")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .to_string();

    let destination = arr_station
        .get("name")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .to_string();

    let date = trip
        .get("departureTime")
        .and_then(serde_json::Value::as_str)
        .and_then(extract_date_from_iso)?;

    let carrier = trip
        .get("provider")
        .and_then(serde_json::Value::as_object)
        .and_then(|p| p.get("name").and_then(serde_json::Value::as_str))
        .unwrap_or_default()
        .to_string();

    if origin.is_empty() || destination.is_empty() {
        return None;
    }

    Some(ParsedJourney {
        date,
        origin,
        destination,
        travel_type: TravelType::Rail,
        carrier,
    })
}

fn parse_bus_reservation(
    obj: &serde_json::Map<String, serde_json::Value>,
) -> Option<ParsedJourney> {
    let trip = obj.get("reservationFor")?.as_object()?;

    let dep = trip.get("departureBusStop")?.as_object()?;
    let arr = trip.get("arrivalBusStop")?.as_object()?;

    let origin = dep
        .get("name")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .to_string();

    let destination = arr
        .get("name")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .to_string();

    let date = trip
        .get("departureTime")
        .and_then(serde_json::Value::as_str)
        .and_then(extract_date_from_iso)?;

    let carrier = trip
        .get("provider")
        .and_then(serde_json::Value::as_object)
        .and_then(|p| p.get("name").and_then(serde_json::Value::as_str))
        .unwrap_or_default()
        .to_string();

    if origin.is_empty() || destination.is_empty() {
        return None;
    }

    Some(ParsedJourney {
        date,
        origin,
        destination,
        travel_type: TravelType::Transport,
        carrier,
    })
}

fn extract_date_from_iso(s: &str) -> Option<String> {
    // Accepts: "2027-03-04T20:15:00-08:00", "2027-03-04", "20270304T201500Z"
    if s.len() >= 10 && s.as_bytes()[4] == b'-' && s.as_bytes()[7] == b'-' {
        return Some(s[..10].to_string());
    }
    // Compact ISO: YYYYMMDD...
    if s.len() >= 8 && s.bytes().take(8).all(|b| b.is_ascii_digit()) {
        let y = &s[..4];
        let m = &s[4..6];
        let d = &s[6..8];
        return Some(format!("{y}-{m}-{d}"));
    }
    None
}

// ---------------------------------------------------------------------------
// Tier 2: ICS attachment extraction
// ---------------------------------------------------------------------------

fn extract_ics_attachments(message: &Message) -> Vec<ParsedJourney> {
    let mut results = Vec::new();

    for attachment in message.attachments() {
        let is_ics = attachment
            .content_type()
            .is_some_and(|ct| ct.ctype() == "text" && ct.subtype() == Some("calendar"))
            || attachment.attachment_name().is_some_and(|name| {
                std::path::Path::new(name)
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("ics"))
            });

        if is_ics && let Ok(text) = std::str::from_utf8(attachment.contents()) {
            results.extend(ics_import::parse_ics(text));
        }
    }

    results
}

// ---------------------------------------------------------------------------
// Tier 3: Regex heuristic extraction
// ---------------------------------------------------------------------------

static FLIGHT_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    // Match patterns like "EI 123", "UA1234", "BA 9876" (airline codes are uppercase)
    Regex::new(r"\b([A-Z]{2})\s?(\d{1,4})\b").expect("flight number regex")
});

static IATA_ROUTE: LazyLock<Regex> = LazyLock::new(|| {
    // Match patterns like "DUB → JFK", "LHR - CDG", "SFO to LAX"
    Regex::new(r"\b([A-Z]{3})\s*(?:→|->|—|–|-|to)\s*([A-Z]{3})\b").expect("iata route regex")
});

static DATE_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    // Match dates like "2027-03-04", "04 Mar 2027", "March 4, 2027", "4/3/2027"
    Regex::new(
        r"(?x)
        (\d{4}-\d{2}-\d{2})                              # ISO date
        | (\d{1,2})\s+(Jan|Feb|Mar|Apr|May|Jun|Jul|Aug|Sep|Oct|Nov|Dec)[a-z]*\s+(\d{4})  # DD Mon YYYY
        | (Jan|Feb|Mar|Apr|May|Jun|Jul|Aug|Sep|Oct|Nov|Dec)[a-z]*\s+(\d{1,2}),?\s+(\d{4}) # Mon DD, YYYY
        ",
    )
    .expect("date regex")
});

fn extract_regex(text: &str) -> Vec<ParsedJourney> {
    let mut results = Vec::new();

    // Find IATA route pairs
    let routes: Vec<(String, String)> = IATA_ROUTE
        .captures_iter(text)
        .map(|cap| (cap[1].to_uppercase(), cap[2].to_uppercase()))
        .collect();

    if routes.is_empty() {
        return results;
    }

    // Find dates in the text
    let dates = extract_dates_from_text(text);

    // Find flight numbers for carrier info
    let flight_numbers: Vec<String> = FLIGHT_PATTERN
        .captures_iter(text)
        .map(|cap| format!("{}{}", &cap[1].to_uppercase(), &cap[2]))
        .collect();

    // Match routes with the nearest date (or first date found)
    for (i, (origin, dest)) in routes.iter().enumerate() {
        let date = dates
            .get(i)
            .or_else(|| dates.first())
            .cloned()
            .unwrap_or_default();

        if date.is_empty() {
            continue;
        }

        let carrier = flight_numbers.get(i).cloned().unwrap_or_default();

        results.push(ParsedJourney {
            date,
            origin: origin.clone(),
            destination: dest.clone(),
            travel_type: TravelType::Air,
            carrier,
        });
    }

    results
}

fn extract_dates_from_text(text: &str) -> Vec<String> {
    DATE_PATTERN
        .captures_iter(text)
        .filter_map(|cap| {
            // ISO date: group 1
            if let Some(iso) = cap.get(1) {
                return Some(iso.as_str().to_string());
            }
            // DD Mon YYYY: groups 2, 3, 4
            if let (Some(day), Some(mon), Some(year)) = (cap.get(2), cap.get(3), cap.get(4)) {
                return format_date(year.as_str(), mon.as_str(), day.as_str());
            }
            // Mon DD, YYYY: groups 5, 6, 7
            if let (Some(mon), Some(day), Some(year)) = (cap.get(5), cap.get(6), cap.get(7)) {
                return format_date(year.as_str(), mon.as_str(), day.as_str());
            }
            None
        })
        .collect()
}

fn format_date(year: &str, month_name: &str, day: &str) -> Option<String> {
    let month_num = match &month_name.to_lowercase()[..3] {
        "jan" => "01",
        "feb" => "02",
        "mar" => "03",
        "apr" => "04",
        "may" => "05",
        "jun" => "06",
        "jul" => "07",
        "aug" => "08",
        "sep" => "09",
        "oct" => "10",
        "nov" => "11",
        "dec" => "12",
        _ => return None,
    };
    let day_num: u32 = day.parse().ok()?;
    Some(format!("{year}-{month_num}-{day_num:02}"))
}

fn strip_html_tags(html: &str) -> String {
    static TAG_RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"<[^>]+>").expect("html tag regex"));
    let text = TAG_RE.replace_all(html, " ");
    // Collapse whitespace
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_ld_flight_reservation() {
        let html = r#"
        <html><body>
        <script type="application/ld+json">
        {
            "@type": "FlightReservation",
            "reservationFor": {
                "@type": "Flight",
                "flightNumber": "110",
                "provider": {"@type": "Airline", "name": "United Airlines", "iataCode": "UA"},
                "departureAirport": {"@type": "Airport", "iataCode": "SFO"},
                "departureTime": "2027-03-04T20:15:00-08:00",
                "arrivalAirport": {"@type": "Airport", "iataCode": "JFK"},
                "arrivalTime": "2027-03-05T06:30:00-05:00"
            }
        }
        </script>
        </body></html>"#;

        let results = extract_json_ld(html);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].origin, "SFO");
        assert_eq!(results[0].destination, "JFK");
        assert_eq!(results[0].date, "2027-03-04");
        assert_eq!(results[0].travel_type, TravelType::Air);
        assert_eq!(results[0].carrier, "United Airlines");
    }

    #[test]
    fn json_ld_train_reservation() {
        let html = r#"
        <html><body>
        <script type="application/ld+json">
        {
            "@type": "TrainReservation",
            "reservationFor": {
                "@type": "TrainTrip",
                "trainNumber": "9012",
                "departureStation": {"@type": "TrainStation", "name": "London St Pancras"},
                "departureTime": "2027-04-05T08:30:00+01:00",
                "arrivalStation": {"@type": "TrainStation", "name": "Paris Gare du Nord"},
                "arrivalTime": "2027-04-05T11:47:00+02:00",
                "provider": {"@type": "Organization", "name": "Eurostar"}
            }
        }
        </script>
        </body></html>"#;

        let results = extract_json_ld(html);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].origin, "London St Pancras");
        assert_eq!(results[0].destination, "Paris Gare du Nord");
        assert_eq!(results[0].date, "2027-04-05");
        assert_eq!(results[0].travel_type, TravelType::Rail);
        assert_eq!(results[0].carrier, "Eurostar");
    }

    #[test]
    fn json_ld_array_multiple_flights() {
        let html = r#"
        <script type="application/ld+json">
        [
            {
                "@type": "FlightReservation",
                "reservationFor": {
                    "@type": "Flight",
                    "departureAirport": {"iataCode": "DUB"},
                    "departureTime": "2027-06-01T07:00:00Z",
                    "arrivalAirport": {"iataCode": "LHR"},
                    "provider": {"name": "Aer Lingus"}
                }
            },
            {
                "@type": "FlightReservation",
                "reservationFor": {
                    "@type": "Flight",
                    "departureAirport": {"iataCode": "LHR"},
                    "departureTime": "2027-06-01T14:00:00Z",
                    "arrivalAirport": {"iataCode": "JFK"},
                    "provider": {"name": "British Airways"}
                }
            }
        ]
        </script>"#;

        let results = extract_json_ld(html);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].origin, "DUB");
        assert_eq!(results[0].destination, "LHR");
        assert_eq!(results[1].origin, "LHR");
        assert_eq!(results[1].destination, "JFK");
    }

    #[test]
    fn json_ld_graph_wrapper() {
        let html = r#"
        <script type="application/ld+json">
        {
            "@context": "http://schema.org",
            "@graph": [
                {
                    "@type": "FlightReservation",
                    "reservationFor": {
                        "@type": "Flight",
                        "departureAirport": {"iataCode": "CDG"},
                        "departureTime": "2027-09-15T10:00:00Z",
                        "arrivalAirport": {"iataCode": "FCO"},
                        "provider": {"name": "Air France"}
                    }
                }
            ]
        }
        </script>"#;

        let results = extract_json_ld(html);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].origin, "CDG");
        assert_eq!(results[0].destination, "FCO");
    }

    #[test]
    fn json_ld_skips_incomplete_data() {
        let html = r#"
        <script type="application/ld+json">
        {"@type": "FlightReservation", "reservationFor": {"@type": "Flight"}}
        </script>"#;

        let results = extract_json_ld(html);
        assert!(results.is_empty());
    }

    #[test]
    fn ics_attachment_extraction() {
        let raw_email = b"From: booking@airline.com\r\n\
Subject: Your Flight Confirmation\r\n\
MIME-Version: 1.0\r\n\
Content-Type: multipart/mixed; boundary=\"boundary123\"\r\n\
\r\n\
--boundary123\r\n\
Content-Type: text/plain\r\n\
\r\n\
Your flight is confirmed.\r\n\
--boundary123\r\n\
Content-Type: text/calendar; charset=utf-8\r\n\
Content-Disposition: attachment; filename=\"booking.ics\"\r\n\
\r\n\
BEGIN:VCALENDAR\r\n\
VERSION:2.0\r\n\
BEGIN:VEVENT\r\n\
DTSTART:20270601T070000Z\r\n\
SUMMARY:DUB to LHR\r\n\
LOCATION:DUB \xe2\x86\x92 LHR\r\n\
DESCRIPTION:Type: air\\nCarrier: Ryanair\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n\
--boundary123--\r\n";

        let results = parse_email(raw_email);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].origin, "DUB");
        assert_eq!(results[0].destination, "LHR");
        assert_eq!(results[0].travel_type, TravelType::Air);
        assert_eq!(results[0].carrier, "Ryanair");
    }

    #[test]
    fn regex_iata_route_extraction() {
        let text = "Your flight DUB → JFK on 2027-03-15 is confirmed. Flight EI 105.";
        let results = extract_regex(text);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].origin, "DUB");
        assert_eq!(results[0].destination, "JFK");
        assert_eq!(results[0].date, "2027-03-15");
        assert_eq!(results[0].carrier, "EI105");
    }

    #[test]
    fn regex_date_formats() {
        let dates = extract_dates_from_text(
            "departing 15 Mar 2027, returning March 22, 2027, also 2027-04-01",
        );
        assert_eq!(dates.len(), 3);
        assert_eq!(dates[0], "2027-03-15");
        assert_eq!(dates[1], "2027-03-22");
        assert_eq!(dates[2], "2027-04-01");
    }

    #[test]
    fn full_email_json_ld_priority() {
        let raw_email = b"From: bookings@united.com\r\n\
Subject: Booking Confirmed\r\n\
Content-Type: text/html; charset=utf-8\r\n\
\r\n\
<html><body>\r\n\
<p>Your flight SFO to JFK on 2027-03-04</p>\r\n\
<script type=\"application/ld+json\">\r\n\
{\"@type\":\"FlightReservation\",\"reservationFor\":{\"@type\":\"Flight\",\r\n\
\"departureAirport\":{\"iataCode\":\"SFO\"},\"departureTime\":\"2027-03-04T20:15:00\",\r\n\
\"arrivalAirport\":{\"iataCode\":\"JFK\"},\"provider\":{\"name\":\"United\"}}}\r\n\
</script>\r\n\
</body></html>";

        let results = parse_email(raw_email);
        assert_eq!(results.len(), 1);
        // JSON-LD takes priority (we get structured carrier name)
        assert_eq!(results[0].carrier, "United");
    }

    #[test]
    fn extract_date_iso_format() {
        assert_eq!(
            extract_date_from_iso("2027-03-04T20:15:00-08:00"),
            Some("2027-03-04".to_string())
        );
        assert_eq!(
            extract_date_from_iso("2027-03-04"),
            Some("2027-03-04".to_string())
        );
        assert_eq!(
            extract_date_from_iso("20270304T201500Z"),
            Some("2027-03-04".to_string())
        );
    }

    #[test]
    fn empty_email_returns_nothing() {
        let results = parse_email(b"");
        assert!(results.is_empty());
    }

    #[test]
    fn plain_text_no_travel_returns_nothing() {
        let raw = b"From: friend@example.com\r\n\
Subject: Hello!\r\n\
Content-Type: text/plain\r\n\
\r\n\
Hey, how are you doing? Let's catch up soon.";

        let results = parse_email(raw);
        assert!(results.is_empty());
    }
}

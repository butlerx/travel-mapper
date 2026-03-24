//! Generic CSV/delimited import — auto-detects and parses flight history exports
//! from myFlightradar24, OpenFlights, App in the Air, and Flighty.

use serde::Deserialize;
use std::io::Read;
use thiserror::Error;

/// Errors that can occur when parsing a generic CSV import.
#[derive(Debug, Error)]
pub enum ImportError {
    #[error("failed to parse CSV: {0}")]
    Csv(#[from] csv::Error),
    #[error("failed to read input: {0}")]
    Io(#[from] std::io::Error),
    #[error("unable to detect import format from header row")]
    UnknownFormat,
}

/// Supported import formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportFormat {
    MyFlightradar24,
    OpenFlights,
    AppInTheAir,
    Flighty,
}

impl std::fmt::Display for ImportFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MyFlightradar24 => write!(f, "myflightradar24"),
            Self::OpenFlights => write!(f, "openflights"),
            Self::AppInTheAir => write!(f, "appintheair"),
            Self::Flighty => write!(f, "flighty"),
        }
    }
}

/// A normalised flight row produced by any format parser.
#[derive(Debug, Clone)]
pub struct GenericRow {
    pub date: String,
    pub from_iata: String,
    pub to_iata: String,
    pub flight_number: String,
    pub airline: String,
    pub aircraft_type: String,
    pub registration: String,
    pub seat: String,
    pub seat_type: String,
    pub cabin_class: String,
    pub flight_reason: String,
    pub note: String,
    pub dep_time: String,
    pub arr_time: String,
    pub pnr: String,
    pub source_format: ImportFormat,
    pub dep_terminal: Option<String>,
    pub dep_gate: Option<String>,
    pub arr_terminal: Option<String>,
    pub arr_gate: Option<String>,
    pub canceled: Option<bool>,
    pub diverted_to: Option<String>,
    pub gate_dep_scheduled: Option<String>,
    pub gate_dep_actual: Option<String>,
    pub takeoff_scheduled: Option<String>,
    pub takeoff_actual: Option<String>,
    pub landing_scheduled: Option<String>,
    pub landing_actual: Option<String>,
    pub gate_arr_scheduled: Option<String>,
    pub gate_arr_actual: Option<String>,
    pub tail_number: Option<String>,
    pub flighty_flight_id: Option<String>,
    pub airline_id: Option<String>,
    pub dep_airport_id: Option<String>,
    pub arr_airport_id: Option<String>,
    pub diverted_airport_id: Option<String>,
    pub aircraft_type_id: Option<String>,
}

/// Parse a flight-history export, auto-detecting the format unless overridden.
///
/// # Errors
///
/// Returns an error if the format cannot be detected, or if the data is
/// malformed or cannot be deserialized.
pub fn parse_csv<R: Read>(
    mut reader: R,
    format: Option<ImportFormat>,
) -> Result<Vec<GenericRow>, ImportError> {
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf)?;

    let content = strip_bom(&buf);

    let fmt = match format {
        Some(f) => f,
        None => detect_format(content)?,
    };

    match fmt {
        ImportFormat::MyFlightradar24 => parse_myflightradar24(content),
        ImportFormat::OpenFlights => parse_openflights(content),
        ImportFormat::AppInTheAir => Ok(parse_app_in_the_air(content)),
        ImportFormat::Flighty => parse_flighty(content),
    }
}

/// Strip a UTF-8 BOM if present.
fn strip_bom(data: &[u8]) -> &[u8] {
    data.strip_prefix(b"\xef\xbb\xbf").unwrap_or(data)
}

/// Detect the import format from the first line / overall structure.
fn detect_format(content: &[u8]) -> Result<ImportFormat, ImportError> {
    let text = std::str::from_utf8(content).unwrap_or("");
    let first_line = text.lines().next().unwrap_or("");

    if first_line.contains("Flight Flighty ID") {
        return Ok(ImportFormat::Flighty);
    }

    if first_line.contains("Dep_id") && first_line.contains("Arr_id") {
        return Ok(ImportFormat::MyFlightradar24);
    }

    if first_line.contains("From_OID") && first_line.contains("Plane_OID") {
        return Ok(ImportFormat::OpenFlights);
    }

    if first_line.starts_with("flights:") || is_semicolon_delimited(text) {
        return Ok(ImportFormat::AppInTheAir);
    }

    Err(ImportError::UnknownFormat)
}

/// Heuristic: semicolons as field separators with ~17 fields per line.
fn is_semicolon_delimited(text: &str) -> bool {
    let data_lines: Vec<&str> = text
        .lines()
        .filter(|l| !l.is_empty() && !l.ends_with(':'))
        .take(3)
        .collect();
    if data_lines.is_empty() {
        return false;
    }
    data_lines
        .iter()
        .all(|l| l.contains(';') && l.split(';').count() >= 15)
}

// ---------------------------------------------------------------------------
// myFlightradar24
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct MyFr24Row {
    #[serde(rename = "Date")]
    date: String,
    #[serde(rename = "Flight number")]
    flight_number: String,
    #[serde(rename = "From")]
    from: String,
    #[serde(rename = "To")]
    to: String,
    #[serde(rename = "Dep time")]
    dep_time: String,
    #[serde(rename = "Arr time")]
    arr_time: String,
    #[serde(rename = "Duration")]
    _duration: String,
    #[serde(rename = "Airline")]
    airline: String,
    #[serde(rename = "Aircraft")]
    aircraft: String,
    #[serde(rename = "Registration")]
    registration: String,
    #[serde(rename = "Seat number")]
    seat_number: String,
    #[serde(rename = "Seat type")]
    seat_type: String,
    #[serde(rename = "Flight class")]
    flight_class: String,
    #[serde(rename = "Flight reason")]
    flight_reason: String,
    #[serde(rename = "Note")]
    note: String,
    #[serde(rename = "Dep_id")]
    _dep_id: String,
    #[serde(rename = "Arr_id")]
    _arr_id: String,
    #[serde(rename = "Airline_id")]
    _airline_id: String,
    #[serde(rename = "Aircraft_id")]
    _aircraft_id: String,
}

/// Extract IATA code from myFlightradar24 airport format: `"LHR (EGLL)"` → `"LHR"`.
fn extract_iata_from_fr24(airport: &str) -> String {
    airport
        .split_once('(')
        .map_or(airport.trim(), |(iata, _)| iata.trim())
        .to_string()
}

/// Extract airline name from myFlightradar24 format: `"British Airways (BA/)"` → `"British Airways"`.
fn extract_airline_from_fr24(airline: &str) -> String {
    airline
        .split_once('(')
        .map_or(airline.trim(), |(name, _)| name.trim())
        .to_string()
}

fn fr24_seat_type(code: &str) -> &str {
    match code {
        "1" => "Window",
        "2" => "Middle",
        "3" => "Aisle",
        _ => "",
    }
}

fn fr24_flight_class(code: &str) -> &str {
    match code {
        "1" => "Economy",
        "2" => "Business",
        "3" => "First",
        "4" => "Economy+",
        "5" => "Private",
        _ => "",
    }
}

fn fr24_flight_reason(code: &str) -> &str {
    match code {
        "1" => "Leisure",
        "2" => "Business",
        "3" => "Crew",
        "4" => "Other",
        _ => "",
    }
}

fn parse_myflightradar24(content: &[u8]) -> Result<Vec<GenericRow>, ImportError> {
    let mut rdr = csv::Reader::from_reader(content);
    let mut rows = Vec::new();
    for result in rdr.deserialize::<MyFr24Row>() {
        let r = result?;
        rows.push(GenericRow {
            date: r.date,
            from_iata: extract_iata_from_fr24(&r.from),
            to_iata: extract_iata_from_fr24(&r.to),
            flight_number: r.flight_number,
            airline: extract_airline_from_fr24(&r.airline),
            aircraft_type: r.aircraft,
            registration: r.registration,
            seat: r.seat_number,
            seat_type: fr24_seat_type(&r.seat_type).to_string(),
            cabin_class: fr24_flight_class(&r.flight_class).to_string(),
            flight_reason: fr24_flight_reason(&r.flight_reason).to_string(),
            note: r.note,
            dep_time: r.dep_time,
            arr_time: r.arr_time,
            pnr: String::new(),
            source_format: ImportFormat::MyFlightradar24,
            dep_terminal: None,
            dep_gate: None,
            arr_terminal: None,
            arr_gate: None,
            canceled: None,
            diverted_to: None,
            gate_dep_scheduled: None,
            gate_dep_actual: None,
            takeoff_scheduled: None,
            takeoff_actual: None,
            landing_scheduled: None,
            landing_actual: None,
            gate_arr_scheduled: None,
            gate_arr_actual: None,
            tail_number: None,
            flighty_flight_id: None,
            airline_id: None,
            dep_airport_id: None,
            arr_airport_id: None,
            diverted_airport_id: None,
            aircraft_type_id: None,
        });
    }
    Ok(rows)
}

// ---------------------------------------------------------------------------
// OpenFlights
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct OpenFlightsRow {
    #[serde(rename = "Date")]
    date: String,
    #[serde(rename = "From")]
    from: String,
    #[serde(rename = "To")]
    to: String,
    #[serde(rename = "Flight_Number")]
    flight_number: String,
    #[serde(rename = "Airline")]
    airline: String,
    #[serde(rename = "Distance")]
    _distance: String,
    #[serde(rename = "Duration")]
    _duration: String,
    #[serde(rename = "Seat")]
    seat: String,
    #[serde(rename = "Seat_Type")]
    seat_type: String,
    #[serde(rename = "Class")]
    class: String,
    #[serde(rename = "Reason")]
    reason: String,
    #[serde(rename = "Plane")]
    plane: String,
    #[serde(rename = "Registration")]
    registration: String,
    #[serde(rename = "Trip")]
    _trip: String,
    #[serde(rename = "Note")]
    note: String,
    #[serde(rename = "From_OID")]
    _from_oid: String,
    #[serde(rename = "To_OID")]
    _to_oid: String,
    #[serde(rename = "Airline_OID")]
    _airline_oid: String,
    #[serde(rename = "Plane_OID")]
    _plane_oid: String,
}

fn openflights_seat_type(code: &str) -> &str {
    match code {
        "W" => "Window",
        "A" => "Aisle",
        "M" => "Middle",
        _ => "",
    }
}

fn openflights_class(code: &str) -> &str {
    match code {
        "F" => "First",
        "C" => "Business",
        "P" => "Economy+",
        "Y" => "Economy",
        _ => "",
    }
}

fn openflights_reason(code: &str) -> &str {
    match code {
        "B" => "Business",
        "L" => "Leisure",
        "C" => "Crew",
        "O" => "Other",
        _ => "",
    }
}

fn parse_openflights(content: &[u8]) -> Result<Vec<GenericRow>, ImportError> {
    let mut rdr = csv::Reader::from_reader(content);
    let mut rows = Vec::new();
    for result in rdr.deserialize::<OpenFlightsRow>() {
        let r = result?;
        rows.push(GenericRow {
            date: r.date,
            from_iata: r.from.trim().to_string(),
            to_iata: r.to.trim().to_string(),
            flight_number: r.flight_number,
            airline: r.airline,
            aircraft_type: r.plane,
            registration: r.registration,
            seat: r.seat,
            seat_type: openflights_seat_type(&r.seat_type).to_string(),
            cabin_class: openflights_class(&r.class).to_string(),
            flight_reason: openflights_reason(&r.reason).to_string(),
            note: r.note,
            dep_time: String::new(),
            arr_time: String::new(),
            pnr: String::new(),
            source_format: ImportFormat::OpenFlights,
            dep_terminal: None,
            dep_gate: None,
            arr_terminal: None,
            arr_gate: None,
            canceled: None,
            diverted_to: None,
            gate_dep_scheduled: None,
            gate_dep_actual: None,
            takeoff_scheduled: None,
            takeoff_actual: None,
            landing_scheduled: None,
            landing_actual: None,
            gate_arr_scheduled: None,
            gate_arr_actual: None,
            tail_number: None,
            flighty_flight_id: None,
            airline_id: None,
            dep_airport_id: None,
            arr_airport_id: None,
            diverted_airport_id: None,
            aircraft_type_id: None,
        });
    }
    Ok(rows)
}

// ---------------------------------------------------------------------------
// App in the Air
// ---------------------------------------------------------------------------

fn parse_app_in_the_air(content: &[u8]) -> Vec<GenericRow> {
    let text = std::str::from_utf8(content).unwrap_or("");
    let mut rows = Vec::new();

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.ends_with(':') {
            continue;
        }

        let fields: Vec<&str> = line.split(';').collect();
        if fields.len() < 17 {
            continue;
        }

        let dep_utc = fields[12].trim();
        let arr_utc = fields[13].trim();
        let date = dep_utc.split('T').next().unwrap_or("").to_string();

        rows.push(GenericRow {
            date,
            from_iata: fields[10].trim().to_string(),
            to_iata: fields[11].trim().to_string(),
            flight_number: format!("{}{}", fields[7].trim(), fields[8].trim()),
            airline: fields[7].trim().to_string(),
            aircraft_type: fields[9].trim().to_string(),
            registration: String::new(),
            seat: fields[1].trim().to_string(),
            seat_type: String::new(),
            cabin_class: fields[0].trim().to_string(),
            flight_reason: String::new(),
            note: String::new(),
            dep_time: dep_utc.to_string(),
            arr_time: arr_utc.to_string(),
            pnr: String::new(),
            source_format: ImportFormat::AppInTheAir,
            dep_terminal: None,
            dep_gate: None,
            arr_terminal: None,
            arr_gate: None,
            canceled: None,
            diverted_to: None,
            gate_dep_scheduled: None,
            gate_dep_actual: None,
            takeoff_scheduled: None,
            takeoff_actual: None,
            landing_scheduled: None,
            landing_actual: None,
            gate_arr_scheduled: None,
            gate_arr_actual: None,
            tail_number: None,
            flighty_flight_id: None,
            airline_id: None,
            dep_airport_id: None,
            arr_airport_id: None,
            diverted_airport_id: None,
            aircraft_type_id: None,
        });
    }

    rows
}

// ---------------------------------------------------------------------------
// Flighty
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct FlightyCsvRow {
    #[serde(rename = "Date")]
    date: String,
    #[serde(rename = "Airline")]
    airline: String,
    #[serde(rename = "Flight")]
    flight: String,
    #[serde(rename = "From")]
    from: String,
    #[serde(rename = "To")]
    to: String,
    #[serde(rename = "Dep Terminal")]
    dep_terminal: String,
    #[serde(rename = "Dep Gate")]
    dep_gate: String,
    #[serde(rename = "Arr Terminal")]
    arr_terminal: String,
    #[serde(rename = "Arr Gate")]
    arr_gate: String,
    #[serde(rename = "Canceled")]
    canceled: String,
    #[serde(rename = "Diverted To")]
    diverted_to: String,
    #[serde(rename = "Gate Departure (Scheduled)")]
    gate_dep_scheduled: String,
    #[serde(rename = "Gate Departure (Actual)")]
    gate_dep_actual: String,
    #[serde(rename = "Take off (Scheduled)")]
    takeoff_scheduled: String,
    #[serde(rename = "Take off (Actual)")]
    takeoff_actual: String,
    #[serde(rename = "Landing (Scheduled)")]
    landing_scheduled: String,
    #[serde(rename = "Landing (Actual)")]
    landing_actual: String,
    #[serde(rename = "Gate Arrival (Scheduled)")]
    gate_arr_scheduled: String,
    #[serde(rename = "Gate Arrival (Actual)")]
    gate_arr_actual: String,
    #[serde(rename = "Aircraft Type Name")]
    aircraft_type: String,
    #[serde(rename = "Tail Number")]
    tail_number: String,
    #[serde(rename = "PNR")]
    pnr: String,
    #[serde(rename = "Seat")]
    seat: String,
    #[serde(rename = "Seat Type")]
    seat_type: String,
    #[serde(rename = "Cabin Class")]
    cabin_class: String,
    #[serde(rename = "Flight Reason")]
    flight_reason: String,
    #[serde(rename = "Notes")]
    notes: String,
    #[serde(rename = "Flight Flighty ID")]
    flighty_flight_id: String,
    #[serde(rename = "Airline Flighty ID")]
    flighty_airline_id: String,
    #[serde(rename = "Departure Airport Flighty ID")]
    flighty_dep_airport_id: String,
    #[serde(rename = "Arrival Airport Flighty ID")]
    flighty_arr_airport_id: String,
    #[serde(rename = "Diverted To Airport Flighty ID")]
    flighty_diverted_airport_id: String,
    #[serde(rename = "Aircraft Type Flighty ID")]
    flighty_aircraft_type_id: String,
}

fn parse_canceled(s: &str) -> bool {
    s.eq_ignore_ascii_case("true")
}

fn non_empty(s: String) -> Option<String> {
    if s.is_empty() { None } else { Some(s) }
}

fn parse_flighty(content: &[u8]) -> Result<Vec<GenericRow>, ImportError> {
    let mut rdr = csv::Reader::from_reader(content);
    let mut rows = Vec::new();
    for result in rdr.deserialize::<FlightyCsvRow>() {
        let r = result?;
        rows.push(GenericRow {
            date: r.date,
            from_iata: r.from,
            to_iata: r.to,
            flight_number: r.flight,
            airline: r.airline,
            aircraft_type: r.aircraft_type,
            registration: r.tail_number.clone(),
            seat: r.seat,
            seat_type: r.seat_type,
            cabin_class: r.cabin_class,
            flight_reason: r.flight_reason,
            note: r.notes,
            dep_time: r.gate_dep_scheduled.clone(),
            arr_time: r.gate_arr_scheduled.clone(),
            pnr: r.pnr,
            source_format: ImportFormat::Flighty,
            dep_terminal: non_empty(r.dep_terminal),
            dep_gate: non_empty(r.dep_gate),
            arr_terminal: non_empty(r.arr_terminal),
            arr_gate: non_empty(r.arr_gate),
            canceled: Some(parse_canceled(&r.canceled)),
            diverted_to: non_empty(r.diverted_to),
            gate_dep_scheduled: non_empty(r.gate_dep_scheduled),
            gate_dep_actual: non_empty(r.gate_dep_actual),
            takeoff_scheduled: non_empty(r.takeoff_scheduled),
            takeoff_actual: non_empty(r.takeoff_actual),
            landing_scheduled: non_empty(r.landing_scheduled),
            landing_actual: non_empty(r.landing_actual),
            gate_arr_scheduled: non_empty(r.gate_arr_scheduled),
            gate_arr_actual: non_empty(r.gate_arr_actual),
            tail_number: non_empty(r.tail_number),
            flighty_flight_id: non_empty(r.flighty_flight_id),
            airline_id: non_empty(r.flighty_airline_id),
            dep_airport_id: non_empty(r.flighty_dep_airport_id),
            arr_airport_id: non_empty(r.flighty_arr_airport_id),
            diverted_airport_id: non_empty(r.flighty_diverted_airport_id),
            aircraft_type_id: non_empty(r.flighty_aircraft_type_id),
        });
    }
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    const FR24_CSV: &str = "\
Date,Flight number,From,To,Dep time,Arr time,Duration,Airline,Aircraft,Registration,Seat number,Seat type,Flight class,Flight reason,Note,Dep_id,Arr_id,Airline_id,Aircraft_id
2024-03-15,BA709,LHR (EGLL),DUB (EIDW),08:30,10:00,01:30,British Airways (BA/),Airbus A320,G-EUYL,12A,1,2,1,Nice flight,1234,5678,BA01,A320-01
2023-11-20,FR332,DUB (EIDW),STN (EGSS),06:15,07:30,01:15,Ryanair (FR/),Boeing 737-800,EI-DCL,,0,1,2,,2345,6789,FR01,B738-01";

    #[test]
    fn detect_myflightradar24_format() {
        let fmt = detect_format(FR24_CSV.as_bytes()).expect("detection failed");
        assert_eq!(fmt, ImportFormat::MyFlightradar24);
    }

    #[test]
    fn parse_myflightradar24_extracts_fields() {
        let rows = parse_csv(FR24_CSV.as_bytes(), None).expect("parse failed");
        assert_eq!(rows.len(), 2);

        let first = &rows[0];
        assert_eq!(first.date, "2024-03-15");
        assert_eq!(first.from_iata, "LHR");
        assert_eq!(first.to_iata, "DUB");
        assert_eq!(first.flight_number, "BA709");
        assert_eq!(first.airline, "British Airways");
        assert_eq!(first.aircraft_type, "Airbus A320");
        assert_eq!(first.registration, "G-EUYL");
        assert_eq!(first.seat, "12A");
        assert_eq!(first.seat_type, "Window");
        assert_eq!(first.cabin_class, "Business");
        assert_eq!(first.flight_reason, "Leisure");
        assert_eq!(first.note, "Nice flight");
        assert_eq!(first.dep_time, "08:30");
        assert_eq!(first.arr_time, "10:00");
        assert_eq!(first.source_format, ImportFormat::MyFlightradar24);
    }

    #[test]
    fn parse_myflightradar24_handles_empty_fields() {
        let rows = parse_csv(FR24_CSV.as_bytes(), None).expect("parse failed");
        let second = &rows[1];
        assert_eq!(second.seat, "");
        assert_eq!(second.seat_type, "");
        assert_eq!(second.cabin_class, "Economy");
        assert_eq!(second.flight_reason, "Business");
        assert_eq!(second.note, "");
    }

    const OPENFLIGHTS_CSV: &str = "\
Date,From,To,Flight_Number,Airline,Distance,Duration,Seat,Seat_Type,Class,Reason,Plane,Registration,Trip,Note,From_OID,To_OID,Airline_OID,Plane_OID
2024-06-01,DUB,JFK,EI105,Aer Lingus,5103,07:30,23A,W,Y,L,Airbus A330,EI-GAJ,Summer Trip,Transatlantic,1001,2002,EI01,A330-01
2024-06-15,JFK,DUB,EI106,Aer Lingus,5103,06:45,,,,B,,,,Return leg,2002,1001,EI01,";

    #[test]
    fn detect_openflights_format() {
        let fmt = detect_format(OPENFLIGHTS_CSV.as_bytes()).expect("detection failed");
        assert_eq!(fmt, ImportFormat::OpenFlights);
    }

    #[test]
    fn parse_openflights_extracts_fields() {
        let rows = parse_csv(OPENFLIGHTS_CSV.as_bytes(), None).expect("parse failed");
        assert_eq!(rows.len(), 2);

        let first = &rows[0];
        assert_eq!(first.date, "2024-06-01");
        assert_eq!(first.from_iata, "DUB");
        assert_eq!(first.to_iata, "JFK");
        assert_eq!(first.flight_number, "EI105");
        assert_eq!(first.airline, "Aer Lingus");
        assert_eq!(first.aircraft_type, "Airbus A330");
        assert_eq!(first.registration, "EI-GAJ");
        assert_eq!(first.seat, "23A");
        assert_eq!(first.seat_type, "Window");
        assert_eq!(first.cabin_class, "Economy");
        assert_eq!(first.flight_reason, "Leisure");
        assert_eq!(first.note, "Transatlantic");
        assert_eq!(first.dep_time, "");
        assert_eq!(first.arr_time, "");
        assert_eq!(first.source_format, ImportFormat::OpenFlights);
    }

    #[test]
    fn parse_openflights_handles_sparse_row() {
        let rows = parse_csv(OPENFLIGHTS_CSV.as_bytes(), None).expect("parse failed");
        let second = &rows[1];
        assert_eq!(second.seat, "");
        assert_eq!(second.seat_type, "");
        assert_eq!(second.cabin_class, "");
        assert_eq!(second.flight_reason, "Business");
    }

    const APPINTHEAIR_DATA: &str = "flights:
Economy;12A;ABC123;;flight;Y;manual;EI;105;A330;DUB;JFK;2024-06-01T10:30:00Z;2024-06-01T18:00:00Z;2024-06-01T11:30:00+01:00;2024-06-01T14:00:00-04:00;2024-05-20
Business;;;DEF456;flight;J;import;BA;709;A320;LHR;DUB;2024-03-15T07:30:00Z;2024-03-15T09:00:00Z;2024-03-15T07:30:00Z;2024-03-15T09:00:00Z;2024-03-01";

    #[test]
    fn detect_appintheair_format() {
        let fmt = detect_format(APPINTHEAIR_DATA.as_bytes()).expect("detection failed");
        assert_eq!(fmt, ImportFormat::AppInTheAir);
    }

    #[test]
    fn parse_appintheair_extracts_fields() {
        let rows = parse_csv(APPINTHEAIR_DATA.as_bytes(), None).expect("parse failed");
        assert_eq!(rows.len(), 2);

        let first = &rows[0];
        assert_eq!(first.date, "2024-06-01");
        assert_eq!(first.from_iata, "DUB");
        assert_eq!(first.to_iata, "JFK");
        assert_eq!(first.flight_number, "EI105");
        assert_eq!(first.airline, "EI");
        assert_eq!(first.aircraft_type, "A330");
        assert_eq!(first.seat, "12A");
        assert_eq!(first.cabin_class, "Economy");
        assert_eq!(first.dep_time, "2024-06-01T10:30:00Z");
        assert_eq!(first.arr_time, "2024-06-01T18:00:00Z");
        assert_eq!(first.source_format, ImportFormat::AppInTheAir);
    }

    #[test]
    fn parse_appintheair_handles_sparse_row() {
        let rows = parse_csv(APPINTHEAIR_DATA.as_bytes(), None).expect("parse failed");
        let second = &rows[1];
        assert_eq!(second.seat, "");
        assert_eq!(second.cabin_class, "Business");
        assert_eq!(second.from_iata, "LHR");
        assert_eq!(second.to_iata, "DUB");
    }

    #[test]
    fn parse_csv_empty_input_returns_empty() {
        let csv = "Date,Flight number,From,To,Dep time,Arr time,Duration,Airline,Aircraft,Registration,Seat number,Seat type,Flight class,Flight reason,Note,Dep_id,Arr_id,Airline_id,Aircraft_id\n";
        let rows =
            parse_csv(csv.as_bytes(), Some(ImportFormat::MyFlightradar24)).expect("parse failed");
        assert!(rows.is_empty());
    }

    #[test]
    fn unknown_format_returns_error() {
        let csv = "col1,col2,col3\na,b,c\n";
        let result = parse_csv(csv.as_bytes(), None);
        assert!(result.is_err());
    }

    #[test]
    fn bom_is_stripped() {
        let mut data = vec![0xef, 0xbb, 0xbf];
        data.extend_from_slice(FR24_CSV.as_bytes());
        let rows = parse_csv(data.as_slice(), None).expect("parse with BOM failed");
        assert_eq!(rows.len(), 2);
    }

    #[test]
    fn extract_iata_from_fr24_various_formats() {
        assert_eq!(extract_iata_from_fr24("LHR (EGLL)"), "LHR");
        assert_eq!(extract_iata_from_fr24("DUB"), "DUB");
        assert_eq!(extract_iata_from_fr24(" STN (EGSS) "), "STN");
    }

    #[test]
    fn extract_airline_from_fr24_various_formats() {
        assert_eq!(
            extract_airline_from_fr24("British Airways (BA/)"),
            "British Airways"
        );
        assert_eq!(extract_airline_from_fr24("Ryanair"), "Ryanair");
    }

    #[test]
    fn explicit_format_override_skips_detection() {
        let rows = parse_csv(OPENFLIGHTS_CSV.as_bytes(), Some(ImportFormat::OpenFlights))
            .expect("parse failed");
        assert_eq!(rows.len(), 2);
    }

    const FLIGHTY_CSV: &str = "\
Date,Airline,Flight,From,To,Dep Terminal,Dep Gate,Arr Terminal,Arr Gate,Canceled,Diverted To,Gate Departure (Scheduled),Gate Departure (Actual),Take off (Scheduled),Take off (Actual),Landing (Scheduled),Landing (Actual),Gate Arrival (Scheduled),Gate Arrival (Actual),Aircraft Type Name,Tail Number,PNR,Seat,Seat Type,Cabin Class,Flight Reason,Notes,Flight Flighty ID,Airline Flighty ID,Departure Airport Flighty ID,Arrival Airport Flighty ID,Diverted To Airport Flighty ID,Aircraft Type Flighty ID
2024-12-31,EIN,709,RAK,DUB,1,,2,302,false,,2024-12-31T17:55,2024-12-31T18:07,2024-12-31T18:05,2024-12-31T18:29,2024-12-31T20:25,2024-12-31T20:52,2024-12-31T20:35,,Airbus A320,EIGAL,2IYS7F,,,,,,1d00499b-cab2-4830-bacc-d8ed713b9074,a4a016d4-2a72-44e9-8708-6ed6a5a9d2d1,afd2bec1-df2f-4126-95b8-fa33971a2afe,5dc01a9f-6855-4248-8e7b-0fef83144078,,dd390a1f-e92a-4411-ab72-04db16d42030
2011-05-12,RYR,332,DUB,LTN,,,,,false,,2011-05-12T08:15,,,,,,2011-05-12T09:20,,,,FY1IJP,,,,,,cec8cb9f-3245-42f7-b864-cded4c4d8b1c,89ad0292-b411-46f2-a3ad-27896d1be60b,5dc01a9f-6855-4248-8e7b-0fef83144078,61ec3188-adbe-4481-ba0f-9ddcfa7df9b8,,";

    #[test]
    fn detect_flighty_format() {
        let fmt = detect_format(FLIGHTY_CSV.as_bytes()).expect("detection failed");
        assert_eq!(fmt, ImportFormat::Flighty);
    }

    #[test]
    fn parse_flighty_extracts_fields() {
        let rows = parse_csv(FLIGHTY_CSV.as_bytes(), None).expect("parse failed");
        assert_eq!(rows.len(), 2);

        let first = &rows[0];
        assert_eq!(first.date, "2024-12-31");
        assert_eq!(first.from_iata, "RAK");
        assert_eq!(first.to_iata, "DUB");
        assert_eq!(first.flight_number, "709");
        assert_eq!(first.airline, "EIN");
        assert_eq!(first.aircraft_type, "Airbus A320");
        assert_eq!(first.registration, "EIGAL");
        assert_eq!(first.seat, "");
        assert_eq!(first.note, "");
        assert_eq!(first.dep_time, "2024-12-31T17:55");
        assert_eq!(first.arr_time, "2024-12-31T20:35");
        assert_eq!(first.source_format, ImportFormat::Flighty);
        assert_eq!(first.dep_terminal.as_deref(), Some("1"));
        assert_eq!(first.dep_gate, None);
        assert_eq!(first.arr_terminal.as_deref(), Some("2"));
        assert_eq!(first.arr_gate.as_deref(), Some("302"));
        assert_eq!(first.canceled, Some(false));
        assert_eq!(first.diverted_to, None);
        assert_eq!(
            first.gate_dep_scheduled.as_deref(),
            Some("2024-12-31T17:55")
        );
        assert_eq!(first.gate_dep_actual.as_deref(), Some("2024-12-31T18:07"));
        assert_eq!(first.tail_number.as_deref(), Some("EIGAL"));
        assert_eq!(
            first.flighty_flight_id.as_deref(),
            Some("1d00499b-cab2-4830-bacc-d8ed713b9074")
        );
        assert_eq!(
            first.aircraft_type_id.as_deref(),
            Some("dd390a1f-e92a-4411-ab72-04db16d42030")
        );
    }

    #[test]
    fn parse_flighty_handles_sparse_row() {
        let rows = parse_csv(FLIGHTY_CSV.as_bytes(), None).expect("parse failed");
        let second = &rows[1];
        assert_eq!(second.date, "2011-05-12");
        assert_eq!(second.from_iata, "DUB");
        assert_eq!(second.to_iata, "LTN");
        assert_eq!(second.dep_terminal, None);
        assert_eq!(second.dep_gate, None);
        assert_eq!(second.arr_terminal, None);
        assert_eq!(second.arr_gate, None);
        assert_eq!(second.canceled, Some(false));
        assert_eq!(second.aircraft_type, "");
        assert_eq!(second.tail_number, None);
        assert_eq!(second.note, "");
    }

    #[test]
    fn parse_flighty_canceled_true() {
        let csv = "\
Date,Airline,Flight,From,To,Dep Terminal,Dep Gate,Arr Terminal,Arr Gate,Canceled,Diverted To,Gate Departure (Scheduled),Gate Departure (Actual),Take off (Scheduled),Take off (Actual),Landing (Scheduled),Landing (Actual),Gate Arrival (Scheduled),Gate Arrival (Actual),Aircraft Type Name,Tail Number,PNR,Seat,Seat Type,Cabin Class,Flight Reason,Notes,Flight Flighty ID,Airline Flighty ID,Departure Airport Flighty ID,Arrival Airport Flighty ID,Diverted To Airport Flighty ID,Aircraft Type Flighty ID
2024-01-01,RYR,100,DUB,LHR,,,,,true,,,,,,,,,,,,,,,,,,,,,,,";
        let rows = parse_csv(csv.as_bytes(), Some(ImportFormat::Flighty)).expect("parse failed");
        assert_eq!(rows[0].canceled, Some(true));
    }

    #[test]
    fn parse_flighty_empty_input_returns_empty() {
        let csv = "Date,Airline,Flight,From,To,Dep Terminal,Dep Gate,Arr Terminal,Arr Gate,Canceled,Diverted To,Gate Departure (Scheduled),Gate Departure (Actual),Take off (Scheduled),Take off (Actual),Landing (Scheduled),Landing (Actual),Gate Arrival (Scheduled),Gate Arrival (Actual),Aircraft Type Name,Tail Number,PNR,Seat,Seat Type,Cabin Class,Flight Reason,Notes,Flight Flighty ID,Airline Flighty ID,Departure Airport Flighty ID,Arrival Airport Flighty ID,Diverted To Airport Flighty ID,Aircraft Type Flighty ID\n";
        let rows = parse_csv(csv.as_bytes(), Some(ImportFormat::Flighty)).expect("parse failed");
        assert!(rows.is_empty());
    }
}

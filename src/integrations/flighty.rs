use serde::Deserialize;
use std::io::Read;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ImportError {
    #[error("failed to parse CSV: {0}")]
    Csv(#[from] csv::Error),
}

fn parse_canceled(s: &str) -> bool {
    s.eq_ignore_ascii_case("true")
}

#[derive(Debug, Clone)]
pub struct FlightRow {
    pub date: String,
    pub airline: String,
    pub flight_number: String,
    pub from: String,
    pub to: String,
    pub dep_terminal: String,
    pub dep_gate: String,
    pub arr_terminal: String,
    pub arr_gate: String,
    pub canceled: bool,
    pub diverted_to: String,
    pub gate_dep_scheduled: String,
    pub gate_dep_actual: String,
    pub takeoff_scheduled: String,
    pub takeoff_actual: String,
    pub landing_scheduled: String,
    pub landing_actual: String,
    pub gate_arr_scheduled: String,
    pub gate_arr_actual: String,
    pub aircraft_type: String,
    pub tail_number: String,
    pub pnr: String,
    pub seat: String,
    pub seat_type: String,
    pub cabin_class: String,
    pub flight_reason: String,
    pub notes: String,
    pub flighty_flight_id: String,
    pub airline_id: String,
    pub dep_airport_id: String,
    pub arr_airport_id: String,
    pub diverted_airport_id: String,
    pub aircraft_type_id: String,
}

#[derive(Debug, Deserialize)]
struct CsvRow {
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

impl From<CsvRow> for FlightRow {
    fn from(r: CsvRow) -> Self {
        Self {
            date: r.date,
            airline: r.airline,
            flight_number: r.flight,
            from: r.from,
            to: r.to,
            dep_terminal: r.dep_terminal,
            dep_gate: r.dep_gate,
            arr_terminal: r.arr_terminal,
            arr_gate: r.arr_gate,
            canceled: parse_canceled(&r.canceled),
            diverted_to: r.diverted_to,
            gate_dep_scheduled: r.gate_dep_scheduled,
            gate_dep_actual: r.gate_dep_actual,
            takeoff_scheduled: r.takeoff_scheduled,
            takeoff_actual: r.takeoff_actual,
            landing_scheduled: r.landing_scheduled,
            landing_actual: r.landing_actual,
            gate_arr_scheduled: r.gate_arr_scheduled,
            gate_arr_actual: r.gate_arr_actual,
            aircraft_type: r.aircraft_type,
            tail_number: r.tail_number,
            pnr: r.pnr,
            seat: r.seat,
            seat_type: r.seat_type,
            cabin_class: r.cabin_class,
            flight_reason: r.flight_reason,
            notes: r.notes,
            flighty_flight_id: r.flighty_flight_id,
            airline_id: r.flighty_airline_id,
            dep_airport_id: r.flighty_dep_airport_id,
            arr_airport_id: r.flighty_arr_airport_id,
            diverted_airport_id: r.flighty_diverted_airport_id,
            aircraft_type_id: r.flighty_aircraft_type_id,
        }
    }
}

/// # Errors
///
/// Returns an error if the CSV data is malformed or cannot be deserialized.
pub fn parse_csv<R: Read>(reader: R) -> Result<Vec<FlightRow>, ImportError> {
    let mut rdr = csv::Reader::from_reader(reader);
    let mut rows = Vec::new();
    for result in rdr.deserialize::<CsvRow>() {
        rows.push(FlightRow::from(result?));
    }
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_CSV: &str = "\
Date,Airline,Flight,From,To,Dep Terminal,Dep Gate,Arr Terminal,Arr Gate,Canceled,Diverted To,Gate Departure (Scheduled),Gate Departure (Actual),Take off (Scheduled),Take off (Actual),Landing (Scheduled),Landing (Actual),Gate Arrival (Scheduled),Gate Arrival (Actual),Aircraft Type Name,Tail Number,PNR,Seat,Seat Type,Cabin Class,Flight Reason,Notes,Flight Flighty ID,Airline Flighty ID,Departure Airport Flighty ID,Arrival Airport Flighty ID,Diverted To Airport Flighty ID,Aircraft Type Flighty ID
2024-12-31,EIN,709,RAK,DUB,1,,2,302,false,,2024-12-31T17:55,2024-12-31T18:07,2024-12-31T18:05,2024-12-31T18:29,2024-12-31T20:25,2024-12-31T20:52,2024-12-31T20:35,,Airbus A320,EIGAL,2IYS7F,,,,,,1d00499b-cab2-4830-bacc-d8ed713b9074,a4a016d4-2a72-44e9-8708-6ed6a5a9d2d1,afd2bec1-df2f-4126-95b8-fa33971a2afe,5dc01a9f-6855-4248-8e7b-0fef83144078,,dd390a1f-e92a-4411-ab72-04db16d42030
2011-05-12,RYR,332,DUB,LTN,,,,,false,,2011-05-12T08:15,,,,,,2011-05-12T09:20,,,,FY1IJP,,,,,,cec8cb9f-3245-42f7-b864-cded4c4d8b1c,89ad0292-b411-46f2-a3ad-27896d1be60b,5dc01a9f-6855-4248-8e7b-0fef83144078,61ec3188-adbe-4481-ba0f-9ddcfa7df9b8,,";

    #[test]
    fn parse_csv_extracts_all_fields() {
        let rows = parse_csv(SAMPLE_CSV.as_bytes()).expect("parse failed");
        assert_eq!(rows.len(), 2);

        let first = &rows[0];
        assert_eq!(first.date, "2024-12-31");
        assert_eq!(first.airline, "EIN");
        assert_eq!(first.flight_number, "709");
        assert_eq!(first.from, "RAK");
        assert_eq!(first.to, "DUB");
        assert_eq!(first.dep_terminal, "1");
        assert_eq!(first.dep_gate, "");
        assert_eq!(first.arr_terminal, "2");
        assert_eq!(first.arr_gate, "302");
        assert!(!first.canceled);
        assert_eq!(first.diverted_to, "");
        assert_eq!(first.gate_dep_scheduled, "2024-12-31T17:55");
        assert_eq!(first.gate_dep_actual, "2024-12-31T18:07");
        assert_eq!(first.aircraft_type, "Airbus A320");
        assert_eq!(first.tail_number, "EIGAL");
        assert_eq!(first.pnr, "2IYS7F");
        assert_eq!(first.seat, "");
        assert_eq!(
            first.flighty_flight_id,
            "1d00499b-cab2-4830-bacc-d8ed713b9074"
        );
    }

    #[test]
    fn parse_csv_handles_sparse_row() {
        let rows = parse_csv(SAMPLE_CSV.as_bytes()).expect("parse failed");
        let second = &rows[1];
        assert_eq!(second.date, "2011-05-12");
        assert_eq!(second.from, "DUB");
        assert_eq!(second.to, "LTN");
        assert_eq!(second.dep_terminal, "");
        assert_eq!(second.dep_gate, "");
        assert_eq!(second.arr_terminal, "");
        assert_eq!(second.arr_gate, "");
        assert!(!second.canceled);
        assert_eq!(second.aircraft_type, "");
        assert_eq!(second.tail_number, "");
        assert_eq!(second.pnr, "FY1IJP");
    }

    #[test]
    fn parse_csv_canceled_true() {
        let csv = "\
Date,Airline,Flight,From,To,Dep Terminal,Dep Gate,Arr Terminal,Arr Gate,Canceled,Diverted To,Gate Departure (Scheduled),Gate Departure (Actual),Take off (Scheduled),Take off (Actual),Landing (Scheduled),Landing (Actual),Gate Arrival (Scheduled),Gate Arrival (Actual),Aircraft Type Name,Tail Number,PNR,Seat,Seat Type,Cabin Class,Flight Reason,Notes,Flight Flighty ID,Airline Flighty ID,Departure Airport Flighty ID,Arrival Airport Flighty ID,Diverted To Airport Flighty ID,Aircraft Type Flighty ID
2024-01-01,RYR,100,DUB,LHR,,,,,true,,,,,,,,,,,,,,,,,,,,,,,";
        let rows = parse_csv(csv.as_bytes()).expect("parse failed");
        assert!(rows[0].canceled);
    }

    #[test]
    fn parse_csv_empty_input_returns_empty() {
        let csv = "Date,Airline,Flight,From,To,Dep Terminal,Dep Gate,Arr Terminal,Arr Gate,Canceled,Diverted To,Gate Departure (Scheduled),Gate Departure (Actual),Take off (Scheduled),Take off (Actual),Landing (Scheduled),Landing (Actual),Gate Arrival (Scheduled),Gate Arrival (Actual),Aircraft Type Name,Tail Number,PNR,Seat,Seat Type,Cabin Class,Flight Reason,Notes,Flight Flighty ID,Airline Flighty ID,Departure Airport Flighty ID,Arrival Airport Flighty ID,Diverted To Airport Flighty ID,Aircraft Type Flighty ID\n";
        let rows = parse_csv(csv.as_bytes()).expect("parse failed");
        assert!(rows.is_empty());
    }
}

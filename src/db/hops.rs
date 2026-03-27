//! Query objects for the `hops` table — individual travel legs.
//!
//! Each command struct lives in its own submodule and is re-exported here
//! so that callers continue to use `db::hops::Create`, `db::hops::Row`, etc.

mod create;
/// Insert or replace hops from a generic CSV/delimited flight history import.
mod create_from_csv;
mod create_manual;
mod delete_for_trip;
mod delete_stale;
mod exists_in_trip;
mod get_all;
mod get_all_for_stats;
mod get_by_id;
mod get_for_trip;
mod get_unassigned;
mod replace_for_trip;
/// Search hops with optional filters across all travel types and detail tables.
mod search;
/// Update an existing hop's core fields and type-specific details by ID.
mod update_by_id;

pub use create::Create;
pub use create_from_csv::CreateFromCsv;
pub use create_manual::CreateManual;
pub use delete_for_trip::DeleteForTrip;
pub use delete_stale::DeleteStaleTripItTrips;
pub use exists_in_trip::ExistsInTrip;
pub use get_all::GetAll;
pub use get_all_for_stats::{GetAllForStats, StatsRow};
pub use get_by_id::{DetailRow, FullFlightDetail, GetById};
pub use get_for_trip::GetForTrip;
pub use get_unassigned::GetUnassigned;
pub use replace_for_trip::ReplaceForTrip;
pub use search::Search;
pub use update_by_id::UpdateById;

/// Type-specific detail payload for manually-created hops.
#[derive(Debug, Clone)]
pub enum ManualDetail {
    Air(FlightDetail),
    Rail(RailDetail),
    Boat(BoatDetail),
    Transport(TransportDetail),
}

// ---------------------------------------------------------------------------
// Shared types — travel type enums, detail structs, and row mappings
// ---------------------------------------------------------------------------

/// The type of travel for a hop.
#[derive(Debug, Clone, PartialEq)]
pub enum TravelType {
    Air,
    Rail,
    Boat,
    Transport,
}

impl std::fmt::Display for TravelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Air => write!(f, "air"),
            Self::Rail => write!(f, "rail"),
            Self::Boat => write!(f, "boat"),
            Self::Transport => write!(f, "transport"),
        }
    }
}

impl TravelType {
    /// Returns the emoji representing this travel type.
    #[must_use]
    pub const fn emoji(&self) -> &'static str {
        match self {
            Self::Air => "\u{2708}\u{fe0f}",
            Self::Rail => "\u{1f686}",
            Self::Boat => "\u{1f6a2}",
            Self::Transport => "\u{1f697}",
        }
    }
}

/// Flight-specific metadata for an air hop.
#[derive(Debug, Clone, Default)]
pub struct FlightDetail {
    pub airline: String,
    pub flight_number: String,
    pub aircraft_type: String,
    pub cabin_class: String,
    pub seat: String,
    pub pnr: String,
}

/// Rail-specific metadata for a train hop.
#[derive(Debug, Clone, Default)]
pub struct RailDetail {
    pub carrier: String,
    pub train_number: String,
    pub service_class: String,
    pub coach_number: String,
    pub seats: String,
    pub confirmation_num: String,
    pub booking_site: String,
    pub notes: String,
}

/// Boat-specific metadata for a sea hop.
#[derive(Debug, Clone, Default)]
pub struct BoatDetail {
    pub ship_name: String,
    pub cabin_type: String,
    pub cabin_number: String,
    pub confirmation_num: String,
    pub booking_site: String,
    pub notes: String,
}

/// Ground-transport metadata for a car, bus, or shuttle hop.
#[derive(Debug, Clone, Default)]
pub struct TransportDetail {
    pub carrier_name: String,
    pub vehicle_description: String,
    pub confirmation_num: String,
    pub notes: String,
}

/// A single origin -> destination travel hop.
#[derive(Debug, Clone)]
pub struct Row {
    pub id: i64,
    pub travel_type: TravelType,
    pub origin_name: String,
    pub origin_lat: f64,
    pub origin_lng: f64,
    pub origin_country: Option<String>,
    pub dest_name: String,
    pub dest_lat: f64,
    pub dest_lng: f64,
    pub dest_country: Option<String>,
    pub start_date: String,
    pub end_date: String,
    pub raw_json: Option<String>,
    /// Transient — not persisted to DB.
    pub origin_address_query: Option<String>,
    /// Transient — not persisted to DB.
    pub dest_address_query: Option<String>,
    /// Transient IANA timezone from `StartDateTime.timezone` — not persisted to DB.
    pub origin_tz: Option<String>,
    /// Transient IANA timezone from `EndDateTime.timezone` — not persisted to DB.
    pub dest_tz: Option<String>,
    pub flight_detail: Option<FlightDetail>,
    pub rail_detail: Option<RailDetail>,
    pub boat_detail: Option<BoatDetail>,
    pub transport_detail: Option<TransportDetail>,
    pub cost_amount: Option<f64>,
    pub cost_currency: Option<String>,
    pub loyalty_program: Option<String>,
    pub miles_earned: Option<f64>,
    /// Carrier name from SQL COALESCE — used as fallback when detail structs
    /// are not loaded (e.g. listing queries).  Not a domain field; populated
    /// only by `TryFrom<HopRow>`.  External constructors should set this to
    /// `None` — prefer [`Row::carrier()`] for reads.
    pub cached_carrier: Option<String>,
}

impl Row {
    #[must_use]
    pub fn carrier(&self) -> Option<&str> {
        self.flight_detail
            .as_ref()
            .map(|d| d.airline.as_str())
            .or_else(|| self.rail_detail.as_ref().map(|d| d.carrier.as_str()))
            .or_else(|| self.boat_detail.as_ref().map(|d| d.ship_name.as_str()))
            .or_else(|| {
                self.transport_detail
                    .as_ref()
                    .map(|d| d.carrier_name.as_str())
            })
            .or(self.cached_carrier.as_deref())
            .filter(|s| !s.is_empty())
    }
}

// ---------------------------------------------------------------------------
// Internal helpers for row mapping
// ---------------------------------------------------------------------------

/// Internal row type for sqlx `query_as!` macro (`SQLite` stores `travel_type` as text).
pub(super) struct HopRow {
    pub id: i64,
    pub travel_type: String,
    pub origin_name: String,
    pub origin_lat: f64,
    pub origin_lng: f64,
    pub origin_country: Option<String>,
    pub dest_name: String,
    pub dest_lat: f64,
    pub dest_lng: f64,
    pub dest_country: Option<String>,
    pub start_date: String,
    pub end_date: String,
    pub carrier: Option<String>,
    pub cost_amount: Option<f64>,
    pub cost_currency: Option<String>,
    pub loyalty_program: Option<String>,
    pub miles_earned: Option<f64>,
}

/// Lightweight summary of a hop — used for trip detail listings and
/// unassigned-hop dropdowns where full coordinate data is unnecessary.
pub struct SummaryRow {
    pub id: i64,
    pub travel_type: String,
    pub origin_name: String,
    pub dest_name: String,
    pub start_date: String,
    pub carrier: Option<String>,
}

impl TryFrom<HopRow> for Row {
    type Error = sqlx::Error;

    fn try_from(row: HopRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.id,
            travel_type: parse_travel_type(&row.travel_type)?,
            origin_name: row.origin_name,
            origin_lat: row.origin_lat,
            origin_lng: row.origin_lng,
            origin_country: row.origin_country,
            dest_name: row.dest_name,
            dest_lat: row.dest_lat,
            dest_lng: row.dest_lng,
            dest_country: row.dest_country,
            start_date: row.start_date,
            end_date: row.end_date,
            raw_json: None,
            origin_address_query: None,
            dest_address_query: None,
            origin_tz: None,
            dest_tz: None,
            flight_detail: None,
            rail_detail: None,
            boat_detail: None,
            transport_detail: None,
            cached_carrier: row.carrier,
            cost_amount: row.cost_amount,
            cost_currency: row.cost_currency,
            loyalty_program: row.loyalty_program,
            miles_earned: row.miles_earned,
        })
    }
}

#[derive(Debug, thiserror::Error)]
#[error("unknown travel_type '{value}': expected air|rail|boat|transport")]
struct ParseTravelTypeError {
    value: String,
}

/// Parse a travel type string from the database into a [`TravelType`] enum.
pub(super) fn parse_travel_type(value: &str) -> Result<TravelType, sqlx::Error> {
    match value {
        "air" => Ok(TravelType::Air),
        "rail" => Ok(TravelType::Rail),
        "boat" | "cruise" => Ok(TravelType::Boat),
        "transport" => Ok(TravelType::Transport),
        other => Err(sqlx::Error::Decode(Box::new(ParseTravelTypeError {
            value: other.to_string(),
        }))),
    }
}

// ---------------------------------------------------------------------------
// Shared helper functions for hop command submodules
// ---------------------------------------------------------------------------

pub(super) fn scoped_trip_id(user_id: i64, trip_id: &str) -> String {
    format!("{user_id}:{trip_id}")
}

pub(super) async fn insert_flight_detail(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    hop_id: i64,
    detail: &FlightDetail,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r"INSERT OR REPLACE INTO flight_details (
           hop_id,
           airline,
           flight_number,
           aircraft_type,
           cabin_class,
           seat,
           pnr
       ) VALUES (?, ?, ?, ?, ?, ?, ?)",
        hop_id,
        detail.airline,
        detail.flight_number,
        detail.aircraft_type,
        detail.cabin_class,
        detail.seat,
        detail.pnr,
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}

pub(super) async fn upsert_full_flight_detail(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    hop_id: i64,
    detail: &get_by_id::FullFlightDetail,
) -> Result<(), sqlx::Error> {
    let canceled = i32::from(detail.canceled);
    sqlx::query!(
        r"INSERT OR REPLACE INTO flight_details (
           hop_id, airline, flight_number, dep_terminal, dep_gate,
           arr_terminal, arr_gate, canceled, diverted_to,
           gate_dep_scheduled, gate_dep_actual,
           takeoff_scheduled, takeoff_actual,
           landing_scheduled, landing_actual,
           gate_arr_scheduled, gate_arr_actual,
           aircraft_type, tail_number, pnr, seat,
           seat_type, cabin_class, flight_reason, notes
       ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        hop_id,
        detail.airline,
        detail.flight_number,
        detail.dep_terminal,
        detail.dep_gate,
        detail.arr_terminal,
        detail.arr_gate,
        canceled,
        detail.diverted_to,
        detail.gate_dep_scheduled,
        detail.gate_dep_actual,
        detail.takeoff_scheduled,
        detail.takeoff_actual,
        detail.landing_scheduled,
        detail.landing_actual,
        detail.gate_arr_scheduled,
        detail.gate_arr_actual,
        detail.aircraft_type,
        detail.tail_number,
        detail.pnr,
        detail.seat,
        detail.seat_type,
        detail.cabin_class,
        detail.flight_reason,
        detail.notes,
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}

pub(super) async fn insert_rail_detail(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    hop_id: i64,
    detail: &RailDetail,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r"INSERT OR REPLACE INTO rail_details (
           hop_id,
           carrier,
           train_number,
           service_class,
           coach_number,
           seats,
           confirmation_num,
           booking_site,
           notes
       ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        hop_id,
        detail.carrier,
        detail.train_number,
        detail.service_class,
        detail.coach_number,
        detail.seats,
        detail.confirmation_num,
        detail.booking_site,
        detail.notes,
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}

pub(super) async fn insert_boat_detail(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    hop_id: i64,
    detail: &BoatDetail,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r"INSERT OR REPLACE INTO boat_details (
           hop_id,
           ship_name,
           cabin_type,
           cabin_number,
           confirmation_num,
           booking_site,
           notes
       ) VALUES (?, ?, ?, ?, ?, ?, ?)",
        hop_id,
        detail.ship_name,
        detail.cabin_type,
        detail.cabin_number,
        detail.confirmation_num,
        detail.booking_site,
        detail.notes,
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}

pub(super) async fn insert_transport_detail(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    hop_id: i64,
    detail: &TransportDetail,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r"INSERT OR REPLACE INTO transport_details (
           hop_id,
           carrier_name,
           vehicle_description,
           confirmation_num,
           notes
       ) VALUES (?, ?, ?, ?, ?)",
        hop_id,
        detail.carrier_name,
        detail.vehicle_description,
        detail.confirmation_num,
        detail.notes,
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}

#[cfg(test)]
pub(super) fn sample_hop(
    travel_type: TravelType,
    origin: &str,
    dest: &str,
    start_date: &str,
    end_date: &str,
) -> Row {
    Row {
        id: 0,
        travel_type,
        origin_name: origin.to_string(),
        origin_lat: 1.0,
        origin_lng: 2.0,
        origin_country: None,
        dest_name: dest.to_string(),
        dest_lat: 3.0,
        dest_lng: 4.0,
        dest_country: None,
        start_date: start_date.to_string(),
        end_date: end_date.to_string(),
        raw_json: None,
        origin_address_query: None,
        dest_address_query: None,
        origin_tz: None,
        dest_tz: None,
        flight_detail: None,
        rail_detail: None,
        boat_detail: None,
        transport_detail: None,
        cached_carrier: None,
        cost_amount: None,
        cost_currency: None,
        loyalty_program: None,
        miles_earned: None,
    }
}

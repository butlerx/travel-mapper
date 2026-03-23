use crate::{geocode::airports, integrations::flighty::FlightRow};
use sqlx::SqlitePool;
use uuid::Uuid;

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
    #[must_use]
    pub const fn emoji(&self) -> &'static str {
        match self {
            Self::Air => "✈️",
            Self::Rail => "🚆",
            Self::Boat => "🚢",
            Self::Transport => "🚗",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct FlightDetail {
    pub airline: String,
    pub flight_number: String,
    pub aircraft_type: String,
    pub cabin_class: String,
    pub seat: String,
    pub pnr: String,
}

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

#[derive(Debug, Clone, Default)]
pub struct BoatDetail {
    pub ship_name: String,
    pub cabin_type: String,
    pub cabin_number: String,
    pub confirmation_num: String,
    pub booking_site: String,
    pub notes: String,
}

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
}

/// Internal row type for sqlx `query_as!` macro (`SQLite` stores `travel_type` as text).
struct HopRow {
    travel_type: String,
    origin_name: String,
    origin_lat: f64,
    origin_lng: f64,
    origin_country: Option<String>,
    dest_name: String,
    dest_lat: f64,
    dest_lng: f64,
    dest_country: Option<String>,
    start_date: String,
    end_date: String,
}

impl TryFrom<HopRow> for Row {
    type Error = sqlx::Error;

    fn try_from(row: HopRow) -> Result<Self, Self::Error> {
        Ok(Self {
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
        })
    }
}

#[derive(Debug, thiserror::Error)]
#[error("unknown travel_type '{value}': expected air|rail|boat|transport")]
struct ParseTravelTypeError {
    value: String,
}

fn parse_travel_type(value: &str) -> Result<TravelType, sqlx::Error> {
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

fn scoped_trip_id(user_id: i64, trip_id: &str) -> String {
    format!("{user_id}:{trip_id}")
}

/// Insert or replace travel hops for a trip in a single transaction.
pub struct Create<'a> {
    pub trip_id: &'a str,
    pub user_id: i64,
    pub hops: &'a [Row],
}

impl Create<'_> {
    /// # Errors
    ///
    /// Returns an error if inserting any hop fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<u64, sqlx::Error> {
        if self.hops.is_empty() {
            return Ok(0);
        }

        let mut tx = pool.begin().await?;
        let mut inserted = 0_u64;
        let db_trip_id = scoped_trip_id(self.user_id, self.trip_id);

        for hop in self.hops {
            let trip_id = db_trip_id.as_str();
            let travel_type = hop.travel_type.to_string();
            let origin_name = hop.origin_name.as_str();
            let dest_name = hop.dest_name.as_str();
            let start_date = hop.start_date.as_str();
            let end_date = hop.end_date.as_str();

            let result = sqlx::query!(
                r"INSERT OR REPLACE INTO hops (
                   trip_id,
                   user_id,
                   travel_type,
                   origin_name,
                   origin_lat,
                   origin_lng,
                   origin_country,
                   dest_name,
                   dest_lat,
                   dest_lng,
                   dest_country,
                   start_date,
                   end_date,
                   raw_json,
                   updated_at
               ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, datetime('now'))",
                trip_id,
                self.user_id,
                travel_type,
                origin_name,
                hop.origin_lat,
                hop.origin_lng,
                hop.origin_country,
                dest_name,
                hop.dest_lat,
                hop.dest_lng,
                hop.dest_country,
                start_date,
                end_date,
                hop.raw_json,
            )
            .execute(&mut *tx)
            .await?;

            let hop_id = result.last_insert_rowid();
            if let Some(detail) = &hop.flight_detail {
                insert_flight_detail(&mut tx, hop_id, detail).await?;
            }
            if let Some(detail) = &hop.rail_detail {
                insert_rail_detail(&mut tx, hop_id, detail).await?;
            }
            if let Some(detail) = &hop.boat_detail {
                insert_boat_detail(&mut tx, hop_id, detail).await?;
            }
            if let Some(detail) = &hop.transport_detail {
                insert_transport_detail(&mut tx, hop_id, detail).await?;
            }

            inserted += 1;
        }

        tx.commit().await?;
        Ok(inserted)
    }
}

async fn insert_flight_detail(
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

async fn insert_rail_detail(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    hop_id: i64,
    detail: &RailDetail,
) -> Result<(), sqlx::Error> {
    sqlx::query(
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
    )
    .bind(hop_id)
    .bind(&detail.carrier)
    .bind(&detail.train_number)
    .bind(&detail.service_class)
    .bind(&detail.coach_number)
    .bind(&detail.seats)
    .bind(&detail.confirmation_num)
    .bind(&detail.booking_site)
    .bind(&detail.notes)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

async fn insert_boat_detail(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    hop_id: i64,
    detail: &BoatDetail,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r"INSERT OR REPLACE INTO boat_details (
           hop_id,
           ship_name,
           cabin_type,
           cabin_number,
           confirmation_num,
           booking_site,
           notes
       ) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(hop_id)
    .bind(&detail.ship_name)
    .bind(&detail.cabin_type)
    .bind(&detail.cabin_number)
    .bind(&detail.confirmation_num)
    .bind(&detail.booking_site)
    .bind(&detail.notes)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

async fn insert_transport_detail(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    hop_id: i64,
    detail: &TransportDetail,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r"INSERT OR REPLACE INTO transport_details (
           hop_id,
           carrier_name,
           vehicle_description,
           confirmation_num,
           notes
       ) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(hop_id)
    .bind(&detail.carrier_name)
    .bind(&detail.vehicle_description)
    .bind(&detail.confirmation_num)
    .bind(&detail.notes)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

/// Insert or replace travel hops from a Flighty CSV import in a single transaction.
pub struct CreateFromFlighty<'a> {
    pub user_id: i64,
    pub rows: &'a [FlightRow],
}

impl CreateFromFlighty<'_> {
    /// # Errors
    ///
    /// Returns an error if inserting any hop fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<u64, sqlx::Error> {
        let mut tx = pool.begin().await?;
        let mut inserted = 0_u64;

        for row in self.rows {
            let fallback_id = format!("{}-{}-{}", row.from, row.to, row.date);
            let trip_id_suffix = if row.flighty_flight_id.is_empty() {
                &fallback_id
            } else {
                &row.flighty_flight_id
            };
            let db_trip_id = scoped_trip_id(self.user_id, &format!("flighty:{trip_id_suffix}"));

            let hop_id = self.insert_flighty_hop(&mut tx, &db_trip_id, row).await?;
            self.insert_flight_details(&mut tx, hop_id, row).await?;
            inserted += 1;
        }

        tx.commit().await?;
        Ok(inserted)
    }

    async fn insert_flighty_hop(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        db_trip_id: &str,
        row: &FlightRow,
    ) -> Result<i64, sqlx::Error> {
        let travel_type = "air";
        let end_date = if row.gate_arr_scheduled.len() >= 10 {
            &row.gate_arr_scheduled[..10]
        } else {
            &row.date
        };

        let origin = airports::lookup_enriched(&row.from);
        let dest = airports::lookup_enriched(&row.to);
        let origin_lat = origin.as_ref().map_or(0.0, |a| a.latitude);
        let origin_lng = origin.as_ref().map_or(0.0, |a| a.longitude);
        let origin_country = origin.as_ref().map(|a| a.country_code.clone());
        let dest_lat = dest.as_ref().map_or(0.0, |a| a.latitude);
        let dest_lng = dest.as_ref().map_or(0.0, |a| a.longitude);
        let dest_country = dest.as_ref().map(|a| a.country_code.clone());

        let result = sqlx::query!(
            r"INSERT OR REPLACE INTO hops (
               trip_id,
               user_id,
               travel_type,
               origin_name,
               origin_lat,
               origin_lng,
               origin_country,
               dest_name,
               dest_lat,
               dest_lng,
               dest_country,
               start_date,
               end_date,
               raw_json,
               updated_at
           ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, NULL, datetime('now'))",
            db_trip_id,
            self.user_id,
            travel_type,
            row.from,
            origin_lat,
            origin_lng,
            origin_country,
            row.to,
            dest_lat,
            dest_lng,
            dest_country,
            row.date,
            end_date,
        )
        .execute(&mut **tx)
        .await?;

        Ok(result.last_insert_rowid())
    }

    async fn insert_flight_details(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        hop_id: i64,
        row: &FlightRow,
    ) -> Result<(), sqlx::Error> {
        let canceled = i32::from(row.canceled);

        sqlx::query!(
            r"INSERT OR REPLACE INTO flight_details (
               hop_id,
               airline,
               flight_number,
               dep_terminal,
               dep_gate,
               arr_terminal,
               arr_gate,
               canceled,
               diverted_to,
               gate_dep_scheduled,
               gate_dep_actual,
               takeoff_scheduled,
               takeoff_actual,
               landing_scheduled,
               landing_actual,
               gate_arr_scheduled,
               gate_arr_actual,
               aircraft_type,
               tail_number,
               pnr,
               seat,
               seat_type,
               cabin_class,
               flight_reason,
               notes,
               airline_id,
               dep_airport_id,
               arr_airport_id,
               diverted_airport_id,
               aircraft_type_id
           ) VALUES (
               ?, ?, ?, ?, ?, ?, ?, ?, ?,
               ?, ?, ?, ?, ?, ?, ?, ?,
               ?, ?, ?, ?, ?, ?, ?, ?,
               ?, ?, ?, ?, ?
           )",
            hop_id,
            row.airline,
            row.flight_number,
            row.dep_terminal,
            row.dep_gate,
            row.arr_terminal,
            row.arr_gate,
            canceled,
            row.diverted_to,
            row.gate_dep_scheduled,
            row.gate_dep_actual,
            row.takeoff_scheduled,
            row.takeoff_actual,
            row.landing_scheduled,
            row.landing_actual,
            row.gate_arr_scheduled,
            row.gate_arr_actual,
            row.aircraft_type,
            row.tail_number,
            row.pnr,
            row.seat,
            row.seat_type,
            row.cabin_class,
            row.flight_reason,
            row.notes,
            row.airline_id,
            row.dep_airport_id,
            row.arr_airport_id,
            row.diverted_airport_id,
            row.aircraft_type_id,
        )
        .execute(&mut **tx)
        .await?;

        Ok(())
    }
}

/// Create a single manually-entered flight hop with flight details.
pub struct CreateManual {
    pub user_id: i64,
    pub origin: String,
    pub destination: String,
    pub date: String,
    pub flight_detail: FlightDetail,
}

impl CreateManual {
    /// # Errors
    ///
    /// Returns an error if inserting the hop or flight details fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<u64, sqlx::Error> {
        let trip_id = scoped_trip_id(self.user_id, &format!("manual:{}", Uuid::new_v4()));
        let travel_type = "air";
        let origin_airport = airports::lookup_enriched(&self.origin);
        let dest_airport = airports::lookup_enriched(&self.destination);
        let origin_lat = origin_airport.as_ref().map_or(0.0, |a| a.latitude);
        let origin_lng = origin_airport.as_ref().map_or(0.0, |a| a.longitude);
        let origin_country = origin_airport.as_ref().map(|a| a.country_code.clone());
        let dest_lat = dest_airport.as_ref().map_or(0.0, |a| a.latitude);
        let dest_lng = dest_airport.as_ref().map_or(0.0, |a| a.longitude);
        let dest_country = dest_airport.as_ref().map(|a| a.country_code.clone());
        let origin = self.origin.as_str();
        let destination = self.destination.as_str();
        let date = self.date.as_str();

        let mut tx = pool.begin().await?;

        let result = sqlx::query!(
            r"INSERT OR REPLACE INTO hops (
               trip_id,
               user_id,
               travel_type,
               origin_name,
               origin_lat,
               origin_lng,
               origin_country,
               dest_name,
               dest_lat,
               dest_lng,
               dest_country,
               start_date,
               end_date,
               raw_json,
               updated_at
           ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, NULL, datetime('now'))",
            trip_id,
            self.user_id,
            travel_type,
            origin,
            origin_lat,
            origin_lng,
            origin_country,
            destination,
            dest_lat,
            dest_lng,
            dest_country,
            date,
            date,
        )
        .execute(&mut *tx)
        .await?;

        let hop_id = result.last_insert_rowid();
        insert_flight_detail(&mut tx, hop_id, &self.flight_detail).await?;

        tx.commit().await?;
        Ok(1)
    }
}

/// A denormalised row for stats computation — joins hops with flight details.
#[derive(Debug, Clone)]
pub struct StatsRow {
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
    pub airline: Option<String>,
    pub aircraft_type: Option<String>,
    pub cabin_class: Option<String>,
    pub seat_type: Option<String>,
    pub flight_reason: Option<String>,
}

/// Internal row type for the stats query.
struct StatsHopRow {
    travel_type: String,
    origin_name: String,
    origin_lat: f64,
    origin_lng: f64,
    origin_country: Option<String>,
    dest_name: String,
    dest_lat: f64,
    dest_lng: f64,
    dest_country: Option<String>,
    start_date: String,
    end_date: String,
    airline: Option<String>,
    aircraft_type: Option<String>,
    cabin_class: Option<String>,
    seat_type: Option<String>,
    flight_reason: Option<String>,
}

impl TryFrom<StatsHopRow> for StatsRow {
    type Error = sqlx::Error;

    fn try_from(row: StatsHopRow) -> Result<Self, Self::Error> {
        /// Collapse `Some("")` (from LEFT JOIN on text columns) into `None`.
        fn non_empty(val: Option<String>) -> Option<String> {
            val.filter(|s| !s.is_empty())
        }

        Ok(Self {
            travel_type: parse_travel_type(&row.travel_type)?,
            origin_name: row.origin_name,
            origin_lat: row.origin_lat,
            origin_lng: row.origin_lng,
            origin_country: non_empty(row.origin_country),
            dest_name: row.dest_name,
            dest_lat: row.dest_lat,
            dest_lng: row.dest_lng,
            dest_country: non_empty(row.dest_country),
            start_date: row.start_date,
            end_date: row.end_date,
            airline: non_empty(row.airline),
            aircraft_type: non_empty(row.aircraft_type),
            cabin_class: non_empty(row.cabin_class),
            seat_type: non_empty(row.seat_type),
            flight_reason: non_empty(row.flight_reason),
        })
    }
}

/// Fetch all hops with flight detail fields for stats computation.
pub struct GetAllForStats {
    pub user_id: i64,
}

impl GetAllForStats {
    /// # Errors
    ///
    /// Returns an error if the query fails or a row cannot be mapped.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<Vec<StatsRow>, sqlx::Error> {
        let rows = sqlx::query_as!(
            StatsHopRow,
            r"SELECT
                   h.travel_type,
                   h.origin_name,
                   h.origin_lat,
                   h.origin_lng,
                   h.origin_country,
                   h.dest_name,
                   h.dest_lat,
                   h.dest_lng,
                   h.dest_country,
                   h.start_date,
                   h.end_date,
                   fd.airline,
                   fd.aircraft_type,
                   fd.cabin_class,
                   fd.seat_type,
                   fd.flight_reason
               FROM hops h
               LEFT JOIN flight_details fd ON fd.hop_id = h.id
               WHERE h.user_id = ?
               ORDER BY h.start_date ASC",
            self.user_id,
        )
        .fetch_all(pool)
        .await?;

        rows.into_iter().map(StatsRow::try_from).collect()
    }
}

/// Fetch all hops for a user, optionally filtered by travel type.
pub struct GetAll<'a> {
    pub user_id: i64,
    pub travel_type_filter: Option<&'a str>,
}

impl GetAll<'_> {
    /// # Errors
    ///
    /// Returns an error if the query fails or a row cannot be mapped.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<Vec<Row>, sqlx::Error> {
        let rows = match self.travel_type_filter {
            Some(filter) => {
                sqlx::query_as!(
                    HopRow,
                    r"SELECT
                           travel_type,
                           origin_name,
                           origin_lat,
                           origin_lng,
                           origin_country,
                           dest_name,
                           dest_lat,
                           dest_lng,
                           dest_country,
                           start_date,
                           end_date
                       FROM hops
                       WHERE user_id = ? AND travel_type = ?
                       ORDER BY start_date ASC",
                    self.user_id,
                    filter,
                )
                .fetch_all(pool)
                .await?
            }
            None => {
                sqlx::query_as!(
                    HopRow,
                    r"SELECT
                           travel_type,
                           origin_name,
                           origin_lat,
                           origin_lng,
                           origin_country,
                           dest_name,
                           dest_lat,
                           dest_lng,
                           dest_country,
                           start_date,
                           end_date
                       FROM hops
                       WHERE user_id = ?
                       ORDER BY start_date ASC",
                    self.user_id,
                )
                .fetch_all(pool)
                .await?
            }
        };

        rows.into_iter().map(Row::try_from).collect()
    }
}

/// Delete all hops belonging to a specific trip for a user.
pub struct DeleteForTrip<'a> {
    pub trip_id: &'a str,
    pub user_id: i64,
}

impl DeleteForTrip<'_> {
    /// # Errors
    ///
    /// Returns an error if the delete query fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<u64, sqlx::Error> {
        let db_trip_id = scoped_trip_id(self.user_id, self.trip_id);
        let result = sqlx::query!(
            "DELETE FROM hops WHERE trip_id = ? AND user_id = ?",
            db_trip_id,
            self.user_id,
        )
        .execute(pool)
        .await?;
        Ok(result.rows_affected())
    }
}

/// Atomically replace all hops for a trip: delete existing + insert new in one transaction.
pub struct ReplaceForTrip<'a> {
    pub trip_id: &'a str,
    pub user_id: i64,
    pub hops: &'a [Row],
}

impl ReplaceForTrip<'_> {
    /// # Errors
    ///
    /// Returns an error if the transaction fails. On failure, neither the
    /// delete nor the insert is committed — existing data is preserved.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<u64, sqlx::Error> {
        let db_trip_id = scoped_trip_id(self.user_id, self.trip_id);

        let mut tx = pool.begin().await?;

        sqlx::query!(
            "DELETE FROM hops WHERE trip_id = ? AND user_id = ?",
            db_trip_id,
            self.user_id,
        )
        .execute(&mut *tx)
        .await?;

        let mut inserted = 0_u64;
        for hop in self.hops {
            let trip_id = db_trip_id.as_str();
            let travel_type = hop.travel_type.to_string();
            let origin_name = hop.origin_name.as_str();
            let dest_name = hop.dest_name.as_str();
            let start_date = hop.start_date.as_str();
            let end_date = hop.end_date.as_str();

            let result = sqlx::query!(
                r"INSERT OR REPLACE INTO hops (
                   trip_id,
                   user_id,
                   travel_type,
                   origin_name,
                   origin_lat,
                   origin_lng,
                   origin_country,
                   dest_name,
                   dest_lat,
                   dest_lng,
                   dest_country,
                   start_date,
                   end_date,
                   raw_json,
                   updated_at
               ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, datetime('now'))",
                trip_id,
                self.user_id,
                travel_type,
                origin_name,
                hop.origin_lat,
                hop.origin_lng,
                hop.origin_country,
                dest_name,
                hop.dest_lat,
                hop.dest_lng,
                hop.dest_country,
                start_date,
                end_date,
                hop.raw_json,
            )
            .execute(&mut *tx)
            .await?;

            let hop_id = result.last_insert_rowid();
            if let Some(detail) = &hop.flight_detail {
                insert_flight_detail(&mut tx, hop_id, detail).await?;
            }
            if let Some(detail) = &hop.rail_detail {
                insert_rail_detail(&mut tx, hop_id, detail).await?;
            }
            if let Some(detail) = &hop.boat_detail {
                insert_boat_detail(&mut tx, hop_id, detail).await?;
            }
            if let Some(detail) = &hop.transport_detail {
                insert_transport_detail(&mut tx, hop_id, detail).await?;
            }

            inserted += 1;
        }

        tx.commit().await?;
        Ok(inserted)
    }
}

/// Remove hops for `TripIt` trips that no longer exist in the API response.
pub struct DeleteStaleTripItTrips<'a> {
    pub user_id: i64,
    pub active_trip_ids: &'a [String],
}

impl DeleteStaleTripItTrips<'_> {
    /// # Errors
    ///
    /// Returns an error if the delete query fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<u64, sqlx::Error> {
        let like_pattern = format!("{}:tripit:%", self.user_id);

        if self.active_trip_ids.is_empty() {
            let result = sqlx::query!(
                "DELETE FROM hops WHERE user_id = ? AND trip_id LIKE ?",
                self.user_id,
                like_pattern,
            )
            .execute(pool)
            .await?;
            return Ok(result.rows_affected());
        }

        let scoped: Vec<String> = self
            .active_trip_ids
            .iter()
            .map(|tid| scoped_trip_id(self.user_id, &format!("tripit:{tid}")))
            .collect();

        let mut qb = sqlx::QueryBuilder::new("DELETE FROM hops WHERE user_id = ");
        qb.push_bind(self.user_id);
        qb.push(" AND trip_id LIKE ");
        qb.push_bind(like_pattern);
        qb.push(" AND trip_id NOT IN ");
        qb.push("(");
        let mut sep = qb.separated(", ");
        for id in &scoped {
            sep.push_bind(id.clone());
        }
        sep.push_unseparated(")");

        let result = qb.build().execute(pool).await?;
        Ok(result.rows_affected())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::tests::{test_pool, test_user};

    fn sample_hop(
        travel_type: TravelType,
        origin: &str,
        dest: &str,
        start_date: &str,
        end_date: &str,
    ) -> Row {
        Row {
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
        }
    }

    #[tokio::test]
    async fn insert_and_get_all_hops_roundtrip() {
        let pool = test_pool().await;
        let user_id = test_user(&pool, "alice").await;
        let hops = vec![
            sample_hop(
                TravelType::Rail,
                "Paris",
                "London",
                "2024-01-01",
                "2024-01-01",
            ),
            sample_hop(TravelType::Air, "LHR", "JFK", "2024-02-01", "2024-02-01"),
        ];

        let inserted = Create {
            trip_id: "trip-1",
            user_id,
            hops: &hops,
        }
        .execute(&pool)
        .await
        .expect("insert failed");
        assert_eq!(inserted, 2);

        let fetched = GetAll {
            user_id,
            travel_type_filter: None,
        }
        .execute(&pool)
        .await
        .expect("fetch failed");
        assert_eq!(fetched.len(), 2);
        assert_eq!(fetched[0].start_date, "2024-01-01");
        assert_eq!(fetched[1].start_date, "2024-02-01");
    }

    #[tokio::test]
    async fn get_all_hops_filters_by_travel_type() {
        let pool = test_pool().await;
        let user_id = test_user(&pool, "alice").await;
        let hops = vec![
            sample_hop(
                TravelType::Rail,
                "Paris",
                "London",
                "2024-01-01",
                "2024-01-01",
            ),
            sample_hop(TravelType::Air, "LHR", "JFK", "2024-02-01", "2024-02-01"),
        ];

        Create {
            trip_id: "trip-1",
            user_id,
            hops: &hops,
        }
        .execute(&pool)
        .await
        .expect("insert failed");

        let filtered = GetAll {
            user_id,
            travel_type_filter: Some("rail"),
        }
        .execute(&pool)
        .await
        .expect("filter failed");
        assert_eq!(filtered.len(), 1);
        assert!(matches!(filtered[0].travel_type, TravelType::Rail));
    }

    #[tokio::test]
    async fn get_all_hops_isolated_per_user() {
        let pool = test_pool().await;
        let alice_id = test_user(&pool, "alice").await;
        let bob_id = test_user(&pool, "bob").await;

        Create {
            trip_id: "trip-1",
            user_id: alice_id,
            hops: &[sample_hop(
                TravelType::Rail,
                "Paris",
                "London",
                "2024-01-01",
                "2024-01-01",
            )],
        }
        .execute(&pool)
        .await
        .expect("insert alice failed");

        Create {
            trip_id: "trip-1",
            user_id: bob_id,
            hops: &[sample_hop(
                TravelType::Air,
                "LHR",
                "JFK",
                "2024-02-01",
                "2024-02-01",
            )],
        }
        .execute(&pool)
        .await
        .expect("insert bob failed");

        let alice_hops = GetAll {
            user_id: alice_id,
            travel_type_filter: None,
        }
        .execute(&pool)
        .await
        .expect("fetch alice failed");
        let bob_hops = GetAll {
            user_id: bob_id,
            travel_type_filter: None,
        }
        .execute(&pool)
        .await
        .expect("fetch bob failed");

        assert_eq!(alice_hops.len(), 1);
        assert_eq!(bob_hops.len(), 1);
        assert_eq!(alice_hops[0].origin_name, "Paris");
        assert_eq!(bob_hops[0].origin_name, "LHR");
    }

    #[tokio::test]
    async fn delete_hops_for_trip_removes_only_target_trip() {
        let pool = test_pool().await;
        let user_id = test_user(&pool, "alice").await;

        let trip_1_hops = vec![sample_hop(
            TravelType::Rail,
            "Paris",
            "London",
            "2024-01-01",
            "2024-01-01",
        )];
        let trip_2_hops = vec![sample_hop(
            TravelType::Air,
            "LHR",
            "JFK",
            "2024-02-01",
            "2024-02-01",
        )];

        Create {
            trip_id: "trip-1",
            user_id,
            hops: &trip_1_hops,
        }
        .execute(&pool)
        .await
        .expect("insert trip-1 failed");
        Create {
            trip_id: "trip-2",
            user_id,
            hops: &trip_2_hops,
        }
        .execute(&pool)
        .await
        .expect("insert trip-2 failed");

        let deleted = DeleteForTrip {
            trip_id: "trip-1",
            user_id,
        }
        .execute(&pool)
        .await
        .expect("delete failed");
        assert_eq!(deleted, 1);

        let remaining = GetAll {
            user_id,
            travel_type_filter: None,
        }
        .execute(&pool)
        .await
        .expect("fetch failed");
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].origin_name, "LHR");
    }

    #[tokio::test]
    async fn replace_for_trip_atomically_swaps_hops() {
        let pool = test_pool().await;
        let user_id = test_user(&pool, "alice").await;

        // Insert initial hops for two trips
        Create {
            trip_id: "tripit:100",
            user_id,
            hops: &[sample_hop(
                TravelType::Air,
                "LHR",
                "JFK",
                "2024-01-01",
                "2024-01-01",
            )],
        }
        .execute(&pool)
        .await
        .expect("insert trip 100 failed");

        Create {
            trip_id: "tripit:200",
            user_id,
            hops: &[sample_hop(
                TravelType::Rail,
                "Paris",
                "London",
                "2024-02-01",
                "2024-02-01",
            )],
        }
        .execute(&pool)
        .await
        .expect("insert trip 200 failed");

        let inserted = ReplaceForTrip {
            trip_id: "tripit:100",
            user_id,
            hops: &[
                sample_hop(TravelType::Air, "SFO", "NRT", "2024-03-01", "2024-03-01"),
                sample_hop(TravelType::Air, "NRT", "SFO", "2024-03-10", "2024-03-10"),
            ],
        }
        .execute(&pool)
        .await
        .expect("replace failed");
        assert_eq!(inserted, 2);

        // Trip 200 should be untouched
        let all = GetAll {
            user_id,
            travel_type_filter: None,
        }
        .execute(&pool)
        .await
        .expect("fetch failed");
        assert_eq!(all.len(), 3);

        // The old LHR->JFK hop is gone, replaced by SFO->NRT and NRT->SFO
        let origins: Vec<&str> = all.iter().map(|h| h.origin_name.as_str()).collect();
        assert!(!origins.contains(&"LHR"));
        assert!(origins.contains(&"SFO"));
        assert!(origins.contains(&"NRT"));
        assert!(origins.contains(&"Paris"));
    }

    #[tokio::test]
    async fn replace_for_trip_with_empty_hops_deletes_only() {
        let pool = test_pool().await;
        let user_id = test_user(&pool, "alice").await;

        Create {
            trip_id: "tripit:100",
            user_id,
            hops: &[sample_hop(
                TravelType::Air,
                "LHR",
                "JFK",
                "2024-01-01",
                "2024-01-01",
            )],
        }
        .execute(&pool)
        .await
        .expect("insert failed");

        let inserted = ReplaceForTrip {
            trip_id: "tripit:100",
            user_id,
            hops: &[],
        }
        .execute(&pool)
        .await
        .expect("replace with empty failed");
        assert_eq!(inserted, 0);

        let all = GetAll {
            user_id,
            travel_type_filter: None,
        }
        .execute(&pool)
        .await
        .expect("fetch failed");
        assert_eq!(all.len(), 0);
    }

    #[tokio::test]
    async fn delete_stale_tripit_trips_removes_inactive() {
        let pool = test_pool().await;
        let user_id = test_user(&pool, "alice").await;

        // Insert hops for three tripit trips and one non-tripit trip
        for (trip_id, origin) in [
            ("tripit:100", "LHR"),
            ("tripit:200", "CDG"),
            ("tripit:300", "SFO"),
            ("flighty:abc", "NRT"),
        ] {
            Create {
                trip_id,
                user_id,
                hops: &[sample_hop(
                    TravelType::Air,
                    origin,
                    "JFK",
                    "2024-01-01",
                    "2024-01-01",
                )],
            }
            .execute(&pool)
            .await
            .unwrap();
        }

        // Only trips 100 and 300 are still active
        let deleted = DeleteStaleTripItTrips {
            user_id,
            active_trip_ids: &["100".to_string(), "300".to_string()],
        }
        .execute(&pool)
        .await
        .expect("delete stale failed");
        assert_eq!(deleted, 1); // trip 200 removed

        let all = GetAll {
            user_id,
            travel_type_filter: None,
        }
        .execute(&pool)
        .await
        .expect("fetch failed");
        assert_eq!(all.len(), 3); // trips 100, 300, and flighty:abc remain

        let origins: Vec<&str> = all.iter().map(|h| h.origin_name.as_str()).collect();
        assert!(origins.contains(&"LHR"));
        assert!(!origins.contains(&"CDG"));
        assert!(origins.contains(&"SFO"));
        assert!(origins.contains(&"NRT"));
    }

    #[tokio::test]
    async fn delete_stale_tripit_trips_empty_active_removes_all_tripit() {
        let pool = test_pool().await;
        let user_id = test_user(&pool, "alice").await;

        Create {
            trip_id: "tripit:100",
            user_id,
            hops: &[sample_hop(
                TravelType::Air,
                "LHR",
                "JFK",
                "2024-01-01",
                "2024-01-01",
            )],
        }
        .execute(&pool)
        .await
        .unwrap();

        Create {
            trip_id: "flighty:abc",
            user_id,
            hops: &[sample_hop(
                TravelType::Air,
                "NRT",
                "SFO",
                "2024-02-01",
                "2024-02-01",
            )],
        }
        .execute(&pool)
        .await
        .unwrap();

        let deleted = DeleteStaleTripItTrips {
            user_id,
            active_trip_ids: &[],
        }
        .execute(&pool)
        .await
        .expect("delete stale failed");
        assert_eq!(deleted, 1); // tripit:100 removed

        let all = GetAll {
            user_id,
            travel_type_filter: None,
        }
        .execute(&pool)
        .await
        .expect("fetch failed");
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].origin_name, "NRT"); // flighty hop survives
    }

    #[tokio::test]
    async fn delete_stale_tripit_trips_isolated_per_user() {
        let pool = test_pool().await;
        let alice_id = test_user(&pool, "alice").await;
        let bob_id = test_user(&pool, "bob").await;

        // Both users have tripit:100
        for (uid, origin) in [(alice_id, "LHR"), (bob_id, "CDG")] {
            Create {
                trip_id: "tripit:100",
                user_id: uid,
                hops: &[sample_hop(
                    TravelType::Air,
                    origin,
                    "JFK",
                    "2024-01-01",
                    "2024-01-01",
                )],
            }
            .execute(&pool)
            .await
            .unwrap();
        }

        DeleteStaleTripItTrips {
            user_id: alice_id,
            active_trip_ids: &[],
        }
        .execute(&pool)
        .await
        .expect("delete stale failed");

        // Alice has nothing, Bob still has his hop
        let alice_hops = GetAll {
            user_id: alice_id,
            travel_type_filter: None,
        }
        .execute(&pool)
        .await
        .unwrap();
        let bob_hops = GetAll {
            user_id: bob_id,
            travel_type_filter: None,
        }
        .execute(&pool)
        .await
        .unwrap();

        assert_eq!(alice_hops.len(), 0);
        assert_eq!(bob_hops.len(), 1);
        assert_eq!(bob_hops[0].origin_name, "CDG");
    }

    #[tokio::test]
    async fn insert_hops_duplicate_unique_key_replaces_row() {
        let pool = test_pool().await;
        let user_id = test_user(&pool, "alice").await;

        let first = sample_hop(TravelType::Air, "LHR", "JFK", "2024-03-01", "2024-03-01");
        let mut replacement = first.clone();
        replacement.origin_lat = 99.9;

        Create {
            trip_id: "trip-1",
            user_id,
            hops: &[first],
        }
        .execute(&pool)
        .await
        .expect("first insert failed");
        Create {
            trip_id: "trip-1",
            user_id,
            hops: &[replacement],
        }
        .execute(&pool)
        .await
        .expect("replacement insert failed");

        let fetched = GetAll {
            user_id,
            travel_type_filter: Some("air"),
        }
        .execute(&pool)
        .await
        .expect("fetch failed");
        assert_eq!(fetched.len(), 1);
        assert!((fetched[0].origin_lat - 99.9_f64).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn get_all_for_stats_joins_flight_details() {
        let pool = test_pool().await;
        let user_id = test_user(&pool, "alice").await;

        let mut hop = sample_hop(TravelType::Air, "DUB", "LHR", "2024-06-01", "2024-06-01");
        hop.flight_detail = Some(FlightDetail {
            airline: "Aer Lingus".to_string(),
            flight_number: "EI154".to_string(),
            aircraft_type: "A320".to_string(),
            cabin_class: "Economy".to_string(),
            seat: "12A".to_string(),
            pnr: "ABC123".to_string(),
        });
        Create {
            trip_id: "trip-1",
            user_id,
            hops: &[hop],
        }
        .execute(&pool)
        .await
        .expect("insert failed");

        let stats = GetAllForStats { user_id }
            .execute(&pool)
            .await
            .expect("stats query failed");
        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0].airline.as_deref(), Some("Aer Lingus"));
        assert_eq!(stats[0].aircraft_type.as_deref(), Some("A320"));
        assert_eq!(stats[0].cabin_class.as_deref(), Some("Economy"));
    }

    #[tokio::test]
    async fn get_all_for_stats_returns_none_for_non_air_hops() {
        let pool = test_pool().await;
        let user_id = test_user(&pool, "alice").await;

        Create {
            trip_id: "trip-1",
            user_id,
            hops: &[sample_hop(
                TravelType::Rail,
                "Paris",
                "London",
                "2024-01-01",
                "2024-01-01",
            )],
        }
        .execute(&pool)
        .await
        .expect("insert failed");

        let stats = GetAllForStats { user_id }
            .execute(&pool)
            .await
            .expect("stats query failed");
        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0].airline, None);
        assert_eq!(stats[0].aircraft_type, None);
    }

    #[tokio::test]
    async fn create_manual_inserts_hop_and_flight_detail() {
        let pool = test_pool().await;
        let user_id = test_user(&pool, "alice").await;

        let created = CreateManual {
            user_id,
            origin: "DUB".to_string(),
            destination: "LHR".to_string(),
            date: "2025-06-15".to_string(),
            flight_detail: FlightDetail {
                airline: "Aer Lingus".to_string(),
                flight_number: "EI154".to_string(),
                aircraft_type: "Airbus A320".to_string(),
                cabin_class: "Economy".to_string(),
                seat: "12A".to_string(),
                pnr: "ABC123".to_string(),
            },
        }
        .execute(&pool)
        .await
        .expect("create manual failed");
        assert_eq!(created, 1);

        let hops = GetAll {
            user_id,
            travel_type_filter: Some("air"),
        }
        .execute(&pool)
        .await
        .expect("fetch failed");
        assert_eq!(hops.len(), 1);
        assert_eq!(hops[0].origin_name, "DUB");
        assert_eq!(hops[0].dest_name, "LHR");
        assert_eq!(hops[0].start_date, "2025-06-15");
        assert!(
            hops[0].origin_lat.abs() > 0.1,
            "origin_lat should be resolved from airport lookup"
        );
        assert!(
            hops[0].dest_lat.abs() > 0.1,
            "dest_lat should be resolved from airport lookup"
        );

        let detail = sqlx::query!(
            "SELECT airline, flight_number, aircraft_type, cabin_class, seat, pnr
             FROM flight_details fd
             JOIN hops h ON fd.hop_id = h.id
             WHERE h.user_id = ?",
            user_id,
        )
        .fetch_one(&pool)
        .await
        .expect("flight detail not found");
        assert_eq!(detail.airline, "Aer Lingus");
        assert_eq!(detail.flight_number, "EI154");
        assert_eq!(detail.aircraft_type, "Airbus A320");
        assert_eq!(detail.cabin_class, "Economy");
        assert_eq!(detail.seat, "12A");
        assert_eq!(detail.pnr, "ABC123");
    }
}

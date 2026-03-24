use super::{FlightDetail, insert_flight_detail, scoped_trip_id};
use crate::{
    geocode::airports,
    integrations::generic_csv::{GenericRow, ImportFormat},
};
use sqlx::SqlitePool;

/// Insert or replace travel hops from a generic CSV import in a single transaction.
pub struct CreateFromCsv<'a> {
    pub user_id: i64,
    pub rows: &'a [GenericRow],
}

impl CreateFromCsv<'_> {
    /// # Errors
    ///
    /// Returns an error if inserting any hop fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<u64, sqlx::Error> {
        let mut tx = pool.begin().await?;
        let mut inserted = 0_u64;

        for row in self.rows {
            let db_trip_id = build_trip_id(self.user_id, row);
            let hop_id = self.insert_hop(&mut tx, &db_trip_id, row).await?;

            if row.source_format == ImportFormat::Flighty {
                insert_flighty_flight_detail(&mut tx, hop_id, row).await?;
            } else {
                insert_flight_detail(&mut tx, hop_id, &flight_detail_from_row(row)).await?;
            }
            inserted += 1;
        }

        tx.commit().await?;
        Ok(inserted)
    }

    async fn insert_hop(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        db_trip_id: &str,
        row: &GenericRow,
    ) -> Result<i64, sqlx::Error> {
        let travel_type = "air";

        let end_date = if row.arr_time.len() >= 10 {
            &row.arr_time[..10]
        } else {
            &row.date
        };

        let origin = airports::lookup_enriched(&row.from_iata);
        let dest = airports::lookup_enriched(&row.to_iata);
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
            row.from_iata,
            origin_lat,
            origin_lng,
            origin_country,
            row.to_iata,
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
}

fn flight_detail_from_row(row: &GenericRow) -> FlightDetail {
    let flight_number =
        if row.flight_number.is_empty() && row.source_format == ImportFormat::AppInTheAir {
            String::new()
        } else {
            row.flight_number.clone()
        };

    FlightDetail {
        airline: row.airline.clone(),
        flight_number,
        aircraft_type: row.aircraft_type.clone(),
        cabin_class: row.cabin_class.clone(),
        seat: row.seat.clone(),
        pnr: row.pnr.clone(),
    }
}

fn build_trip_id(user_id: i64, row: &GenericRow) -> String {
    if row.source_format == ImportFormat::Flighty {
        let suffix = row
            .flighty_flight_id
            .as_deref()
            .filter(|id| !id.is_empty())
            .map_or_else(
                || format!("flighty:{}-{}-{}", row.from_iata, row.to_iata, row.date),
                |id| format!("flighty:{id}"),
            );
        scoped_trip_id(user_id, &suffix)
    } else {
        let suffix = format!(
            "{}:{}-{}-{}",
            row.source_format, row.from_iata, row.to_iata, row.date
        );
        scoped_trip_id(user_id, &suffix)
    }
}

async fn insert_flighty_flight_detail(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    hop_id: i64,
    row: &GenericRow,
) -> Result<(), sqlx::Error> {
    let canceled = row.canceled.map_or(0, i32::from);
    sqlx::query!(
        r"INSERT OR REPLACE INTO flight_details (
           hop_id, airline, flight_number, dep_terminal, dep_gate,
           arr_terminal, arr_gate, canceled, diverted_to,
           gate_dep_scheduled, gate_dep_actual,
           takeoff_scheduled, takeoff_actual,
           landing_scheduled, landing_actual,
           gate_arr_scheduled, gate_arr_actual,
           aircraft_type, tail_number, pnr, seat,
           seat_type, cabin_class, flight_reason, notes,
           airline_id, dep_airport_id, arr_airport_id,
           diverted_airport_id, aircraft_type_id
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
        row.note,
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

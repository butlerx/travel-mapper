use super::scoped_trip_id;
use crate::{geocode::airports, integrations::flighty::FlightRow};
use sqlx::SqlitePool;

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

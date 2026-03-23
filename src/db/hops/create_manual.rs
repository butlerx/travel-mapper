use super::{FlightDetail, insert_flight_detail, scoped_trip_id};
use crate::geocode::airports;
use sqlx::SqlitePool;
use uuid::Uuid;

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

#[cfg(test)]
mod tests {
    use crate::db::{
        hops::{FlightDetail, GetAll, TravelType},
        tests::{test_pool, test_user},
    };

    use super::CreateManual;

    #[tokio::test]
    async fn create_manual_inserts_hop_and_flight_detail() {
        let pool = test_pool().await;
        let user_id = test_user(&pool, "alice").await;

        CreateManual {
            user_id,
            origin: "DUB".to_string(),
            destination: "LHR".to_string(),
            date: "2024-07-01".to_string(),
            flight_detail: FlightDetail {
                airline: "EI".to_string(),
                flight_number: "EI154".to_string(),
                aircraft_type: "A320".to_string(),
                cabin_class: "business".to_string(),
                seat: String::new(),
                pnr: String::new(),
            },
        }
        .execute(&pool)
        .await
        .expect("create_manual failed");

        let hops = GetAll {
            user_id,
            travel_type_filter: Some("air"),
        }
        .execute(&pool)
        .await
        .expect("fetch failed");

        assert_eq!(hops.len(), 1);
        assert_eq!(hops[0].travel_type, TravelType::Air);
        assert_eq!(hops[0].origin_name, "DUB");
        assert_eq!(hops[0].dest_name, "LHR");

        let row = sqlx::query!(
            "SELECT airline, flight_number, aircraft_type, cabin_class FROM flight_details WHERE hop_id = ?",
            hops[0].id,
        )
        .fetch_one(&pool)
        .await
        .expect("flight_details query failed");

        assert_eq!(row.airline, "EI");
        assert_eq!(row.flight_number, "EI154");
        assert_eq!(row.aircraft_type, "A320");
        assert_eq!(row.cabin_class, "business");
    }
}

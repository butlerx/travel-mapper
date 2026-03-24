use super::{
    ManualDetail, insert_boat_detail, insert_flight_detail, insert_rail_detail,
    insert_transport_detail, scoped_trip_id,
};
use crate::geocode::airports;
use sqlx::SqlitePool;
use uuid::Uuid;

/// Create a single manually-entered hop with type-specific details.
pub struct CreateManual {
    pub user_id: i64,
    pub origin: String,
    pub destination: String,
    pub date: String,
    pub detail: ManualDetail,
}

impl CreateManual {
    /// # Errors
    ///
    /// Returns an error if inserting the hop or its details fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<u64, sqlx::Error> {
        let trip_id = scoped_trip_id(self.user_id, &format!("manual:{}", Uuid::new_v4()));

        let (travel_type, origin_lat, origin_lng, origin_country, dest_lat, dest_lng, dest_country) =
            match &self.detail {
                ManualDetail::Air(_) => {
                    let o = airports::lookup_enriched(&self.origin);
                    let d = airports::lookup_enriched(&self.destination);
                    (
                        "air",
                        o.as_ref().map_or(0.0, |a| a.latitude),
                        o.as_ref().map_or(0.0, |a| a.longitude),
                        o.as_ref().map(|a| a.country_code.clone()),
                        d.as_ref().map_or(0.0, |a| a.latitude),
                        d.as_ref().map_or(0.0, |a| a.longitude),
                        d.as_ref().map(|a| a.country_code.clone()),
                    )
                }
                ManualDetail::Rail(_) => ("rail", 0.0, 0.0, None, 0.0, 0.0, None),
                ManualDetail::Boat(_) => ("boat", 0.0, 0.0, None, 0.0, 0.0, None),
                ManualDetail::Transport(_) => ("transport", 0.0, 0.0, None, 0.0, 0.0, None),
            };

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
        match &self.detail {
            ManualDetail::Air(d) => insert_flight_detail(&mut tx, hop_id, d).await?,
            ManualDetail::Rail(d) => insert_rail_detail(&mut tx, hop_id, d).await?,
            ManualDetail::Boat(d) => insert_boat_detail(&mut tx, hop_id, d).await?,
            ManualDetail::Transport(d) => insert_transport_detail(&mut tx, hop_id, d).await?,
        }

        tx.commit().await?;
        Ok(1)
    }
}

#[cfg(test)]
mod tests {
    use crate::db::{
        hops::{
            BoatDetail, FlightDetail, GetAll, ManualDetail, RailDetail, TransportDetail, TravelType,
        },
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
            detail: ManualDetail::Air(FlightDetail {
                airline: "EI".to_string(),
                flight_number: "EI154".to_string(),
                aircraft_type: "A320".to_string(),
                cabin_class: "business".to_string(),
                seat: String::new(),
                pnr: String::new(),
            }),
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

    #[tokio::test]
    async fn create_manual_inserts_rail_hop() {
        let pool = test_pool().await;
        let user_id = test_user(&pool, "bob").await;

        CreateManual {
            user_id,
            origin: "London Euston".to_string(),
            destination: "Manchester Piccadilly".to_string(),
            date: "2024-08-15".to_string(),
            detail: ManualDetail::Rail(RailDetail {
                carrier: "Avanti".to_string(),
                train_number: "9S42".to_string(),
                service_class: "standard".to_string(),
                coach_number: "B".to_string(),
                seats: "42A".to_string(),
                confirmation_num: "XYZ123".to_string(),
                booking_site: String::new(),
                notes: String::new(),
            }),
        }
        .execute(&pool)
        .await
        .expect("create_manual rail failed");

        let hops = GetAll {
            user_id,
            travel_type_filter: Some("rail"),
        }
        .execute(&pool)
        .await
        .expect("fetch failed");

        assert_eq!(hops.len(), 1);
        assert_eq!(hops[0].travel_type, TravelType::Rail);
        assert_eq!(hops[0].origin_name, "London Euston");

        let row = sqlx::query!(
            "SELECT carrier, train_number FROM rail_details WHERE hop_id = ?",
            hops[0].id,
        )
        .fetch_one(&pool)
        .await
        .expect("rail_details query failed");

        assert_eq!(row.carrier, "Avanti");
        assert_eq!(row.train_number, "9S42");
    }

    #[tokio::test]
    async fn create_manual_inserts_boat_hop() {
        let pool = test_pool().await;
        let user_id = test_user(&pool, "carol").await;

        CreateManual {
            user_id,
            origin: "Dover".to_string(),
            destination: "Calais".to_string(),
            date: "2024-09-01".to_string(),
            detail: ManualDetail::Boat(BoatDetail {
                ship_name: "Spirit of Britain".to_string(),
                cabin_type: String::new(),
                cabin_number: String::new(),
                confirmation_num: "PO999".to_string(),
                booking_site: String::new(),
                notes: String::new(),
            }),
        }
        .execute(&pool)
        .await
        .expect("create_manual boat failed");

        let hops = GetAll {
            user_id,
            travel_type_filter: Some("boat"),
        }
        .execute(&pool)
        .await
        .expect("fetch failed");

        assert_eq!(hops.len(), 1);
        assert_eq!(hops[0].travel_type, TravelType::Boat);

        let row = sqlx::query!(
            "SELECT ship_name, confirmation_num FROM boat_details WHERE hop_id = ?",
            hops[0].id,
        )
        .fetch_one(&pool)
        .await
        .expect("boat_details query failed");

        assert_eq!(row.ship_name, "Spirit of Britain");
        assert_eq!(row.confirmation_num, "PO999");
    }

    #[tokio::test]
    async fn create_manual_inserts_transport_hop() {
        let pool = test_pool().await;
        let user_id = test_user(&pool, "dave").await;

        CreateManual {
            user_id,
            origin: "Dublin".to_string(),
            destination: "Cork".to_string(),
            date: "2024-10-05".to_string(),
            detail: ManualDetail::Transport(TransportDetail {
                carrier_name: "Bus Eireann".to_string(),
                vehicle_description: "Coach".to_string(),
                confirmation_num: String::new(),
                notes: "Window seat preferred".to_string(),
            }),
        }
        .execute(&pool)
        .await
        .expect("create_manual transport failed");

        let hops = GetAll {
            user_id,
            travel_type_filter: Some("transport"),
        }
        .execute(&pool)
        .await
        .expect("fetch failed");

        assert_eq!(hops.len(), 1);
        assert_eq!(hops[0].travel_type, TravelType::Transport);

        let row = sqlx::query!(
            "SELECT carrier_name, vehicle_description, notes FROM transport_details WHERE hop_id = ?",
            hops[0].id,
        )
        .fetch_one(&pool)
        .await
        .expect("transport_details query failed");

        assert_eq!(row.carrier_name, "Bus Eireann");
        assert_eq!(row.vehicle_description, "Coach");
        assert_eq!(row.notes, "Window seat preferred");
    }
}

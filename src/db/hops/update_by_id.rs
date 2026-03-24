use super::{
    BoatDetail, FullFlightDetail, RailDetail, TransportDetail, TravelType, insert_boat_detail,
    insert_rail_detail, insert_transport_detail, upsert_full_flight_detail,
};
use sqlx::SqlitePool;

/// Update an existing hop's core fields and type-specific details by ID.
///
/// The hop must belong to the given `user_id` — returns `Ok(false)` if not
/// found (either missing or owned by another user).
pub struct UpdateById {
    pub id: i64,
    pub user_id: i64,
    pub origin_name: String,
    pub dest_name: String,
    pub start_date: String,
    pub end_date: String,
    pub origin_lat: f64,
    pub origin_lng: f64,
    pub origin_country: Option<String>,
    pub dest_lat: f64,
    pub dest_lng: f64,
    pub dest_country: Option<String>,
    pub travel_type: TravelType,
    pub flight_detail: Option<FullFlightDetail>,
    pub rail_detail: Option<RailDetail>,
    pub boat_detail: Option<BoatDetail>,
    pub transport_detail: Option<TransportDetail>,
}

impl UpdateById {
    /// # Errors
    ///
    /// Returns an error if the update query or detail upserts fail.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<bool, sqlx::Error> {
        let mut tx = pool.begin().await?;
        let travel_type = self.travel_type.to_string();

        let result = sqlx::query!(
            r"UPDATE hops
               SET origin_name    = ?,
                   dest_name      = ?,
                   start_date     = ?,
                   end_date       = ?,
                   origin_lat     = ?,
                   origin_lng     = ?,
                   origin_country = ?,
                   dest_lat       = ?,
                   dest_lng       = ?,
                   dest_country   = ?,
                   travel_type    = ?,
                   updated_at     = datetime('now')
             WHERE id = ? AND user_id = ?",
            self.origin_name,
            self.dest_name,
            self.start_date,
            self.end_date,
            self.origin_lat,
            self.origin_lng,
            self.origin_country,
            self.dest_lat,
            self.dest_lng,
            self.dest_country,
            travel_type,
            self.id,
            self.user_id,
        )
        .execute(&mut *tx)
        .await?;

        if result.rows_affected() == 0 {
            return Ok(false);
        }

        // Upsert type-specific detail tables (INSERT OR REPLACE keyed on hop_id).
        match self.travel_type {
            TravelType::Air => {
                if let Some(detail) = &self.flight_detail {
                    upsert_full_flight_detail(&mut tx, self.id, detail).await?;
                }
            }
            TravelType::Rail => {
                if let Some(detail) = &self.rail_detail {
                    insert_rail_detail(&mut tx, self.id, detail).await?;
                }
            }
            TravelType::Boat => {
                if let Some(detail) = &self.boat_detail {
                    insert_boat_detail(&mut tx, self.id, detail).await?;
                }
            }
            TravelType::Transport => {
                if let Some(detail) = &self.transport_detail {
                    insert_transport_detail(&mut tx, self.id, detail).await?;
                }
            }
        }

        tx.commit().await?;
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use crate::db::{
        hops::{
            Create, FlightDetail, FullFlightDetail, GetAll, GetById, RailDetail, TravelType,
            sample_hop,
        },
        tests::{test_pool, test_user},
    };

    use super::UpdateById;

    #[tokio::test]
    async fn update_by_id_changes_hop_fields() {
        let pool = test_pool().await;
        let user_id = test_user(&pool, "alice").await;

        let mut hop = sample_hop(TravelType::Air, "LHR", "JFK", "2024-05-01", "2024-05-01");
        hop.flight_detail = Some(FlightDetail {
            airline: "BA".to_string(),
            flight_number: "BA178".to_string(),
            aircraft_type: "777".to_string(),
            cabin_class: "economy".to_string(),
            seat: String::new(),
            pnr: String::new(),
        });

        Create {
            trip_id: "trip-update",
            user_id,
            hops: &[hop],
        }
        .execute(&pool)
        .await
        .expect("insert failed");

        let all = GetAll {
            user_id,
            travel_type_filter: None,
        }
        .execute(&pool)
        .await
        .expect("fetch failed");
        let hop_id = all[0].id;

        let updated = UpdateById {
            id: hop_id,
            user_id,
            origin_name: "DUB".to_string(),
            dest_name: "CDG".to_string(),
            start_date: "2024-06-15".to_string(),
            end_date: "2024-06-15".to_string(),
            origin_lat: 53.4264,
            origin_lng: -6.2499,
            origin_country: Some("IE".to_string()),
            dest_lat: 49.0097,
            dest_lng: 2.5479,
            dest_country: Some("FR".to_string()),
            travel_type: TravelType::Air,
            flight_detail: Some(FullFlightDetail {
                airline: "EI".to_string(),
                flight_number: "EI520".to_string(),
                aircraft_type: "A320".to_string(),
                cabin_class: "business".to_string(),
                seat: "1A".to_string(),
                pnr: "XYZ789".to_string(),
                ..FullFlightDetail::default()
            }),
            rail_detail: None,
            boat_detail: None,
            transport_detail: None,
        }
        .execute(&pool)
        .await
        .expect("update failed");
        assert!(updated, "should return true when hop exists");

        let detail = GetById {
            id: hop_id,
            user_id,
        }
        .execute(&pool)
        .await
        .expect("get_by_id failed")
        .expect("should find row");

        assert_eq!(detail.origin_name, "DUB");
        assert_eq!(detail.dest_name, "CDG");
        assert_eq!(detail.start_date, "2024-06-15");

        let fd = detail.flight_detail.expect("should have flight detail");
        assert_eq!(fd.airline, "EI");
        assert_eq!(fd.flight_number, "EI520");
        assert_eq!(fd.seat, "1A");
        assert_eq!(fd.pnr, "XYZ789");
    }

    #[tokio::test]
    async fn update_by_id_returns_false_for_wrong_user() {
        let pool = test_pool().await;
        let alice = test_user(&pool, "alice").await;
        let bob = test_user(&pool, "bob").await;

        let hop = sample_hop(
            TravelType::Rail,
            "Paris",
            "London",
            "2024-07-01",
            "2024-07-01",
        );
        Create {
            trip_id: "trip-owner",
            user_id: alice,
            hops: &[hop],
        }
        .execute(&pool)
        .await
        .expect("insert failed");

        let all = GetAll {
            user_id: alice,
            travel_type_filter: None,
        }
        .execute(&pool)
        .await
        .expect("fetch failed");
        let hop_id = all[0].id;

        let updated = UpdateById {
            id: hop_id,
            user_id: bob,
            origin_name: "Berlin".to_string(),
            dest_name: "Munich".to_string(),
            start_date: "2024-07-01".to_string(),
            end_date: "2024-07-01".to_string(),
            origin_lat: 52.52,
            origin_lng: 13.405,
            origin_country: None,
            dest_lat: 48.1351,
            dest_lng: 11.582,
            dest_country: None,
            travel_type: TravelType::Rail,
            flight_detail: None,
            rail_detail: Some(RailDetail::default()),
            boat_detail: None,
            transport_detail: None,
        }
        .execute(&pool)
        .await
        .expect("update should not error");
        assert!(!updated, "bob should not be able to update alice's hop");
    }

    #[tokio::test]
    async fn update_by_id_returns_false_for_nonexistent_id() {
        let pool = test_pool().await;
        let user_id = test_user(&pool, "alice").await;

        let updated = UpdateById {
            id: 99999,
            user_id,
            origin_name: "DUB".to_string(),
            dest_name: "LHR".to_string(),
            start_date: "2024-01-01".to_string(),
            end_date: "2024-01-01".to_string(),
            origin_lat: 0.0,
            origin_lng: 0.0,
            origin_country: None,
            dest_lat: 0.0,
            dest_lng: 0.0,
            dest_country: None,
            travel_type: TravelType::Air,
            flight_detail: None,
            rail_detail: None,
            boat_detail: None,
            transport_detail: None,
        }
        .execute(&pool)
        .await
        .expect("update should not error");
        assert!(!updated);
    }
}

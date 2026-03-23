use super::{
    Row, insert_boat_detail, insert_flight_detail, insert_rail_detail, insert_transport_detail,
    scoped_trip_id,
};
use sqlx::SqlitePool;

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

#[cfg(test)]
mod tests {
    use crate::db::{
        hops::{Create, GetAll, TravelType, sample_hop},
        tests::{test_pool, test_user},
    };

    use super::ReplaceForTrip;

    #[tokio::test]
    async fn replace_for_trip_atomically_swaps_hops() {
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

        let all = GetAll {
            user_id,
            travel_type_filter: None,
        }
        .execute(&pool)
        .await
        .expect("fetch failed");
        assert_eq!(all.len(), 3);

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
}

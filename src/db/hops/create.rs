use super::{
    Row, insert_boat_detail, insert_flight_detail, insert_rail_detail, insert_transport_detail,
    scoped_trip_id,
};
use sqlx::SqlitePool;

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

#[cfg(test)]
mod tests {
    use crate::db::{
        hops::{GetAll, TravelType, sample_hop},
        tests::{test_pool, test_user},
    };

    use super::Create;

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
}

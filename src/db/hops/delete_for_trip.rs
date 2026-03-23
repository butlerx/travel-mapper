use super::scoped_trip_id;
use sqlx::SqlitePool;

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

#[cfg(test)]
mod tests {
    use crate::db::{
        hops::{Create, GetAll, TravelType, sample_hop},
        tests::{test_pool, test_user},
    };

    use super::DeleteForTrip;

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
}

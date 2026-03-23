use super::scoped_trip_id;
use sqlx::SqlitePool;

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

        let scoped_json =
            serde_json::to_string(&scoped).map_err(|e| sqlx::Error::Decode(Box::new(e)))?;

        let result = sqlx::query!(
            "DELETE FROM hops WHERE
                user_id = ? AND
                trip_id LIKE ? AND
                trip_id NOT IN (SELECT value FROM json_each(?))",
            self.user_id,
            like_pattern,
            scoped_json,
        )
        .execute(pool)
        .await?;
        Ok(result.rows_affected())
    }
}

#[cfg(test)]
mod tests {
    use super::DeleteStaleTripItTrips;
    use crate::db::{
        hops::{Create, GetAll, TravelType, sample_hop},
        tests::{test_pool, test_user},
    };

    #[tokio::test]
    async fn delete_stale_tripit_trips_removes_inactive() {
        let pool = test_pool().await;
        let user_id = test_user(&pool, "alice").await;

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

        let deleted = DeleteStaleTripItTrips {
            user_id,
            active_trip_ids: &["100".to_string(), "300".to_string()],
        }
        .execute(&pool)
        .await
        .expect("delete stale failed");
        assert_eq!(deleted, 1);

        let all = GetAll {
            user_id,
            travel_type_filter: None,
        }
        .execute(&pool)
        .await
        .expect("fetch failed");
        assert_eq!(all.len(), 3);

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
        assert_eq!(deleted, 1);

        let all = GetAll {
            user_id,
            travel_type_filter: None,
        }
        .execute(&pool)
        .await
        .expect("fetch failed");
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].origin_name, "NRT");
    }

    #[tokio::test]
    async fn delete_stale_tripit_trips_isolated_per_user() {
        let pool = test_pool().await;
        let alice_id = test_user(&pool, "alice").await;
        let bob_id = test_user(&pool, "bob").await;

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
}

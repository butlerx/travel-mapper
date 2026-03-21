use crate::models::{TravelHop, TravelType};
use sqlx::SqlitePool;

struct HopRow {
    travel_type: String,
    origin_name: String,
    origin_lat: Option<f64>,
    origin_lng: Option<f64>,
    dest_name: String,
    dest_lat: Option<f64>,
    dest_lng: Option<f64>,
    start_date: String,
    end_date: String,
}

impl TryFrom<HopRow> for TravelHop {
    type Error = sqlx::Error;

    fn try_from(row: HopRow) -> Result<Self, Self::Error> {
        Ok(Self {
            travel_type: parse_travel_type(&row.travel_type)?,
            origin_name: row.origin_name,
            origin_lat: row.origin_lat,
            origin_lng: row.origin_lng,
            dest_name: row.dest_name,
            dest_lat: row.dest_lat,
            dest_lng: row.dest_lng,
            start_date: row.start_date,
            end_date: row.end_date,
        })
    }
}

#[derive(Debug, thiserror::Error)]
#[error("unknown travel_type '{value}': expected air|rail|cruise|transport")]
struct ParseTravelTypeError {
    value: String,
}

fn parse_travel_type(value: &str) -> Result<TravelType, sqlx::Error> {
    match value {
        "air" => Ok(TravelType::Air),
        "rail" => Ok(TravelType::Rail),
        "cruise" => Ok(TravelType::Cruise),
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
    pub hops: &'a [TravelHop],
}

impl Create<'_> {
    /// # Errors
    ///
    /// Returns an error if inserting any hop fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<u64, sqlx::Error> {
        let mut tx = pool.begin().await?;
        let mut inserted = 0_u64;
        let db_trip_id = scoped_trip_id(self.user_id, self.trip_id);

        for hop in self.hops {
            let travel_type = hop.travel_type.to_string();
            let result = sqlx::query!(
                r"INSERT OR REPLACE INTO hops (
                       trip_id,
                       user_id,
                       travel_type,
                       origin_name,
                       origin_lat,
                       origin_lng,
                       dest_name,
                       dest_lat,
                       dest_lng,
                       start_date,
                       end_date,
                       raw_json,
                       updated_at
                   ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, NULL, datetime('now'))",
                db_trip_id,
                self.user_id,
                travel_type,
                hop.origin_name,
                hop.origin_lat,
                hop.origin_lng,
                hop.dest_name,
                hop.dest_lat,
                hop.dest_lng,
                hop.start_date,
                hop.end_date,
            )
            .execute(&mut *tx)
            .await?;
            inserted += result.rows_affected();
        }

        tx.commit().await?;
        Ok(inserted)
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
    pub async fn execute(&self, pool: &SqlitePool) -> Result<Vec<TravelHop>, sqlx::Error> {
        let rows = match self.travel_type_filter {
            Some(filter) => {
                sqlx::query_as!(
                    HopRow,
                    r"SELECT
                           travel_type,
                           origin_name,
                           origin_lat,
                           origin_lng,
                           dest_name,
                           dest_lat,
                           dest_lng,
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
                           dest_name,
                           dest_lat,
                           dest_lng,
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

        rows.into_iter().map(TravelHop::try_from).collect()
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
    /// Returns an error if the delete operation fails.
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
    use super::*;
    use crate::db::tests::{test_pool, test_user};
    use crate::models::TravelType;

    fn sample_hop(
        travel_type: crate::models::TravelType,
        origin: &str,
        dest: &str,
        start_date: &str,
        end_date: &str,
    ) -> crate::models::TravelHop {
        crate::models::TravelHop {
            travel_type,
            origin_name: origin.to_string(),
            origin_lat: Some(1.0),
            origin_lng: Some(2.0),
            dest_name: dest.to_string(),
            dest_lat: Some(3.0),
            dest_lng: Some(4.0),
            start_date: start_date.to_string(),
            end_date: end_date.to_string(),
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
    async fn insert_hops_duplicate_unique_key_replaces_row() {
        let pool = test_pool().await;
        let user_id = test_user(&pool, "alice").await;

        let first = sample_hop(TravelType::Air, "LHR", "JFK", "2024-03-01", "2024-03-01");
        let mut replacement = first.clone();
        replacement.origin_lat = Some(99.9);

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
        assert_eq!(fetched[0].origin_lat, Some(99.9));
    }
}

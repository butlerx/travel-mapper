use super::{HopRow, Row};
use sqlx::SqlitePool;

/// Fetch all hops for a user, optionally filtered by travel type.
pub struct GetAll<'a> {
    pub user_id: i64,
    pub travel_type_filter: Option<&'a str>,
}

impl GetAll<'_> {
    /// # Errors
    ///
    /// Returns an error if the query fails or a row cannot be mapped.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<Vec<Row>, sqlx::Error> {
        let rows = match self.travel_type_filter {
            Some(filter) => {
                sqlx::query_as!(
                    HopRow,
                    r#"SELECT
                           h.id as "id!: i64",
                           h.travel_type,
                           h.origin_name,
                           h.origin_lat,
                           h.origin_lng,
                           h.origin_country,
                           h.dest_name,
                           h.dest_lat,
                           h.dest_lng,
                           h.dest_country,
                           h.start_date,
                           h.end_date,
                           COALESCE(fd.airline, rd.carrier, bd.ship_name, td.carrier_name) as "carrier: String"
                       FROM hops h
                       LEFT JOIN flight_details fd ON fd.hop_id = h.id
                       LEFT JOIN rail_details rd ON rd.hop_id = h.id
                       LEFT JOIN boat_details bd ON bd.hop_id = h.id
                       LEFT JOIN transport_details td ON td.hop_id = h.id
                       WHERE h.user_id = ? AND h.travel_type = ?
                       ORDER BY h.start_date ASC"#,
                    self.user_id,
                    filter,
                )
                .fetch_all(pool)
                .await?
            }
            None => {
                sqlx::query_as!(
                    HopRow,
                    r#"SELECT
                           h.id as "id!: i64",
                           h.travel_type,
                           h.origin_name,
                           h.origin_lat,
                           h.origin_lng,
                           h.origin_country,
                           h.dest_name,
                           h.dest_lat,
                           h.dest_lng,
                           h.dest_country,
                           h.start_date,
                           h.end_date,
                           COALESCE(fd.airline, rd.carrier, bd.ship_name, td.carrier_name) as "carrier: String"
                       FROM hops h
                       LEFT JOIN flight_details fd ON fd.hop_id = h.id
                       LEFT JOIN rail_details rd ON rd.hop_id = h.id
                       LEFT JOIN boat_details bd ON bd.hop_id = h.id
                       LEFT JOIN transport_details td ON td.hop_id = h.id
                       WHERE h.user_id = ?
                       ORDER BY h.start_date ASC"#,
                    self.user_id,
                )
                .fetch_all(pool)
                .await?
            }
        };

        rows.into_iter().map(Row::try_from).collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::db::{
        hops::{Create, TravelType, sample_hop},
        tests::{test_pool, test_user},
    };

    use super::GetAll;

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
}

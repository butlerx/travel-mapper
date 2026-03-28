use super::{HopRow, Row};
use sqlx::SqlitePool;

/// Search hops with optional filters across all travel types.
pub struct Search<'a> {
    pub user_id: i64,
    pub travel_type: Option<&'a str>,
    pub origin: Option<&'a str>,
    pub dest: Option<&'a str>,
    pub date_from: Option<&'a str>,
    pub date_to: Option<&'a str>,
    pub airline: Option<&'a str>,
    pub flight_number: Option<&'a str>,
    pub cabin_class: Option<&'a str>,
    pub flight_reason: Option<&'a str>,
    pub q: Option<&'a str>,
}

impl Search<'_> {
    /// # Errors
    ///
    /// Returns an error if the query fails or a row cannot be mapped.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<Vec<Row>, sqlx::Error> {
        let q_pattern = self.q.map(|s| format!("%{s}%"));
        let origin_pattern = self.origin.map(|s| format!("%{s}%"));
        let dest_pattern = self.dest.map(|s| format!("%{s}%"));

        let rows = sqlx::query_as!(
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
                   COALESCE(fd.airline, rd.carrier, bd.ship_name, td.carrier_name) as "carrier: String",
                   h.cost_amount,
                   h.cost_currency,
                   h.loyalty_program,
                   h.miles_earned
               FROM hops h
               LEFT JOIN flight_details fd ON fd.hop_id = h.id
               LEFT JOIN rail_details rd ON rd.hop_id = h.id
               LEFT JOIN boat_details bd ON bd.hop_id = h.id
               LEFT JOIN transport_details td ON td.hop_id = h.id
               WHERE h.user_id = ?1
                 AND (?2 IS NULL OR h.travel_type = ?2)
                 AND (?3 IS NULL OR h.origin_name LIKE ?3)
                 AND (?4 IS NULL OR h.dest_name LIKE ?4)
                 AND (?5 IS NULL OR h.start_date >= ?5)
                 AND (?6 IS NULL OR h.start_date <= ?6)
                 AND (?7 IS NULL OR fd.airline LIKE ?7)
                 AND (?8 IS NULL OR fd.flight_number LIKE ?8)
                 AND (?9 IS NULL OR fd.cabin_class = ?9)
                 AND (?10 IS NULL OR fd.flight_reason = ?10)
                 AND (?11 IS NULL
                      OR h.origin_name LIKE ?11
                      OR h.dest_name LIKE ?11
                      OR fd.airline LIKE ?11
                      OR fd.flight_number LIKE ?11
                      OR fd.notes LIKE ?11
                      OR rd.notes LIKE ?11
                      OR bd.notes LIKE ?11
                      OR td.notes LIKE ?11)
               ORDER BY h.start_date ASC"#,
            self.user_id,
            self.travel_type,
            origin_pattern,
            dest_pattern,
            self.date_from,
            self.date_to,
            self.airline,
            self.flight_number,
            self.cabin_class,
            self.flight_reason,
            q_pattern,
        )
        .fetch_all(pool)
        .await?;

        rows.into_iter().map(Row::try_from).collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::db::{
        hops::{Create, FlightDetail, TravelType, sample_hop},
        tests::{test_pool, test_user},
    };

    use super::Search;

    fn default_search(user_id: i64) -> Search<'static> {
        Search {
            user_id,
            travel_type: None,
            origin: None,
            dest: None,
            date_from: None,
            date_to: None,
            airline: None,
            flight_number: None,
            cabin_class: None,
            flight_reason: None,
            q: None,
        }
    }

    #[tokio::test]
    async fn search_no_filters_returns_all() {
        let pool = test_pool().await;
        let user_id = test_user(&pool, "alice").await;

        Create {
            trip_id: "trip-1",
            user_id,
            hops: &[
                sample_hop(TravelType::Air, "LHR", "JFK", "2024-01-01", "2024-01-01"),
                sample_hop(
                    TravelType::Rail,
                    "Paris",
                    "London",
                    "2024-02-01",
                    "2024-02-01",
                ),
            ],
        }
        .execute(&pool)
        .await
        .expect("insert failed");

        let results = default_search(user_id)
            .execute(&pool)
            .await
            .expect("search failed");
        assert_eq!(results.len(), 2);
    }

    #[tokio::test]
    async fn search_filters_by_travel_type() {
        let pool = test_pool().await;
        let user_id = test_user(&pool, "alice").await;

        Create {
            trip_id: "trip-1",
            user_id,
            hops: &[
                sample_hop(TravelType::Air, "LHR", "JFK", "2024-01-01", "2024-01-01"),
                sample_hop(
                    TravelType::Rail,
                    "Paris",
                    "London",
                    "2024-02-01",
                    "2024-02-01",
                ),
            ],
        }
        .execute(&pool)
        .await
        .expect("insert failed");

        let mut search = default_search(user_id);
        search.travel_type = Some("air");
        let results = search.execute(&pool).await.expect("search failed");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].origin_name, "LHR");
    }

    #[tokio::test]
    async fn search_filters_by_origin() {
        let pool = test_pool().await;
        let user_id = test_user(&pool, "alice").await;

        Create {
            trip_id: "trip-1",
            user_id,
            hops: &[
                sample_hop(TravelType::Air, "LHR", "JFK", "2024-01-01", "2024-01-01"),
                sample_hop(TravelType::Air, "SFO", "NRT", "2024-02-01", "2024-02-01"),
            ],
        }
        .execute(&pool)
        .await
        .expect("insert failed");

        let mut search = default_search(user_id);
        search.origin = Some("LHR");
        let results = search.execute(&pool).await.expect("search failed");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].dest_name, "JFK");
    }

    #[tokio::test]
    async fn search_filters_by_date_range() {
        let pool = test_pool().await;
        let user_id = test_user(&pool, "alice").await;

        Create {
            trip_id: "trip-1",
            user_id,
            hops: &[
                sample_hop(TravelType::Air, "LHR", "JFK", "2024-01-15", "2024-01-15"),
                sample_hop(TravelType::Air, "SFO", "NRT", "2024-06-15", "2024-06-15"),
            ],
        }
        .execute(&pool)
        .await
        .expect("insert failed");

        let mut search = default_search(user_id);
        search.date_from = Some("2024-03-01");
        search.date_to = Some("2024-12-31");
        let results = search.execute(&pool).await.expect("search failed");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].origin_name, "SFO");
    }

    #[tokio::test]
    async fn search_filters_by_airline() {
        let pool = test_pool().await;
        let user_id = test_user(&pool, "alice").await;

        let mut ba_hop = sample_hop(TravelType::Air, "LHR", "JFK", "2024-01-01", "2024-01-01");
        ba_hop.flight_detail = Some(FlightDetail {
            airline: "British Airways".to_string(),
            flight_number: "BA178".to_string(),
            ..FlightDetail::default()
        });

        let mut ua_hop = sample_hop(TravelType::Air, "SFO", "NRT", "2024-02-01", "2024-02-01");
        ua_hop.flight_detail = Some(FlightDetail {
            airline: "United".to_string(),
            flight_number: "UA837".to_string(),
            ..FlightDetail::default()
        });

        Create {
            trip_id: "trip-1",
            user_id,
            hops: &[ba_hop, ua_hop],
        }
        .execute(&pool)
        .await
        .expect("insert failed");

        let mut search = default_search(user_id);
        search.airline = Some("British Airways");
        let results = search.execute(&pool).await.expect("search failed");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].origin_name, "LHR");
    }

    #[tokio::test]
    async fn search_free_text_matches_origin_and_dest() {
        let pool = test_pool().await;
        let user_id = test_user(&pool, "alice").await;

        Create {
            trip_id: "trip-1",
            user_id,
            hops: &[
                sample_hop(TravelType::Air, "LHR", "JFK", "2024-01-01", "2024-01-01"),
                sample_hop(TravelType::Air, "SFO", "NRT", "2024-02-01", "2024-02-01"),
            ],
        }
        .execute(&pool)
        .await
        .expect("insert failed");

        let mut search = default_search(user_id);
        search.q = Some("JFK");
        let results = search.execute(&pool).await.expect("search failed");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].origin_name, "LHR");
    }

    #[tokio::test]
    async fn search_combines_multiple_filters() {
        let pool = test_pool().await;
        let user_id = test_user(&pool, "alice").await;

        let mut hop1 = sample_hop(TravelType::Air, "LHR", "JFK", "2024-01-01", "2024-01-01");
        hop1.flight_detail = Some(FlightDetail {
            airline: "BA".to_string(),
            cabin_class: "business".to_string(),
            ..FlightDetail::default()
        });

        let mut hop2 = sample_hop(TravelType::Air, "LHR", "CDG", "2024-06-01", "2024-06-01");
        hop2.flight_detail = Some(FlightDetail {
            airline: "BA".to_string(),
            cabin_class: "economy".to_string(),
            ..FlightDetail::default()
        });

        Create {
            trip_id: "trip-1",
            user_id,
            hops: &[hop1, hop2],
        }
        .execute(&pool)
        .await
        .expect("insert failed");

        let mut search = default_search(user_id);
        search.airline = Some("BA");
        search.cabin_class = Some("business");
        let results = search.execute(&pool).await.expect("search failed");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].dest_name, "JFK");
    }
}

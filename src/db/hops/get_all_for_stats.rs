use super::{TravelType, parse_travel_type};
use sqlx::SqlitePool;

/// A denormalised row for stats computation — joins hops with flight details.
#[derive(Debug, Clone)]
pub struct StatsRow {
    pub travel_type: TravelType,
    pub origin_name: String,
    pub origin_lat: f64,
    pub origin_lng: f64,
    pub origin_country: Option<String>,
    pub dest_name: String,
    pub dest_lat: f64,
    pub dest_lng: f64,
    pub dest_country: Option<String>,
    pub start_date: String,
    pub end_date: String,
    pub airline: Option<String>,
    pub aircraft_type: Option<String>,
    pub cabin_class: Option<String>,
    pub seat_type: Option<String>,
    pub flight_reason: Option<String>,
}

/// Internal row type for the stats query.
struct StatsHopRow {
    travel_type: String,
    origin_name: String,
    origin_lat: f64,
    origin_lng: f64,
    origin_country: Option<String>,
    dest_name: String,
    dest_lat: f64,
    dest_lng: f64,
    dest_country: Option<String>,
    start_date: String,
    end_date: String,
    airline: Option<String>,
    aircraft_type: Option<String>,
    cabin_class: Option<String>,
    seat_type: Option<String>,
    flight_reason: Option<String>,
}

impl TryFrom<StatsHopRow> for StatsRow {
    type Error = sqlx::Error;

    fn try_from(row: StatsHopRow) -> Result<Self, Self::Error> {
        /// Collapse `Some("")` (from LEFT JOIN on text columns) into `None`.
        fn non_empty(val: Option<String>) -> Option<String> {
            val.filter(|s| !s.is_empty())
        }

        Ok(Self {
            travel_type: parse_travel_type(&row.travel_type)?,
            origin_name: row.origin_name,
            origin_lat: row.origin_lat,
            origin_lng: row.origin_lng,
            origin_country: non_empty(row.origin_country),
            dest_name: row.dest_name,
            dest_lat: row.dest_lat,
            dest_lng: row.dest_lng,
            dest_country: non_empty(row.dest_country),
            start_date: row.start_date,
            end_date: row.end_date,
            airline: non_empty(row.airline),
            aircraft_type: non_empty(row.aircraft_type),
            cabin_class: non_empty(row.cabin_class),
            seat_type: non_empty(row.seat_type),
            flight_reason: non_empty(row.flight_reason),
        })
    }
}

/// Fetch all hops with flight detail fields for stats computation.
pub struct GetAllForStats {
    pub user_id: i64,
}

impl GetAllForStats {
    /// # Errors
    ///
    /// Returns an error if the query fails or a row cannot be mapped.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<Vec<StatsRow>, sqlx::Error> {
        let rows = sqlx::query_as!(
            StatsHopRow,
            r"SELECT
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
                   fd.airline,
                   fd.aircraft_type,
                   fd.cabin_class,
                   fd.seat_type,
                   fd.flight_reason
               FROM hops h
               LEFT JOIN flight_details fd ON fd.hop_id = h.id
               WHERE h.user_id = ?
               ORDER BY h.start_date ASC",
            self.user_id,
        )
        .fetch_all(pool)
        .await?;

        rows.into_iter().map(StatsRow::try_from).collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::db::{
        hops::{Create, FlightDetail, TravelType, sample_hop},
        tests::{test_pool, test_user},
    };

    use super::GetAllForStats;

    #[tokio::test]
    async fn get_all_for_stats_joins_flight_details() {
        let pool = test_pool().await;
        let user_id = test_user(&pool, "alice").await;

        let mut hop = sample_hop(TravelType::Air, "LHR", "JFK", "2024-06-01", "2024-06-01");
        hop.flight_detail = Some(FlightDetail {
            airline: "BA".to_string(),
            flight_number: "BA178".to_string(),
            aircraft_type: "777".to_string(),
            cabin_class: "economy".to_string(),
            seat: String::new(),
            pnr: String::new(),
        });

        Create {
            trip_id: "trip-stats",
            user_id,
            hops: &[hop],
        }
        .execute(&pool)
        .await
        .expect("insert failed");

        let stats = GetAllForStats { user_id }
            .execute(&pool)
            .await
            .expect("stats query failed");

        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0].airline.as_deref(), Some("BA"));
        assert_eq!(stats[0].aircraft_type.as_deref(), Some("777"));
        assert_eq!(stats[0].cabin_class.as_deref(), Some("economy"));
    }

    #[tokio::test]
    async fn get_all_for_stats_returns_none_for_non_air_hops() {
        let pool = test_pool().await;
        let user_id = test_user(&pool, "alice").await;

        let hop = sample_hop(
            TravelType::Rail,
            "Paris",
            "London",
            "2024-06-02",
            "2024-06-02",
        );
        Create {
            trip_id: "trip-rail-stats",
            user_id,
            hops: &[hop],
        }
        .execute(&pool)
        .await
        .expect("insert failed");

        let stats = GetAllForStats { user_id }
            .execute(&pool)
            .await
            .expect("stats query failed");

        assert_eq!(stats.len(), 1);
        assert!(stats[0].airline.is_none());
        assert!(stats[0].aircraft_type.is_none());
        assert!(stats[0].cabin_class.is_none());
    }
}

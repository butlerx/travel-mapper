use super::{TravelType, parse_travel_type};
use sqlx::SqlitePool;

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
    pub rail_carrier: Option<String>,
    pub train_number: Option<String>,
    pub service_class: Option<String>,
    pub ship_name: Option<String>,
    pub boat_cabin_type: Option<String>,
    pub transport_carrier: Option<String>,
    pub vehicle_description: Option<String>,
    pub cost_amount: Option<f64>,
    pub cost_currency: Option<String>,
    pub loyalty_program: Option<String>,
    pub miles_earned: Option<f64>,
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
    rail_carrier: Option<String>,
    train_number: Option<String>,
    service_class: Option<String>,
    ship_name: Option<String>,
    boat_cabin_type: Option<String>,
    transport_carrier: Option<String>,
    vehicle_description: Option<String>,
    cost_amount: Option<f64>,
    cost_currency: Option<String>,
    loyalty_program: Option<String>,
    miles_earned: Option<f64>,
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
            rail_carrier: non_empty(row.rail_carrier),
            train_number: non_empty(row.train_number),
            service_class: non_empty(row.service_class),
            ship_name: non_empty(row.ship_name),
            boat_cabin_type: non_empty(row.boat_cabin_type),
            transport_carrier: non_empty(row.transport_carrier),
            vehicle_description: non_empty(row.vehicle_description),
            cost_amount: row.cost_amount,
            cost_currency: non_empty(row.cost_currency),
            loyalty_program: non_empty(row.loyalty_program),
            miles_earned: row.miles_earned,
        })
    }
}

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
                   fd.flight_reason,
                   rd.carrier AS rail_carrier,
                   rd.train_number,
                   rd.service_class,
                   bd.ship_name,
                   bd.cabin_type AS boat_cabin_type,
                   td.carrier_name AS transport_carrier,
                   td.vehicle_description,
                   h.cost_amount,
                   h.cost_currency,
                   h.loyalty_program,
                   h.miles_earned
                FROM hops h
               LEFT JOIN flight_details fd ON fd.hop_id = h.id
               LEFT JOIN rail_details rd ON rd.hop_id = h.id
               LEFT JOIN boat_details bd ON bd.hop_id = h.id
               LEFT JOIN transport_details td ON td.hop_id = h.id
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
        hops::{Create, FlightDetail, RailDetail, TravelType, sample_hop},
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
        assert!(stats[0].rail_carrier.is_none());
        assert!(stats[0].ship_name.is_none());
        assert!(stats[0].transport_carrier.is_none());
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
        assert!(stats[0].rail_carrier.is_none());
        assert!(stats[0].ship_name.is_none());
        assert!(stats[0].transport_carrier.is_none());
    }

    #[tokio::test]
    async fn get_all_for_stats_joins_rail_details() {
        let pool = test_pool().await;
        let user_id = test_user(&pool, "alice").await;

        let mut hop = sample_hop(
            TravelType::Rail,
            "Paris",
            "London",
            "2024-06-02",
            "2024-06-02",
        );
        hop.rail_detail = Some(RailDetail {
            carrier: "Eurostar".to_string(),
            train_number: "ES9026".to_string(),
            service_class: "Standard Premier".to_string(),
            ..Default::default()
        });

        Create {
            trip_id: "trip-rail-detail-stats",
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
        assert_eq!(stats[0].rail_carrier.as_deref(), Some("Eurostar"));
        assert_eq!(stats[0].train_number.as_deref(), Some("ES9026"));
        assert_eq!(stats[0].service_class.as_deref(), Some("Standard Premier"));
    }
}

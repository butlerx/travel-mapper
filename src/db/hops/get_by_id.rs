use super::{BoatDetail, RailDetail, TransportDetail, TravelType, parse_travel_type};
use sqlx::SqlitePool;

/// All columns from the `flight_details` table for the hop detail page.
#[derive(Debug, Clone, Default)]
pub struct FullFlightDetail {
    pub airline: String,
    pub flight_number: String,
    pub dep_terminal: String,
    pub dep_gate: String,
    pub arr_terminal: String,
    pub arr_gate: String,
    pub canceled: bool,
    pub diverted_to: String,
    pub gate_dep_scheduled: String,
    pub gate_dep_actual: String,
    pub takeoff_scheduled: String,
    pub takeoff_actual: String,
    pub landing_scheduled: String,
    pub landing_actual: String,
    pub gate_arr_scheduled: String,
    pub gate_arr_actual: String,
    pub aircraft_type: String,
    pub tail_number: String,
    pub pnr: String,
    pub seat: String,
    pub seat_type: String,
    pub cabin_class: String,
    pub flight_reason: String,
    pub notes: String,
}

/// Full hop detail including type-specific detail tables — used by the hop detail page.
#[derive(Debug, Clone)]
pub struct DetailRow {
    pub id: i64,
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
    pub flight_detail: Option<FullFlightDetail>,
    pub rail_detail: Option<RailDetail>,
    pub boat_detail: Option<BoatDetail>,
    pub transport_detail: Option<TransportDetail>,
    pub cost_amount: Option<f64>,
    pub cost_currency: Option<String>,
}

/// Internal row type for the `GetById` query — all detail columns are nullable from LEFT JOINs.
struct DetailHopRow {
    id: i64,
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
    // flight_details
    airline: Option<String>,
    flight_number: Option<String>,
    dep_terminal: Option<String>,
    dep_gate: Option<String>,
    arr_terminal: Option<String>,
    arr_gate: Option<String>,
    canceled: Option<i64>,
    diverted_to: Option<String>,
    gate_dep_scheduled: Option<String>,
    gate_dep_actual: Option<String>,
    takeoff_scheduled: Option<String>,
    takeoff_actual: Option<String>,
    landing_scheduled: Option<String>,
    landing_actual: Option<String>,
    gate_arr_scheduled: Option<String>,
    gate_arr_actual: Option<String>,
    aircraft_type: Option<String>,
    tail_number: Option<String>,
    pnr: Option<String>,
    seat: Option<String>,
    seat_type: Option<String>,
    cabin_class: Option<String>,
    flight_reason: Option<String>,
    flight_notes: Option<String>,
    // rail_details
    rail_carrier: Option<String>,
    train_number: Option<String>,
    service_class: Option<String>,
    coach_number: Option<String>,
    rail_seats: Option<String>,
    rail_confirmation: Option<String>,
    rail_booking_site: Option<String>,
    rail_notes: Option<String>,
    // boat_details
    ship_name: Option<String>,
    cabin_type: Option<String>,
    cabin_number: Option<String>,
    boat_confirmation: Option<String>,
    boat_booking_site: Option<String>,
    boat_notes: Option<String>,
    // transport_details
    transport_carrier: Option<String>,
    vehicle_description: Option<String>,
    transport_confirmation: Option<String>,
    transport_notes: Option<String>,
    cost_amount: Option<f64>,
    cost_currency: Option<String>,
}

impl TryFrom<DetailHopRow> for DetailRow {
    type Error = sqlx::Error;

    fn try_from(row: DetailHopRow) -> Result<Self, Self::Error> {
        /// Unwrap `Option<String>` to `String`, collapsing `None` and empty to `String::new()`.
        fn s(val: Option<String>) -> String {
            val.filter(|v| !v.is_empty()).unwrap_or_default()
        }

        /// Collapse `Some("")` (from LEFT JOIN on text columns) into `None`.
        fn non_empty(val: Option<String>) -> Option<String> {
            val.filter(|v| !v.is_empty())
        }

        let flight_detail = non_empty(row.airline.clone()).map(|_| FullFlightDetail {
            airline: s(row.airline),
            flight_number: s(row.flight_number),
            dep_terminal: s(row.dep_terminal),
            dep_gate: s(row.dep_gate),
            arr_terminal: s(row.arr_terminal),
            arr_gate: s(row.arr_gate),
            canceled: row.canceled.unwrap_or(0) != 0,
            diverted_to: s(row.diverted_to),
            gate_dep_scheduled: s(row.gate_dep_scheduled),
            gate_dep_actual: s(row.gate_dep_actual),
            takeoff_scheduled: s(row.takeoff_scheduled),
            takeoff_actual: s(row.takeoff_actual),
            landing_scheduled: s(row.landing_scheduled),
            landing_actual: s(row.landing_actual),
            gate_arr_scheduled: s(row.gate_arr_scheduled),
            gate_arr_actual: s(row.gate_arr_actual),
            aircraft_type: s(row.aircraft_type),
            tail_number: s(row.tail_number),
            pnr: s(row.pnr),
            seat: s(row.seat),
            seat_type: s(row.seat_type),
            cabin_class: s(row.cabin_class),
            flight_reason: s(row.flight_reason),
            notes: s(row.flight_notes),
        });

        let rail_detail = non_empty(row.rail_carrier.clone()).map(|_| RailDetail {
            carrier: s(row.rail_carrier),
            train_number: s(row.train_number),
            service_class: s(row.service_class),
            coach_number: s(row.coach_number),
            seats: s(row.rail_seats),
            confirmation_num: s(row.rail_confirmation),
            booking_site: s(row.rail_booking_site),
            notes: s(row.rail_notes),
        });

        let boat_detail = non_empty(row.ship_name.clone()).map(|_| BoatDetail {
            ship_name: s(row.ship_name),
            cabin_type: s(row.cabin_type),
            cabin_number: s(row.cabin_number),
            confirmation_num: s(row.boat_confirmation),
            booking_site: s(row.boat_booking_site),
            notes: s(row.boat_notes),
        });

        let transport_detail = non_empty(row.transport_carrier.clone()).map(|_| TransportDetail {
            carrier_name: s(row.transport_carrier),
            vehicle_description: s(row.vehicle_description),
            confirmation_num: s(row.transport_confirmation),
            notes: s(row.transport_notes),
        });

        Ok(Self {
            id: row.id,
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
            flight_detail,
            rail_detail,
            boat_detail,
            transport_detail,
            cost_amount: row.cost_amount,
            cost_currency: non_empty(row.cost_currency),
        })
    }
}

/// Fetch a single hop by ID with all detail table columns, scoped to a user.
pub struct GetById {
    pub id: i64,
    pub user_id: i64,
}

impl GetById {
    /// # Errors
    ///
    /// Returns an error if the query fails or the row cannot be mapped.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<Option<DetailRow>, sqlx::Error> {
        let row = sqlx::query_as!(
            DetailHopRow,
            r#"SELECT
                   h.id AS "id!: i64",
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
                   fd.flight_number,
                   fd.dep_terminal,
                   fd.dep_gate,
                   fd.arr_terminal,
                   fd.arr_gate,
                   fd.canceled,
                   fd.diverted_to,
                   fd.gate_dep_scheduled,
                   fd.gate_dep_actual,
                   fd.takeoff_scheduled,
                   fd.takeoff_actual,
                   fd.landing_scheduled,
                   fd.landing_actual,
                   fd.gate_arr_scheduled,
                   fd.gate_arr_actual,
                   fd.aircraft_type,
                   fd.tail_number,
                   fd.pnr,
                   fd.seat,
                   fd.seat_type,
                   fd.cabin_class,
                   fd.flight_reason,
                   fd.notes AS flight_notes,
                   rd.carrier AS rail_carrier,
                   rd.train_number,
                   rd.service_class,
                   rd.coach_number,
                   rd.seats AS rail_seats,
                   rd.confirmation_num AS rail_confirmation,
                   rd.booking_site AS rail_booking_site,
                   rd.notes AS rail_notes,
                   bd.ship_name,
                   bd.cabin_type,
                   bd.cabin_number,
                   bd.confirmation_num AS boat_confirmation,
                   bd.booking_site AS boat_booking_site,
                   bd.notes AS boat_notes,
                    td.carrier_name AS transport_carrier,
                    td.vehicle_description,
                    td.confirmation_num AS transport_confirmation,
                    td.notes AS transport_notes,
                    h.cost_amount,
                    h.cost_currency
                FROM hops h
               LEFT JOIN flight_details fd ON fd.hop_id = h.id
               LEFT JOIN rail_details rd ON rd.hop_id = h.id
               LEFT JOIN boat_details bd ON bd.hop_id = h.id
               LEFT JOIN transport_details td ON td.hop_id = h.id
               WHERE h.id = ? AND h.user_id = ?"#,
            self.id,
            self.user_id,
        )
        .fetch_optional(pool)
        .await?;

        row.map(DetailRow::try_from).transpose()
    }
}

#[cfg(test)]
mod tests {
    use crate::db::{
        hops::{Create, FlightDetail, GetAll, TravelType, sample_hop},
        tests::{test_pool, test_user},
    };

    use super::GetById;

    #[tokio::test]
    async fn get_by_id_returns_hop_with_flight_detail() {
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
            trip_id: "trip-detail",
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

        let detail = GetById {
            id: hop_id,
            user_id,
        }
        .execute(&pool)
        .await
        .expect("get_by_id failed")
        .expect("should find row");

        assert_eq!(detail.travel_type, TravelType::Air);
        assert_eq!(detail.origin_name, "LHR");
        assert_eq!(detail.dest_name, "JFK");

        let fd = detail.flight_detail.expect("should have flight detail");
        assert_eq!(fd.airline, "BA");
        assert_eq!(fd.flight_number, "BA178");
        assert_eq!(fd.aircraft_type, "777");
        assert_eq!(fd.cabin_class, "economy");
    }

    #[tokio::test]
    async fn get_by_id_returns_none_for_wrong_user() {
        let pool = test_pool().await;
        let alice = test_user(&pool, "alice").await;
        let bob = test_user(&pool, "bob").await;

        let hop = sample_hop(TravelType::Air, "LHR", "JFK", "2024-05-02", "2024-05-02");
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

        let result = GetById {
            id: hop_id,
            user_id: bob,
        }
        .execute(&pool)
        .await
        .expect("get_by_id failed");
        assert!(result.is_none(), "bob should not see alice's hop");
    }

    #[tokio::test]
    async fn get_by_id_returns_none_for_nonexistent_id() {
        let pool = test_pool().await;
        let user_id = test_user(&pool, "alice").await;

        let result = GetById { id: 99999, user_id }
            .execute(&pool)
            .await
            .expect("get_by_id failed");
        assert!(result.is_none());
    }
}

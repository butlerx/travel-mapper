use super::SummaryRow;
use sqlx::SqlitePool;

/// Fetch hop summaries for unassigned hops (no trip).
pub struct GetUnassigned {
    pub user_id: i64,
}

impl GetUnassigned {
    /// # Errors
    ///
    /// Returns an error if the query fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<Vec<SummaryRow>, sqlx::Error> {
        sqlx::query_as!(
            SummaryRow,
            r#"SELECT
                   h.id as "id!: i64",
                   h.travel_type as "travel_type!: String",
                   h.origin_name as "origin_name!: String",
                   h.origin_lat as "origin_lat!: f64",
                   h.origin_lng as "origin_lng!: f64",
                   h.dest_name as "dest_name!: String",
                   h.dest_lat as "dest_lat!: f64",
                   h.dest_lng as "dest_lng!: f64",
                   h.start_date as "start_date!: String",
                   COALESCE(fd.airline, rd.carrier, bd.ship_name, td.carrier_name) AS "carrier: String"
               FROM hops h
               LEFT JOIN flight_details fd ON fd.hop_id = h.id
               LEFT JOIN rail_details rd ON rd.hop_id = h.id
               LEFT JOIN boat_details bd ON bd.hop_id = h.id
               LEFT JOIN transport_details td ON td.hop_id = h.id
               WHERE h.user_id = ? AND h.user_trip_id IS NULL
               ORDER BY h.start_date DESC, h.id DESC
               LIMIT 200"#,
            self.user_id,
        )
        .fetch_all(pool)
        .await
    }
}

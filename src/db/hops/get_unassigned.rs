use super::SummaryRow;
use sqlx::SqlitePool;

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
                   id as "id!: i64",
                   travel_type as "travel_type!: String",
                   origin_name as "origin_name!: String",
                   dest_name as "dest_name!: String",
                   start_date as "start_date!: String"
               FROM hops
               WHERE user_id = ? AND user_trip_id IS NULL
               ORDER BY start_date DESC, id DESC
               LIMIT 200"#,
            self.user_id,
        )
        .fetch_all(pool)
        .await
    }
}

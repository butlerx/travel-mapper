use sqlx::SqlitePool;

pub struct ExistsInTrip {
    pub hop_id: i64,
    pub user_id: i64,
    pub trip_id: i64,
}

impl ExistsInTrip {
    /// # Errors
    ///
    /// Returns an error if the query fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<bool, sqlx::Error> {
        let exists = sqlx::query_scalar!(
            r#"SELECT EXISTS(
                   SELECT 1 FROM hops
                   WHERE id = ? AND user_id = ? AND user_trip_id = ?
               ) as "exists!: i64""#,
            self.hop_id,
            self.user_id,
            self.trip_id,
        )
        .fetch_one(pool)
        .await?;
        Ok(exists == 1)
    }
}

use sqlx::SqlitePool;

pub struct Get<'a> {
    pub query: &'a str,
}

impl Get<'_> {
    /// # Errors
    ///
    /// Returns an error if the query fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<Option<(f64, f64)>, sqlx::Error> {
        let row = sqlx::query!(
            "SELECT lat, lng FROM geocode_cache WHERE query = ?",
            self.query,
        )
        .fetch_optional(pool)
        .await?;
        Ok(row.map(|r| (r.lat, r.lng)))
    }
}

pub struct Upsert<'a> {
    pub query: &'a str,
    pub lat: f64,
    pub lng: f64,
}

impl Upsert<'_> {
    /// # Errors
    ///
    /// Returns an error if the upsert fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r"INSERT INTO geocode_cache (query, lat, lng)
               VALUES (?, ?, ?)
               ON CONFLICT(query) DO UPDATE SET
                   lat = excluded.lat,
                   lng = excluded.lng,
                   created_at = datetime('now')",
            self.query,
            self.lat,
            self.lng,
        )
        .execute(pool)
        .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::tests::test_pool;

    #[tokio::test]
    async fn upsert_and_get_roundtrip() {
        let pool = test_pool().await;

        Upsert {
            query: "Dublin, Ireland",
            lat: 53.35,
            lng: -6.26,
        }
        .execute(&pool)
        .await
        .expect("upsert failed");

        let coords = Get {
            query: "Dublin, Ireland",
        }
        .execute(&pool)
        .await
        .expect("get failed")
        .expect("should find cached coords");

        assert!((coords.0 - 53.35).abs() < f64::EPSILON);
        assert!((coords.1 - (-6.26)).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn get_missing_returns_none() {
        let pool = test_pool().await;
        let result = Get { query: "Nowhere" }
            .execute(&pool)
            .await
            .expect("get failed");
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn upsert_updates_existing() {
        let pool = test_pool().await;

        Upsert {
            query: "Cork",
            lat: 51.0,
            lng: -8.0,
        }
        .execute(&pool)
        .await
        .expect("first upsert");

        Upsert {
            query: "Cork",
            lat: 51.9,
            lng: -8.47,
        }
        .execute(&pool)
        .await
        .expect("second upsert");

        let coords = Get { query: "Cork" }
            .execute(&pool)
            .await
            .expect("get failed")
            .expect("should find cached coords");

        assert!((coords.0 - 51.9).abs() < f64::EPSILON);
        assert!((coords.1 - (-8.47)).abs() < f64::EPSILON);
    }
}

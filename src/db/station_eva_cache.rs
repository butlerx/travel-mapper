use sqlx::SqlitePool;

pub struct Row {
    pub eva_number: i64,
    pub name: String,
    pub lat: Option<f64>,
    pub lng: Option<f64>,
    pub updated_at: String,
}

pub struct Upsert<'a> {
    pub eva_number: i64,
    pub name: &'a str,
    pub lat: Option<f64>,
    pub lng: Option<f64>,
}

impl Upsert<'_> {
    /// # Errors
    ///
    /// Returns an error if the upsert fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r"INSERT INTO station_eva_cache (eva_number, name, lat, lng)
               VALUES (?, ?, ?, ?)
               ON CONFLICT(eva_number) DO UPDATE SET
                   name = excluded.name,
                   lat = excluded.lat,
                   lng = excluded.lng,
                   updated_at = datetime('now')",
            self.eva_number,
            self.name,
            self.lat,
            self.lng,
        )
        .execute(pool)
        .await?;
        Ok(())
    }
}

pub struct GetByName<'a> {
    pub name: &'a str,
}

impl GetByName<'_> {
    /// # Errors
    ///
    /// Returns an error if the query fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<Option<Row>, sqlx::Error> {
        sqlx::query_as!(
            Row,
            r#"SELECT
                   eva_number as "eva_number!: i64",
                   name as "name!: String",
                   lat,
                   lng,
                   updated_at as "updated_at!: String"
               FROM station_eva_cache
               WHERE name = ?
               LIMIT 1"#,
            self.name,
        )
        .fetch_optional(pool)
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::tests::test_pool;

    #[tokio::test]
    async fn upsert_and_get_by_name_roundtrip() {
        let pool = test_pool().await;

        Upsert {
            eva_number: 8_000_105,
            name: "Frankfurt(Main)Hbf",
            lat: Some(50.1071),
            lng: Some(8.6636),
        }
        .execute(&pool)
        .await
        .expect("upsert failed");

        let row = GetByName {
            name: "Frankfurt(Main)Hbf",
        }
        .execute(&pool)
        .await
        .expect("get failed")
        .expect("should find station");

        assert_eq!(row.eva_number, 8_000_105);
        assert_eq!(row.name, "Frankfurt(Main)Hbf");
        assert!((row.lat.unwrap() - 50.1071).abs() < 0.001);
        assert!((row.lng.unwrap() - 8.6636).abs() < 0.001);
    }

    #[tokio::test]
    async fn upsert_updates_existing_station() {
        let pool = test_pool().await;

        Upsert {
            eva_number: 8_000_105,
            name: "Frankfurt Hbf",
            lat: Some(50.107),
            lng: Some(8.663),
        }
        .execute(&pool)
        .await
        .expect("first upsert");

        Upsert {
            eva_number: 8_000_105,
            name: "Frankfurt(Main)Hbf",
            lat: Some(50.1071),
            lng: Some(8.6636),
        }
        .execute(&pool)
        .await
        .expect("second upsert");

        let row = GetByName {
            name: "Frankfurt(Main)Hbf",
        }
        .execute(&pool)
        .await
        .expect("get failed")
        .expect("should find updated station");

        assert_eq!(row.name, "Frankfurt(Main)Hbf");
    }

    #[tokio::test]
    async fn get_by_name_returns_none_for_missing() {
        let pool = test_pool().await;

        let row = GetByName {
            name: "Nonexistent Station",
        }
        .execute(&pool)
        .await
        .expect("get failed");

        assert!(row.is_none());
    }
}

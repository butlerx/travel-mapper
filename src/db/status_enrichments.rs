use sqlx::SqlitePool;

/// Database row for flight or rail status enrichment data.
#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
pub struct Row {
    pub id: i64,
    pub hop_id: i64,
    pub provider: String,
    pub status: String,
    pub delay_minutes: Option<i64>,
    pub dep_gate: String,
    pub dep_terminal: String,
    pub arr_gate: String,
    pub arr_terminal: String,
    pub dep_platform: String,
    pub arr_platform: String,
    pub raw_json: String,
    pub fetched_at: String,
}

/// Insert or update status enrichment data.
pub struct Upsert<'a> {
    pub hop_id: i64,
    pub provider: &'a str,
    pub status: &'a str,
    pub delay_minutes: Option<i64>,
    pub dep_gate: &'a str,
    pub dep_terminal: &'a str,
    pub arr_gate: &'a str,
    pub arr_terminal: &'a str,
    pub dep_platform: &'a str,
    pub arr_platform: &'a str,
    pub raw_json: &'a str,
}

impl Upsert<'_> {
    /// # Errors
    ///
    /// Returns an error if the upsert fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r"INSERT INTO status_enrichments
                   (hop_id, provider, status, delay_minutes,
                    dep_gate, dep_terminal, arr_gate, arr_terminal,
                    dep_platform, arr_platform, raw_json)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
               ON CONFLICT(hop_id, provider) DO UPDATE SET
                   status = excluded.status,
                   delay_minutes = excluded.delay_minutes,
                   dep_gate = excluded.dep_gate,
                   dep_terminal = excluded.dep_terminal,
                   arr_gate = excluded.arr_gate,
                   arr_terminal = excluded.arr_terminal,
                   dep_platform = excluded.dep_platform,
                   arr_platform = excluded.arr_platform,
                   raw_json = excluded.raw_json,
                   fetched_at = datetime('now')",
            self.hop_id,
            self.provider,
            self.status,
            self.delay_minutes,
            self.dep_gate,
            self.dep_terminal,
            self.arr_gate,
            self.arr_terminal,
            self.dep_platform,
            self.arr_platform,
            self.raw_json,
        )
        .execute(pool)
        .await?;
        Ok(())
    }
}

/// Fetch the most recent status enrichment for a hop.
pub struct GetByHopId {
    pub hop_id: i64,
}

impl GetByHopId {
    /// # Errors
    ///
    /// Returns an error if the query fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<Option<Row>, sqlx::Error> {
        sqlx::query_as!(
            Row,
            r#"SELECT
                   id as "id!: i64",
                   hop_id as "hop_id!: i64",
                   provider as "provider!: String",
                   status as "status!: String",
                   delay_minutes,
                   dep_gate as "dep_gate!: String",
                   dep_terminal as "dep_terminal!: String",
                   arr_gate as "arr_gate!: String",
                   arr_terminal as "arr_terminal!: String",
                   dep_platform as "dep_platform!: String",
                   arr_platform as "arr_platform!: String",
                   raw_json as "raw_json!: String",
                   fetched_at as "fetched_at!: String"
               FROM status_enrichments
               WHERE hop_id = ?
               ORDER BY fetched_at DESC
               LIMIT 1"#,
            self.hop_id,
        )
        .fetch_optional(pool)
        .await
    }
}

/// Fetch the most recent status enrichments for multiple hops.
pub struct GetByHopIds {
    pub hop_ids: Vec<i64>,
}

impl GetByHopIds {
    /// # Errors
    ///
    /// Returns an error if the query fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<Vec<Row>, sqlx::Error> {
        if self.hop_ids.is_empty() {
            return Ok(Vec::new());
        }
        let ids_json =
            serde_json::to_string(&self.hop_ids).map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
        sqlx::query_as!(
            Row,
            r#"SELECT
                   se.id as "id!: i64",
                   se.hop_id as "hop_id!: i64",
                   se.provider as "provider!: String",
                   se.status as "status!: String",
                   se.delay_minutes,
                   se.dep_gate as "dep_gate!: String",
                   se.dep_terminal as "dep_terminal!: String",
                   se.arr_gate as "arr_gate!: String",
                   se.arr_terminal as "arr_terminal!: String",
                   se.dep_platform as "dep_platform!: String",
                   se.arr_platform as "arr_platform!: String",
                   se.raw_json as "raw_json!: String",
                   se.fetched_at as "fetched_at!: String"
               FROM status_enrichments se
               INNER JOIN (
                   SELECT hop_id, MAX(fetched_at) AS max_fetched
                   FROM status_enrichments
                   WHERE hop_id IN (SELECT value FROM json_each(?))
                   GROUP BY hop_id
               ) latest ON se.hop_id = latest.hop_id AND se.fetched_at = latest.max_fetched
               WHERE se.hop_id IN (SELECT value FROM json_each(?))"#,
            ids_json,
            ids_json,
        )
        .fetch_all(pool)
        .await
    }
}

/// Fetch the most recent status enrichment for a hop from a specific provider.
pub struct GetByHopIdAndProvider<'a> {
    pub hop_id: i64,
    pub provider: &'a str,
}

impl GetByHopIdAndProvider<'_> {
    /// # Errors
    ///
    /// Returns an error if the query fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<Option<Row>, sqlx::Error> {
        sqlx::query_as!(
            Row,
            r#"SELECT
                   id as "id!: i64",
                   hop_id as "hop_id!: i64",
                   provider as "provider!: String",
                   status as "status!: String",
                   delay_minutes,
                   dep_gate as "dep_gate!: String",
                   dep_terminal as "dep_terminal!: String",
                   arr_gate as "arr_gate!: String",
                   arr_terminal as "arr_terminal!: String",
                   dep_platform as "dep_platform!: String",
                   arr_platform as "arr_platform!: String",
                   raw_json as "raw_json!: String",
                   fetched_at as "fetched_at!: String"
               FROM status_enrichments
               WHERE hop_id = ? AND provider = ?
               ORDER BY fetched_at DESC
               LIMIT 1"#,
            self.hop_id,
            self.provider,
        )
        .fetch_optional(pool)
        .await
    }
}

/// Fetch status enrichments for multiple hops from a specific provider.
pub struct GetByHopIdsAndProvider<'a> {
    pub hop_ids: Vec<i64>,
    pub provider: &'a str,
}

impl GetByHopIdsAndProvider<'_> {
    /// # Errors
    ///
    /// Returns an error if the query fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<Vec<Row>, sqlx::Error> {
        if self.hop_ids.is_empty() {
            return Ok(Vec::new());
        }
        let ids_json =
            serde_json::to_string(&self.hop_ids).map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
        sqlx::query_as!(
            Row,
            r#"SELECT
                   se.id as "id!: i64",
                   se.hop_id as "hop_id!: i64",
                   se.provider as "provider!: String",
                   se.status as "status!: String",
                   se.delay_minutes,
                   se.dep_gate as "dep_gate!: String",
                   se.dep_terminal as "dep_terminal!: String",
                   se.arr_gate as "arr_gate!: String",
                   se.arr_terminal as "arr_terminal!: String",
                   se.dep_platform as "dep_platform!: String",
                   se.arr_platform as "arr_platform!: String",
                   se.raw_json as "raw_json!: String",
                   se.fetched_at as "fetched_at!: String"
               FROM status_enrichments se
               WHERE se.hop_id IN (SELECT value FROM json_each(?))
                 AND se.provider = ?"#,
            ids_json,
            self.provider,
        )
        .fetch_all(pool)
        .await
    }
}

/// Delete all status enrichments for a hop.
pub struct DeleteByHopId {
    pub hop_id: i64,
}

impl DeleteByHopId {
    /// # Errors
    ///
    /// Returns an error if the delete fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<u64, sqlx::Error> {
        let result = sqlx::query!(
            "DELETE FROM status_enrichments WHERE hop_id = ?",
            self.hop_id,
        )
        .execute(pool)
        .await?;
        Ok(result.rows_affected())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{
        hops::{Create, GetAll, TravelType, sample_hop},
        tests::{test_pool, test_user},
    };

    async fn insert_hop(pool: &SqlitePool, user_id: i64) -> i64 {
        let hop = sample_hop(TravelType::Air, "LHR", "JFK", "2024-05-01", "2024-05-01");
        Create {
            trip_id: "trip-enrichment",
            user_id,
            hops: &[hop],
        }
        .execute(pool)
        .await
        .expect("insert hop");
        let rows = GetAll {
            user_id,
            travel_type_filter: None,
        }
        .execute(pool)
        .await
        .expect("get hops");
        rows[0].id
    }

    #[tokio::test]
    async fn upsert_and_get_by_hop_id_roundtrip() {
        let pool = test_pool().await;
        let user_id = test_user(&pool, "alice").await;
        let hop_id = insert_hop(&pool, user_id).await;

        Upsert {
            hop_id,
            provider: "aviationstack",
            status: "landed",
            delay_minutes: Some(15),
            dep_gate: "B22",
            dep_terminal: "5",
            arr_gate: "C10",
            arr_terminal: "1",
            dep_platform: "",
            arr_platform: "",
            raw_json: r#"{"test":true}"#,
        }
        .execute(&pool)
        .await
        .expect("upsert failed");

        let row = GetByHopId { hop_id }
            .execute(&pool)
            .await
            .expect("get failed")
            .expect("should find enrichment");

        assert_eq!(row.hop_id, hop_id);
        assert_eq!(row.status, "landed");
        assert_eq!(row.delay_minutes, Some(15));
        assert_eq!(row.dep_gate, "B22");
        assert_eq!(row.arr_terminal, "1");
    }

    #[tokio::test]
    async fn upsert_updates_existing_enrichment() {
        let pool = test_pool().await;
        let user_id = test_user(&pool, "alice").await;
        let hop_id = insert_hop(&pool, user_id).await;

        Upsert {
            hop_id,
            provider: "aviationstack",
            status: "active",
            delay_minutes: None,
            dep_gate: "",
            dep_terminal: "",
            arr_gate: "",
            arr_terminal: "",
            dep_platform: "",
            arr_platform: "",
            raw_json: "{}",
        }
        .execute(&pool)
        .await
        .expect("first upsert");

        Upsert {
            hop_id,
            provider: "aviationstack",
            status: "landed",
            delay_minutes: Some(30),
            dep_gate: "A1",
            dep_terminal: "2",
            arr_gate: "D5",
            arr_terminal: "3",
            dep_platform: "",
            arr_platform: "",
            raw_json: r#"{"updated":true}"#,
        }
        .execute(&pool)
        .await
        .expect("second upsert");

        let row = GetByHopId { hop_id }
            .execute(&pool)
            .await
            .expect("get failed")
            .expect("should find enrichment");

        assert_eq!(row.status, "landed");
        assert_eq!(row.delay_minutes, Some(30));
        assert_eq!(row.dep_gate, "A1");
    }

    #[tokio::test]
    async fn get_by_hop_ids_returns_matching_enrichments() {
        let pool = test_pool().await;
        let user_id = test_user(&pool, "alice").await;
        let hop1 = insert_hop(&pool, user_id).await;

        let hop2_hop = sample_hop(TravelType::Air, "JFK", "LAX", "2024-06-01", "2024-06-01");
        Create {
            trip_id: "trip-enrichment-2",
            user_id,
            hops: &[hop2_hop],
        }
        .execute(&pool)
        .await
        .expect("insert hop2");
        let all = GetAll {
            user_id,
            travel_type_filter: None,
        }
        .execute(&pool)
        .await
        .expect("get all");
        let hop2 = all.iter().find(|h| h.origin_name == "JFK").unwrap().id;

        Upsert {
            hop_id: hop1,
            provider: "aviationstack",
            status: "landed",
            delay_minutes: Some(5),
            dep_gate: "",
            dep_terminal: "",
            arr_gate: "",
            arr_terminal: "",
            dep_platform: "",
            arr_platform: "",
            raw_json: "{}",
        }
        .execute(&pool)
        .await
        .expect("upsert 1");

        Upsert {
            hop_id: hop2,
            provider: "aviationstack",
            status: "active",
            delay_minutes: None,
            dep_gate: "",
            dep_terminal: "",
            arr_gate: "",
            arr_terminal: "",
            dep_platform: "",
            arr_platform: "",
            raw_json: "{}",
        }
        .execute(&pool)
        .await
        .expect("upsert 2");

        let results = GetByHopIds {
            hop_ids: vec![hop1, hop2],
        }
        .execute(&pool)
        .await
        .expect("batch get failed");

        assert_eq!(results.len(), 2);
    }

    #[tokio::test]
    async fn get_by_hop_ids_empty_returns_empty() {
        let pool = test_pool().await;
        let results = GetByHopIds {
            hop_ids: Vec::new(),
        }
        .execute(&pool)
        .await
        .expect("empty batch get failed");
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn delete_by_hop_id_removes_enrichment() {
        let pool = test_pool().await;
        let user_id = test_user(&pool, "alice").await;
        let hop_id = insert_hop(&pool, user_id).await;

        Upsert {
            hop_id,
            provider: "aviationstack",
            status: "landed",
            delay_minutes: None,
            dep_gate: "",
            dep_terminal: "",
            arr_gate: "",
            arr_terminal: "",
            dep_platform: "",
            arr_platform: "",
            raw_json: "{}",
        }
        .execute(&pool)
        .await
        .expect("upsert");

        let deleted = DeleteByHopId { hop_id }
            .execute(&pool)
            .await
            .expect("delete failed");
        assert_eq!(deleted, 1);

        let row = GetByHopId { hop_id }
            .execute(&pool)
            .await
            .expect("get failed");
        assert!(row.is_none());
    }
}

use sqlx::SqlitePool;

/// Row returned by attachment queries.
#[derive(Debug, Clone)]
pub struct Row {
    pub id: i64,
    pub hop_id: i64,
    pub user_id: i64,
    pub filename: String,
    pub content_type: String,
    pub size_bytes: i64,
    pub storage_path: String,
    pub created_at: String,
}

/// Insert a new attachment record.
pub struct Create<'a> {
    pub hop_id: i64,
    pub user_id: i64,
    pub filename: &'a str,
    pub content_type: &'a str,
    pub size_bytes: i64,
    pub storage_path: &'a str,
}

impl Create<'_> {
    /// # Errors
    ///
    /// Returns an error if the insert fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<i64, sqlx::Error> {
        let result = sqlx::query!(
            "INSERT INTO attachments (hop_id, user_id, filename, content_type, size_bytes, storage_path)
             VALUES (?, ?, ?, ?, ?, ?)",
            self.hop_id,
            self.user_id,
            self.filename,
            self.content_type,
            self.size_bytes,
            self.storage_path,
        )
        .execute(pool)
        .await?;
        Ok(result.last_insert_rowid())
    }
}

/// Fetch all attachments for a journey, newest first.
pub struct GetByHopId {
    pub hop_id: i64,
    pub user_id: i64,
}

impl GetByHopId {
    /// # Errors
    ///
    /// Returns an error if the query fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<Vec<Row>, sqlx::Error> {
        let rows = sqlx::query_as!(
            Row,
            r#"SELECT
                 id as "id!",
                 hop_id as "hop_id!",
                 user_id as "user_id!",
                 filename as "filename!",
                 content_type as "content_type!",
                 size_bytes as "size_bytes!",
                 storage_path as "storage_path!",
                 created_at as "created_at!"
             FROM attachments
             WHERE hop_id = ? AND user_id = ?
             ORDER BY created_at DESC"#,
            self.hop_id,
            self.user_id,
        )
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }
}

/// Fetch a single attachment by ID (scoped to user).
pub struct GetById {
    pub id: i64,
    pub user_id: i64,
}

impl GetById {
    /// # Errors
    ///
    /// Returns an error if the query fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<Option<Row>, sqlx::Error> {
        let row = sqlx::query_as!(
            Row,
            r#"SELECT
                 id as "id!",
                 hop_id as "hop_id!",
                 user_id as "user_id!",
                 filename as "filename!",
                 content_type as "content_type!",
                 size_bytes as "size_bytes!",
                 storage_path as "storage_path!",
                 created_at as "created_at!"
             FROM attachments
             WHERE id = ? AND user_id = ?"#,
            self.id,
            self.user_id,
        )
        .fetch_optional(pool)
        .await?;
        Ok(row)
    }
}

/// Delete an attachment by ID (scoped to user). Returns `true` if a row was deleted.
pub struct Delete {
    pub id: i64,
    pub user_id: i64,
}

impl Delete {
    /// # Errors
    ///
    /// Returns an error if the query fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            "DELETE FROM attachments WHERE id = ? AND user_id = ?",
            self.id,
            self.user_id,
        )
        .execute(pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }
}

/// Count attachments for a journey.
pub struct CountByHopId {
    pub hop_id: i64,
    pub user_id: i64,
}

impl CountByHopId {
    /// # Errors
    ///
    /// Returns an error if the query fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<i64, sqlx::Error> {
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM attachments WHERE hop_id = ? AND user_id = ?",
            self.hop_id,
            self.user_id,
        )
        .fetch_one(pool)
        .await?;
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::tests::{test_pool, test_user};

    async fn insert_hop(pool: &SqlitePool, user_id: i64) -> i64 {
        use crate::db::hops::{Create as HopCreate, GetAll, TravelType};
        use crate::server::test_helpers::sample_hop;

        HopCreate {
            trip_id: "trip-test",
            user_id,
            hops: &[sample_hop(
                TravelType::Air,
                "DUB",
                "LHR",
                "2024-01-01",
                "2024-01-01",
            )],
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
    async fn attachment_crud() {
        let pool = test_pool().await;
        let user_id = test_user(&pool, "alice").await;
        let hop_id = insert_hop(&pool, user_id).await;

        let att_id = Create {
            hop_id,
            user_id,
            filename: "boarding-pass.jpg",
            content_type: "image/jpeg",
            size_bytes: 12345,
            storage_path: "1/1/abc.jpg",
        }
        .execute(&pool)
        .await
        .expect("create attachment");
        assert!(att_id > 0);

        let by_hop = GetByHopId { hop_id, user_id }
            .execute(&pool)
            .await
            .expect("get by hop");
        assert_eq!(by_hop.len(), 1);
        assert_eq!(by_hop[0].filename, "boarding-pass.jpg");

        let by_id = GetById {
            id: att_id,
            user_id,
        }
        .execute(&pool)
        .await
        .expect("get by id")
        .expect("should exist");
        assert_eq!(by_id.storage_path, "1/1/abc.jpg");

        let count = CountByHopId { hop_id, user_id }
            .execute(&pool)
            .await
            .expect("count");
        assert_eq!(count, 1);

        let deleted = Delete {
            id: att_id,
            user_id,
        }
        .execute(&pool)
        .await
        .expect("delete");
        assert!(deleted);

        let after_delete = GetByHopId { hop_id, user_id }
            .execute(&pool)
            .await
            .expect("get after delete");
        assert!(after_delete.is_empty());
    }

    #[tokio::test]
    async fn attachment_scoped_to_user() {
        let pool = test_pool().await;
        let alice_id = test_user(&pool, "alice").await;
        let bob_id = test_user(&pool, "bob").await;
        let hop_id = insert_hop(&pool, alice_id).await;

        Create {
            hop_id,
            user_id: alice_id,
            filename: "photo.png",
            content_type: "image/png",
            size_bytes: 100,
            storage_path: "a/b/c.png",
        }
        .execute(&pool)
        .await
        .expect("create");

        let bob_result = GetByHopId {
            hop_id,
            user_id: bob_id,
        }
        .execute(&pool)
        .await
        .expect("get for bob");
        assert!(
            bob_result.is_empty(),
            "bob should not see alice's attachments"
        );
    }
}

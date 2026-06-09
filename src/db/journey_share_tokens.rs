use sqlx::SqlitePool;

/// Store a new journey share token for a user and hop.
pub struct Create<'a> {
    pub user_id: i64,
    pub hop_id: i64,
    pub token_hash: &'a str,
    pub expires_at: &'a str,
}

impl Create<'_> {
    /// # Errors
    ///
    /// Returns an error if the insert fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<i64, sqlx::Error> {
        let result = sqlx::query!(
            "INSERT INTO journey_share_tokens (user_id, hop_id, token_hash, expires_at) VALUES (?, ?, ?, ?)",
            self.user_id,
            self.hop_id,
            self.token_hash,
            self.expires_at,
        )
        .execute(pool)
        .await?;
        Ok(result.last_insert_rowid())
    }
}

/// Row returned by journey share token queries.
pub struct Row {
    pub id: i64,
    pub user_id: i64,
    pub hop_id: i64,
    pub token_hash: String,
    pub expires_at: String,
    pub created_at: String,
}

/// Resolve a journey share token hash to its token row.
pub struct GetByTokenHash<'a> {
    pub token_hash: &'a str,
}

impl GetByTokenHash<'_> {
    /// # Errors
    ///
    /// Returns an error if the query fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<Option<Row>, sqlx::Error> {
        sqlx::query_as!(
            Row,
            r#"SELECT
                   id as "id!: i64",
                   user_id as "user_id!: i64",
                   hop_id as "hop_id!: i64",
                   token_hash,
                   expires_at,
                   created_at
               FROM journey_share_tokens
               WHERE token_hash = ?"#,
            self.token_hash,
        )
        .fetch_optional(pool)
        .await
    }
}

/// List all journey share tokens for a hop.
pub struct GetByHopId {
    pub hop_id: i64,
}

impl GetByHopId {
    /// # Errors
    ///
    /// Returns an error if the query fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<Vec<Row>, sqlx::Error> {
        let rows = sqlx::query_as!(
            Row,
            r#"SELECT
                   id as "id!: i64",
                   user_id as "user_id!: i64",
                   hop_id as "hop_id!: i64",
                   token_hash,
                   expires_at,
                   created_at
               FROM journey_share_tokens
               WHERE hop_id = ?
               ORDER BY created_at DESC"#,
            self.hop_id,
        )
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }
}

/// Delete a journey share token by ID, scoped to the owning user.
pub struct Delete {
    pub id: i64,
    pub user_id: i64,
}

impl Delete {
    /// # Errors
    ///
    /// Returns an error if the delete fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            "DELETE FROM journey_share_tokens WHERE id = ? AND user_id = ?",
            self.id,
            self.user_id,
        )
        .execute(pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }
}

/// Purge expired journey share tokens.
pub struct DeleteExpired;

impl DeleteExpired {
    /// # Errors
    ///
    /// Returns an error if the delete fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<u64, sqlx::Error> {
        let result =
            sqlx::query!("DELETE FROM journey_share_tokens WHERE expires_at < datetime('now')")
                .execute(pool)
                .await?;
        Ok(result.rows_affected())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{
        hops::{Create as CreateHops, GetAll as GetAllHops, TravelType, sample_hop},
        tests::{test_pool, test_user},
    };

    async fn test_hop_id(pool: &SqlitePool, user_id: i64) -> i64 {
        let hop = sample_hop(TravelType::Air, "DUB", "LHR", "2026-01-01", "2026-01-01");
        CreateHops {
            trip_id: "trip-share-token",
            user_id,
            hops: &[hop],
        }
        .execute(pool)
        .await
        .expect("insert hop failed");
        GetAllHops {
            user_id,
            travel_type_filter: None,
        }
        .execute(pool)
        .await
        .expect("list hops failed")[0]
            .id
    }

    #[tokio::test]
    async fn journey_share_token_crud() {
        let pool = test_pool().await;
        let user_id = test_user(&pool, "alice").await;
        let hop_id = test_hop_id(&pool, user_id).await;

        let token_id = Create {
            user_id,
            hop_id,
            token_hash: "journey_share_hash_1",
            expires_at: "2999-01-01 00:00:00",
        }
        .execute(&pool)
        .await
        .expect("journey share token create failed");
        assert!(token_id > 0);

        let resolved = GetByTokenHash {
            token_hash: "journey_share_hash_1",
        }
        .execute(&pool)
        .await
        .expect("journey share token lookup failed")
        .expect("journey share token should resolve");
        assert_eq!(resolved.user_id, user_id);
        assert_eq!(resolved.hop_id, hop_id);

        let tokens = GetByHopId { hop_id }
            .execute(&pool)
            .await
            .expect("journey share token list failed");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].token_hash, "journey_share_hash_1");

        let deleted = Delete {
            id: token_id,
            user_id,
        }
        .execute(&pool)
        .await
        .expect("journey share token delete failed");
        assert!(deleted);

        let after_delete = GetByTokenHash {
            token_hash: "journey_share_hash_1",
        }
        .execute(&pool)
        .await
        .expect("journey share token lookup failed");
        assert!(after_delete.is_none());
    }

    #[tokio::test]
    async fn journey_share_token_delete_scoped_to_user() {
        let pool = test_pool().await;
        let alice_id = test_user(&pool, "alice").await;
        let bob_id = test_user(&pool, "bob").await;
        let hop_id = test_hop_id(&pool, alice_id).await;

        let token_id = Create {
            user_id: alice_id,
            hop_id,
            token_hash: "alice_journey_share_hash",
            expires_at: "2999-01-01 00:00:00",
        }
        .execute(&pool)
        .await
        .expect("create failed");

        let not_deleted = Delete {
            id: token_id,
            user_id: bob_id,
        }
        .execute(&pool)
        .await
        .expect("delete failed");
        assert!(!not_deleted);

        let still_exists = GetByTokenHash {
            token_hash: "alice_journey_share_hash",
        }
        .execute(&pool)
        .await
        .expect("lookup failed");
        assert!(still_exists.is_some());
    }

    #[tokio::test]
    async fn journey_share_token_delete_expired() {
        let pool = test_pool().await;
        let user_id = test_user(&pool, "alice").await;
        let hop_id = test_hop_id(&pool, user_id).await;

        Create {
            user_id,
            hop_id,
            token_hash: "expired_journey_share_hash",
            expires_at: "2000-01-01 00:00:00",
        }
        .execute(&pool)
        .await
        .expect("create expired failed");

        Create {
            user_id,
            hop_id,
            token_hash: "valid_journey_share_hash",
            expires_at: "2999-01-01 00:00:00",
        }
        .execute(&pool)
        .await
        .expect("create valid failed");

        let purged = DeleteExpired.execute(&pool).await.expect("purge failed");
        assert_eq!(purged, 1);

        let valid = GetByTokenHash {
            token_hash: "valid_journey_share_hash",
        }
        .execute(&pool)
        .await
        .expect("lookup valid failed");
        assert!(valid.is_some());

        let expired = GetByTokenHash {
            token_hash: "expired_journey_share_hash",
        }
        .execute(&pool)
        .await
        .expect("lookup expired failed");
        assert!(expired.is_none());
    }
}

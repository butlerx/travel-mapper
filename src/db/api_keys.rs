use sqlx::SqlitePool;

/// Store a new API key hash for a user.
pub struct Create<'a> {
    pub user_id: i64,
    pub key_hash: &'a str,
    pub label: &'a str,
}

impl Create<'_> {
    /// # Errors
    ///
    /// Returns an error if the insert fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<i64, sqlx::Error> {
        let result = sqlx::query!(
            "INSERT INTO api_keys (user_id, key_hash, label) VALUES (?, ?, ?)",
            self.user_id,
            self.key_hash,
            self.label,
        )
        .execute(pool)
        .await?;
        Ok(result.last_insert_rowid())
    }
}

/// Resolve an API key hash to its owning user ID.
pub struct GetUserIdByHash<'a> {
    pub key_hash: &'a str,
}

impl GetUserIdByHash<'_> {
    /// # Errors
    ///
    /// Returns an error if the query fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<Option<i64>, sqlx::Error> {
        let row = sqlx::query_scalar!(
            "SELECT user_id FROM api_keys WHERE key_hash = ?",
            self.key_hash
        )
        .fetch_optional(pool)
        .await?;
        Ok(row)
    }
}

/// A single API key row for listing in the settings UI.
pub struct Row {
    pub id: i64,
    pub label: String,
    pub created_at: String,
}

/// List all API keys for a user.
pub struct GetByUserId {
    pub user_id: i64,
}

impl GetByUserId {
    /// # Errors
    ///
    /// Returns an error if the query fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<Vec<Row>, sqlx::Error> {
        let rows = sqlx::query_as!(
            Row,
            r#"SELECT id as "id!: i64", label, created_at
               FROM api_keys
               WHERE user_id = ?
               ORDER BY created_at DESC"#,
            self.user_id,
        )
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }
}

/// Delete an API key by ID, scoped to the owning user.
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
            "DELETE FROM api_keys WHERE id = ? AND user_id = ?",
            self.id,
            self.user_id,
        )
        .execute(pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::credentials::{Get, Upsert};
    use crate::db::tests::{test_pool, test_user};

    #[tokio::test]
    async fn api_key_and_credentials_crud() {
        let pool = test_pool().await;
        let user_id = test_user(&pool, "alice").await;

        let api_key_id = Create {
            user_id,
            key_hash: "hash1",
            label: "default",
        }
        .execute(&pool)
        .await
        .expect("api key create failed");
        assert!(api_key_id > 0);

        let resolved_user_id = GetUserIdByHash { key_hash: "hash1" }
            .execute(&pool)
            .await
            .expect("api key lookup failed")
            .expect("api key should resolve");
        assert_eq!(resolved_user_id, user_id);

        let keys = GetByUserId { user_id }
            .execute(&pool)
            .await
            .expect("api key list failed");
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0].label, "default");

        let deleted = Delete {
            id: api_key_id,
            user_id,
        }
        .execute(&pool)
        .await
        .expect("api key delete failed");
        assert!(deleted);

        let after_delete = GetUserIdByHash { key_hash: "hash1" }
            .execute(&pool)
            .await
            .expect("api key lookup failed");
        assert!(after_delete.is_none());

        Upsert {
            user_id,
            access_token_enc: &[1, 2, 3],
            access_token_secret_enc: &[4, 5, 6],
            nonce_token: &[7; 12],
            nonce_secret: &[8; 12],
        }
        .execute(&pool)
        .await
        .expect("credentials upsert failed");

        let creds = Get { user_id }
            .execute(&pool)
            .await
            .expect("credential lookup failed")
            .expect("credentials should exist");
        assert_eq!(creds.access_token_enc, vec![1, 2, 3]);
        assert_eq!(creds.access_token_secret_enc, vec![4, 5, 6]);
        assert_eq!(creds.nonce_token, vec![7; 12]);
        assert_eq!(creds.nonce_secret, vec![8; 12]);
    }

    #[tokio::test]
    async fn api_key_delete_scoped_to_user() {
        let pool = test_pool().await;
        let alice_id = test_user(&pool, "alice").await;
        let bob_id = test_user(&pool, "bob").await;

        let api_key_id = Create {
            user_id: alice_id,
            key_hash: "alice_hash",
            label: "alice key",
        }
        .execute(&pool)
        .await
        .expect("create failed");

        let not_deleted = Delete {
            id: api_key_id,
            user_id: bob_id,
        }
        .execute(&pool)
        .await
        .expect("delete failed");
        assert!(!not_deleted);

        let still_exists = GetUserIdByHash {
            key_hash: "alice_hash",
        }
        .execute(&pool)
        .await
        .expect("lookup failed");
        assert!(still_exists.is_some());
    }
}

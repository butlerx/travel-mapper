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
}

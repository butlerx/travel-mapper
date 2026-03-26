use sqlx::SqlitePool;

/// Store a new email verification token hash for a user.
pub struct Create<'a> {
    pub user_id: i64,
    pub token_hash: &'a str,
    pub expires_at: &'a str,
}

impl Create<'_> {
    /// # Errors
    ///
    /// Returns an error if the insert fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<i64, sqlx::Error> {
        let result = sqlx::query!(
            "INSERT INTO email_verifications (user_id, token_hash, expires_at) VALUES (?, ?, ?)",
            self.user_id,
            self.token_hash,
            self.expires_at,
        )
        .execute(pool)
        .await?;
        Ok(result.last_insert_rowid())
    }
}

/// Look up a pending verification by its token hash, returning the user ID and
/// expiry if found.
pub struct GetByTokenHash<'a> {
    pub token_hash: &'a str,
}

/// Row returned by [`GetByTokenHash`].
pub struct VerificationRow {
    pub id: i64,
    pub user_id: i64,
    pub expires_at: String,
}

impl GetByTokenHash<'_> {
    /// # Errors
    ///
    /// Returns an error if the query fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<Option<VerificationRow>, sqlx::Error> {
        sqlx::query_as!(
            VerificationRow,
            r#"SELECT
                id as "id!: i64",
                user_id as "user_id!: i64",
                expires_at
            FROM email_verifications
            WHERE token_hash = ?"#,
            self.token_hash,
        )
        .fetch_optional(pool)
        .await
    }
}

/// Delete all verification tokens for a user (e.g. after successful verification
/// or email change).
pub struct DeleteByUserId {
    pub user_id: i64,
}

impl DeleteByUserId {
    /// # Errors
    ///
    /// Returns an error if the delete fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<u64, sqlx::Error> {
        let result = sqlx::query!(
            "DELETE FROM email_verifications WHERE user_id = ?",
            self.user_id,
        )
        .execute(pool)
        .await?;
        Ok(result.rows_affected())
    }
}

/// Purge expired verification tokens.
pub struct DeleteExpired;

impl DeleteExpired {
    /// # Errors
    ///
    /// Returns an error if the delete fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<u64, sqlx::Error> {
        let result =
            sqlx::query!("DELETE FROM email_verifications WHERE expires_at < datetime('now')")
                .execute(pool)
                .await?;
        Ok(result.rows_affected())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::tests::{test_pool, test_user};

    #[tokio::test]
    async fn verification_token_crud() {
        let pool = test_pool().await;
        let user_id = test_user(&pool, "alice").await;

        let token_id = Create {
            user_id,
            token_hash: "hash_abc",
            expires_at: "2999-01-01 00:00:00",
        }
        .execute(&pool)
        .await
        .expect("create failed");
        assert!(token_id > 0);

        let row = GetByTokenHash {
            token_hash: "hash_abc",
        }
        .execute(&pool)
        .await
        .expect("lookup failed")
        .expect("should find token");
        assert_eq!(row.user_id, user_id);
        assert_eq!(row.expires_at, "2999-01-01 00:00:00");

        let unknown = GetByTokenHash {
            token_hash: "nonexistent",
        }
        .execute(&pool)
        .await
        .expect("lookup failed");
        assert!(unknown.is_none());

        let deleted = DeleteByUserId { user_id }
            .execute(&pool)
            .await
            .expect("delete failed");
        assert_eq!(deleted, 1);

        let after = GetByTokenHash {
            token_hash: "hash_abc",
        }
        .execute(&pool)
        .await
        .expect("lookup failed");
        assert!(after.is_none());
    }

    #[tokio::test]
    async fn expired_tokens_are_purged() {
        let pool = test_pool().await;
        let user_id = test_user(&pool, "alice").await;

        Create {
            user_id,
            token_hash: "expired_hash",
            expires_at: "2000-01-01 00:00:00",
        }
        .execute(&pool)
        .await
        .expect("create failed");

        Create {
            user_id,
            token_hash: "valid_hash",
            expires_at: "2999-01-01 00:00:00",
        }
        .execute(&pool)
        .await
        .expect("create failed");

        let purged = DeleteExpired.execute(&pool).await.expect("purge failed");
        assert_eq!(purged, 1);

        let still_valid = GetByTokenHash {
            token_hash: "valid_hash",
        }
        .execute(&pool)
        .await
        .expect("lookup failed");
        assert!(still_valid.is_some());

        let gone = GetByTokenHash {
            token_hash: "expired_hash",
        }
        .execute(&pool)
        .await
        .expect("lookup failed");
        assert!(gone.is_none());
    }
}

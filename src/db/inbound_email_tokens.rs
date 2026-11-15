use sqlx::SqlitePool;

/// Store a new inbound email token for a user.
pub struct Create<'a> {
    pub user_id: i64,
    pub token_hash: &'a str,
    pub label: &'a str,
}

impl Create<'_> {
    /// # Errors
    ///
    /// Returns an error if the insert fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<i64, sqlx::Error> {
        let result = sqlx::query!(
            "INSERT INTO inbound_email_tokens (user_id, token_hash, label) VALUES (?, ?, ?)",
            self.user_id,
            self.token_hash,
            self.label,
        )
        .execute(pool)
        .await?;
        Ok(result.last_insert_rowid())
    }
}

/// Resolve an inbound email token hash to its owning user ID.
pub struct GetUserIdByHash<'a> {
    pub token_hash: &'a str,
}

impl GetUserIdByHash<'_> {
    /// # Errors
    ///
    /// Returns an error if the query fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<Option<i64>, sqlx::Error> {
        let row = sqlx::query_scalar!(
            "SELECT user_id FROM inbound_email_tokens WHERE token_hash = ?",
            self.token_hash
        )
        .fetch_optional(pool)
        .await?;
        Ok(row)
    }
}

/// A single inbound email token row for listing in the settings UI.
pub struct Row {
    pub id: i64,
    pub token_hash: String,
    pub label: String,
    pub created_at: String,
}

/// List all inbound email tokens for a user.
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
            r#"SELECT id as "id!: i64", token_hash, label, created_at
               FROM inbound_email_tokens
               WHERE user_id = ?
               ORDER BY created_at DESC"#,
            self.user_id,
        )
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }
}

/// Delete an inbound email token by ID, scoped to the owning user.
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
            "DELETE FROM inbound_email_tokens WHERE id = ? AND user_id = ?",
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
    use crate::db::tests::{test_pool, test_user};

    #[tokio::test]
    async fn inbound_email_token_crud() {
        let pool = test_pool().await;
        let user_id = test_user(&pool, "alice").await;

        let token_id = Create {
            user_id,
            token_hash: "email_hash_1",
            label: "email import",
        }
        .execute(&pool)
        .await
        .expect("inbound email token create failed");
        assert!(token_id > 0);

        let resolved = GetUserIdByHash {
            token_hash: "email_hash_1",
        }
        .execute(&pool)
        .await
        .expect("inbound email token lookup failed")
        .expect("inbound email token should resolve");
        assert_eq!(resolved, user_id);

        let unknown = GetUserIdByHash {
            token_hash: "nonexistent",
        }
        .execute(&pool)
        .await
        .expect("inbound email token lookup failed");
        assert!(unknown.is_none());

        let tokens = GetByUserId { user_id }
            .execute(&pool)
            .await
            .expect("inbound email token list failed");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].label, "email import");
        assert_eq!(tokens[0].token_hash, "email_hash_1");

        let deleted = Delete {
            id: token_id,
            user_id,
        }
        .execute(&pool)
        .await
        .expect("inbound email token delete failed");
        assert!(deleted);

        let after_delete = GetUserIdByHash {
            token_hash: "email_hash_1",
        }
        .execute(&pool)
        .await
        .expect("inbound email token lookup failed");
        assert!(after_delete.is_none());
    }

    #[tokio::test]
    async fn inbound_email_token_delete_scoped_to_user() {
        let pool = test_pool().await;
        let alice_id = test_user(&pool, "alice").await;
        let bob_id = test_user(&pool, "bob").await;

        let token_id = Create {
            user_id: alice_id,
            token_hash: "alice_email_hash",
            label: "alice import",
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

        let still_exists = GetUserIdByHash {
            token_hash: "alice_email_hash",
        }
        .execute(&pool)
        .await
        .expect("lookup failed");
        assert!(still_exists.is_some());
    }
}

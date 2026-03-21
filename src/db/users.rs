use sqlx::SqlitePool;

/// Insert a new user with a pre-hashed password.
pub struct Create<'a> {
    pub username: &'a str,
    pub password_hash: &'a str,
}

impl Create<'_> {
    /// # Errors
    ///
    /// Returns an error if the insert fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<i64, sqlx::Error> {
        let result = sqlx::query!(
            "INSERT INTO users (username, password_hash) VALUES (?, ?)",
            self.username,
            self.password_hash,
        )
        .execute(pool)
        .await?;
        Ok(result.last_insert_rowid())
    }
}

/// A row from the `users` table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
}

/// Look up a user by their unique username.
pub struct GetByUsername<'a> {
    pub username: &'a str,
}

impl GetByUsername<'_> {
    /// # Errors
    ///
    /// Returns an error if the query fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<Option<Row>, sqlx::Error> {
        sqlx::query_as!(
            Row,
            r#"SELECT id as "id!: i64", username, password_hash FROM users WHERE username = ?"#,
            self.username,
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
    async fn user_and_session_crud() {
        let pool = test_pool().await;
        let user_id = Create {
            username: "alice",
            password_hash: "hash",
        }
        .execute(&pool)
        .await
        .expect("user create failed");

        let user = GetByUsername { username: "alice" }
            .execute(&pool)
            .await
            .expect("user lookup failed")
            .expect("user should exist");
        assert_eq!(user.id, user_id);

        crate::db::sessions::Create {
            token: "session-token",
            user_id,
            expires_at: "2999-01-01 00:00:00",
        }
        .execute(&pool)
        .await
        .expect("session create failed");

        let session = (crate::db::sessions::Get {
            token: "session-token",
        })
        .execute(&pool)
        .await
        .expect("session lookup failed")
        .expect("session should exist");
        assert_eq!(session.user_id, user_id);

        (crate::db::sessions::Delete {
            token: "session-token",
        })
        .execute(&pool)
        .await
        .expect("session delete failed");
        let deleted = (crate::db::sessions::Get {
            token: "session-token",
        })
        .execute(&pool)
        .await
        .expect("session lookup after delete failed");
        assert!(deleted.is_none());
    }
}

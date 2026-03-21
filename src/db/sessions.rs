use sqlx::SqlitePool;

/// A row from the `sessions` table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub user_id: i64,
    pub expires_at: String,
}

/// Insert a new browser session with a token and expiry.
pub struct Create<'a> {
    pub token: &'a str,
    pub user_id: i64,
    pub expires_at: &'a str,
}

impl Create<'_> {
    /// # Errors
    ///
    /// Returns an error if the insert fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO sessions (token, user_id, expires_at) VALUES (?, ?, ?)",
            self.token,
            self.user_id,
            self.expires_at,
        )
        .execute(pool)
        .await?;
        Ok(())
    }
}

/// Fetch a session by its token, returning the user and expiry.
pub struct Get<'a> {
    pub token: &'a str,
}

impl Get<'_> {
    /// # Errors
    ///
    /// Returns an error if the query fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<Option<Row>, sqlx::Error> {
        sqlx::query_as!(
            Row,
            "SELECT user_id, expires_at FROM sessions WHERE token = ?",
            self.token,
        )
        .fetch_optional(pool)
        .await
    }
}

/// Delete a session by token, effectively logging the user out.
pub struct Delete<'a> {
    pub token: &'a str,
}

impl Delete<'_> {
    /// # Errors
    ///
    /// Returns an error if the delete fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM sessions WHERE token = ?", self.token)
            .execute(pool)
            .await?;
        Ok(())
    }
}

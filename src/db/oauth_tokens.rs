use sqlx::SqlitePool;

/// An encrypted OAuth request token row from `oauth_request_tokens`.
#[derive(Debug, Clone)]
pub struct Row {
    pub token_secret_enc: Vec<u8>,
    pub nonce: Vec<u8>,
    pub user_id: i64,
}

/// Persist an OAuth request token during the authorization flow.
pub struct Create<'a> {
    pub token: &'a str,
    pub token_secret_enc: &'a [u8],
    pub nonce: &'a [u8],
    pub user_id: i64,
}

impl Create<'_> {
    /// # Errors
    ///
    /// Returns an error if the insert fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r"INSERT OR REPLACE INTO oauth_request_tokens (token, token_secret_enc, nonce, user_id)
               VALUES (?, ?, ?, ?)",
            self.token,
            self.token_secret_enc,
            self.nonce,
            self.user_id,
        )
        .execute(pool)
        .await?;
        Ok(())
    }
}

/// Retrieve an OAuth request token by its public token string.
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
            r"SELECT token_secret_enc, nonce, user_id FROM oauth_request_tokens WHERE token = ?",
            self.token,
        )
        .fetch_optional(pool)
        .await
    }
}

/// Remove an OAuth request token after the flow completes or is abandoned.
pub struct Delete<'a> {
    pub token: &'a str,
}

impl Delete<'_> {
    /// # Errors
    ///
    /// Returns an error if the delete fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "DELETE FROM oauth_request_tokens WHERE token = ?",
            self.token
        )
        .execute(pool)
        .await?;
        Ok(())
    }
}

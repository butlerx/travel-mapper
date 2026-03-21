use sqlx::SqlitePool;

/// Encrypted `TripIt` OAuth credential pair with per-field AES-256-GCM nonces.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub access_token_enc: Vec<u8>,
    pub access_token_secret_enc: Vec<u8>,
    pub nonce_token: Vec<u8>,
    pub nonce_secret: Vec<u8>,
}

/// Insert or update encrypted `TripIt` credentials for a user.
pub struct Upsert<'a> {
    pub user_id: i64,
    pub access_token_enc: &'a [u8],
    pub access_token_secret_enc: &'a [u8],
    pub nonce_token: &'a [u8],
    pub nonce_secret: &'a [u8],
}

impl Upsert<'_> {
    /// # Errors
    ///
    /// Returns an error if the write fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r"INSERT INTO user_tripit_credentials (
                   user_id,
                   access_token_enc,
                   access_token_secret_enc,
                   nonce_token,
                   nonce_secret
               ) VALUES (?, ?, ?, ?, ?)
               ON CONFLICT(user_id) DO UPDATE SET
                   access_token_enc = excluded.access_token_enc,
                   access_token_secret_enc = excluded.access_token_secret_enc,
                   nonce_token = excluded.nonce_token,
                   nonce_secret = excluded.nonce_secret,
                   updated_at = datetime('now')",
            self.user_id,
            self.access_token_enc,
            self.access_token_secret_enc,
            self.nonce_token,
            self.nonce_secret,
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}

/// Fetch the encrypted `TripIt` credentials for a user.
pub struct Get {
    pub user_id: i64,
}

impl Get {
    /// # Errors
    ///
    /// Returns an error if the query fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<Option<Row>, sqlx::Error> {
        sqlx::query_as!(
            Row,
            r"SELECT
                   access_token_enc,
                   access_token_secret_enc,
                   nonce_token,
                   nonce_secret
               FROM user_tripit_credentials
               WHERE user_id = ?",
            self.user_id,
        )
        .fetch_optional(pool)
        .await
    }
}

/// Check whether a user has stored `TripIt` credentials.
pub struct Has {
    pub user_id: i64,
}

impl Has {
    /// # Errors
    ///
    /// Returns an error if the query fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<bool, sqlx::Error> {
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM user_tripit_credentials WHERE user_id = ?",
            self.user_id,
        )
        .fetch_one(pool)
        .await?;
        Ok(count > 0)
    }
}

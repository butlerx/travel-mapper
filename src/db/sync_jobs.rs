use sqlx::SqlitePool;

/// A row from the `sync_jobs` table representing a queued or completed sync.
#[derive(Debug, Clone)]
pub struct Row {
    pub id: i64,
    pub user_id: i64,
    pub status: String,
    pub error_message: Option<String>,
    pub created_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
}

/// Queue a new sync job with `pending` status for a user.
pub struct Enqueue {
    pub user_id: i64,
}

impl Enqueue {
    /// # Errors
    ///
    /// Returns an error if the insert fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<i64, sqlx::Error> {
        let result = sqlx::query!(
            "INSERT INTO sync_jobs (user_id, status) VALUES (?, 'pending')",
            self.user_id,
        )
        .execute(pool)
        .await?;
        Ok(result.last_insert_rowid())
    }
}

/// Check whether a user already has a pending or running sync job.
pub struct HasPendingOrRunning {
    pub user_id: i64,
}

impl HasPendingOrRunning {
    /// # Errors
    ///
    /// Returns an error if the query fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<bool, sqlx::Error> {
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM sync_jobs WHERE user_id = ? AND status IN ('pending', 'running')",
            self.user_id,
        )
        .fetch_one(pool)
        .await?;
        Ok(count > 0)
    }
}

/// Atomically claim the oldest pending job, marking it as `running`.
pub struct ClaimNext;

impl ClaimNext {
    /// # Errors
    ///
    /// Returns an error if the update fails or no pending job exists.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<Option<Row>, sqlx::Error> {
        let row = sqlx::query_as!(
            Row,
            r#"UPDATE sync_jobs
               SET status = 'running', started_at = datetime('now')
               WHERE id = (
                   SELECT id FROM sync_jobs WHERE status = 'pending' ORDER BY created_at ASC LIMIT 1
               )
               RETURNING id AS "id!: i64", user_id, status, error_message, created_at, started_at, completed_at"#,
        )
        .fetch_optional(pool)
        .await?;
        Ok(row)
    }
}

/// Mark a running job as `completed`.
pub struct Complete {
    pub job_id: i64,
}

impl Complete {
    /// # Errors
    ///
    /// Returns an error if the update fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE sync_jobs SET status = 'completed', completed_at = datetime('now') WHERE id = ?",
            self.job_id,
        )
        .execute(pool)
        .await?;
        Ok(())
    }
}

/// Mark a running job as `failed` with an error message.
pub struct Fail<'a> {
    pub job_id: i64,
    pub error_message: &'a str,
}

impl Fail<'_> {
    /// # Errors
    ///
    /// Returns an error if the update fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE sync_jobs SET status = 'failed', error_message = ?, completed_at = datetime('now') WHERE id = ?",
            self.error_message,
            self.job_id,
        )
        .execute(pool)
        .await?;
        Ok(())
    }
}

/// Reset all `running` jobs back to `pending` (used on worker startup).
pub struct ResetStaleRunning;

impl ResetStaleRunning {
    /// # Errors
    ///
    /// Returns an error if the update fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<u64, sqlx::Error> {
        let result = sqlx::query!(
            "UPDATE sync_jobs SET status = 'pending', started_at = NULL WHERE status = 'running'",
        )
        .execute(pool)
        .await?;
        Ok(result.rows_affected())
    }
}

/// Reset all `running` sync states back to `idle` (used on worker startup).
pub struct ResetStaleSyncStates;

impl ResetStaleSyncStates {
    /// # Errors
    ///
    /// Returns an error if the update fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<u64, sqlx::Error> {
        let result = sqlx::query!(
            "UPDATE sync_state SET sync_status = 'idle' WHERE sync_status = 'running'"
        )
        .execute(pool)
        .await?;
        Ok(result.rows_affected())
    }
}

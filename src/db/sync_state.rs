use sqlx::SqlitePool;

/// Per-user sync progress: last sync time, cursor, status, and counters.
#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
pub struct Row {
    pub last_sync_at: Option<String>,
    pub last_modified_since: Option<i64>,
    pub sync_status: String,
    pub trips_fetched: i64,
    pub hops_fetched: i64,
}

/// Fetch the sync state for a user (fails if no row exists).
pub struct Get {
    pub user_id: i64,
}

impl Get {
    /// # Errors
    ///
    /// Returns an error if the query fails or no row exists.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<Row, sqlx::Error> {
        sqlx::query_as!(
            Row,
            r"SELECT
                   last_sync_at,
                   last_modified_since,
                   sync_status,
                   trips_fetched,
                   hops_fetched
               FROM sync_state
               WHERE user_id = ?",
            self.user_id,
        )
        .fetch_one(pool)
        .await
    }
}

/// Overwrite all sync state fields for a user.
pub struct Update<'a> {
    pub user_id: i64,
    pub state: &'a Row,
}

impl Update<'_> {
    /// # Errors
    ///
    /// Returns an error if updating the row fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r"UPDATE sync_state
               SET
                   last_sync_at = ?,
                   last_modified_since = ?,
                   sync_status = ?,
                   trips_fetched = ?,
                   hops_fetched = ?
               WHERE user_id = ?",
            self.state.last_sync_at,
            self.state.last_modified_since,
            self.state.sync_status,
            self.state.trips_fetched,
            self.state.hops_fetched,
            self.user_id,
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}

/// Return all user IDs that have a sync state row.
pub struct GetAllUserIds;

impl GetAllUserIds {
    /// # Errors
    ///
    /// Returns an error if the query fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<Vec<i64>, sqlx::Error> {
        let rows = sqlx::query_scalar!("SELECT user_id FROM sync_state")
            .fetch_all(pool)
            .await?;
        Ok(rows)
    }
}

/// Return the sync state for a user, creating a default row if none exists.
pub struct GetOrCreate {
    pub user_id: i64,
}

impl GetOrCreate {
    /// # Errors
    ///
    /// Returns an error if either insert-or-ignore or lookup fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<Row, sqlx::Error> {
        sqlx::query!(
            "INSERT INTO sync_state (user_id) VALUES (?) ON CONFLICT(user_id) DO NOTHING",
            self.user_id,
        )
        .execute(pool)
        .await?;

        Get {
            user_id: self.user_id,
        }
        .execute(pool)
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::tests::{test_pool, test_user};

    #[tokio::test]
    async fn get_sync_state_returns_defaults() {
        let pool = test_pool().await;
        let user_id = test_user(&pool, "alice").await;

        let state = GetOrCreate { user_id }
            .execute(&pool)
            .await
            .expect("sync state fetch failed");

        assert_eq!(state.last_sync_at, None);
        assert_eq!(state.last_modified_since, None);
        assert_eq!(state.sync_status, "idle");
        assert_eq!(state.trips_fetched, 0);
        assert_eq!(state.hops_fetched, 0);
    }

    #[tokio::test]
    async fn update_sync_state_roundtrip() {
        let pool = test_pool().await;
        let user_id = test_user(&pool, "alice").await;
        let _ = GetOrCreate { user_id }
            .execute(&pool)
            .await
            .expect("sync state setup failed");

        let updated = Row {
            last_sync_at: Some("2025-12-01T12:00:00Z".to_string()),
            last_modified_since: Some(123_456),
            sync_status: "running".to_string(),
            trips_fetched: 10,
            hops_fetched: 42,
        };

        Update {
            user_id,
            state: &updated,
        }
        .execute(&pool)
        .await
        .expect("update sync state failed");

        let state = Get { user_id }
            .execute(&pool)
            .await
            .expect("sync state fetch after update failed");
        assert_eq!(state, updated);
    }

    #[tokio::test]
    async fn get_all_user_ids_returns_synced_users() {
        let pool = test_pool().await;
        let alice = test_user(&pool, "alice").await;
        let bob = test_user(&pool, "bob").await;

        let _ = GetOrCreate { user_id: alice }
            .execute(&pool)
            .await
            .expect("alice sync state setup failed");
        let _ = GetOrCreate { user_id: bob }
            .execute(&pool)
            .await
            .expect("bob sync state setup failed");

        let mut ids = GetAllUserIds
            .execute(&pool)
            .await
            .expect("get all user ids failed");
        ids.sort_unstable();

        assert_eq!(ids, vec![alice, bob]);
    }
}

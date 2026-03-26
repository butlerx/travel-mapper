//! Database layer — connection pool setup, migrations, and per-table query objects.

/// CRUD operations on the `api_keys` table — programmatic access tokens.
pub mod api_keys;
/// Query objects for the `user_tripit_credentials` table — encrypted OAuth tokens.
pub mod credentials;
/// Query objects for the `feed_tokens` table — per-user calendar feed access tokens.
pub mod feed_tokens;
/// Query objects for the `hops` table — individual travel legs.
pub mod hops;
/// Query objects for the `oauth_request_tokens` table — temporary OAuth flow tokens.
pub mod oauth_tokens;
/// Query objects for the `sessions` table — cookie-based browser sessions.
pub mod sessions;
/// Query objects for the `share_tokens` table — per-user shareable stats access tokens.
pub mod share_tokens;
/// Query objects for the `status_enrichments` table — live/historical flight status data.
pub mod status_enrichments;
/// Query objects for the `sync_jobs` table — background sync job queue.
pub mod sync_jobs;
/// Query objects for the `sync_state` table — per-user sync progress and status.
pub mod sync_state;
/// Query objects for the `trips` table — named travel trip groups.
pub mod trips;
/// Query objects for the `users` table — registration and lookup.
pub mod users;

/// Create a configured `SQLite` pool and run all migrations.
///
/// # Errors
///
/// Returns an error if the pool cannot be created, pragmas cannot be applied,
/// or migrations fail.
pub async fn create_pool(database_url: &str) -> Result<sqlx::SqlitePool, sqlx::Error> {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;

    sqlx::query("PRAGMA journal_mode=WAL")
        .execute(&pool)
        .await?;
    sqlx::query("PRAGMA busy_timeout=5000")
        .execute(&pool)
        .await?;

    sqlx::migrate!().run(&pool).await?;
    Ok(pool)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use sqlx::SqlitePool;
    use uuid::Uuid;

    pub(crate) async fn test_pool() -> SqlitePool {
        let db_name = Uuid::new_v4();
        let url = format!("sqlite:file:{db_name}?mode=memory&cache=shared");
        create_pool(&url).await.expect("failed to create test pool")
    }

    pub(crate) async fn test_user(pool: &SqlitePool, username: &str) -> i64 {
        users::Create {
            username,
            password_hash: "hash",
        }
        .execute(pool)
        .await
        .expect("failed to create test user")
    }

    #[tokio::test]
    async fn create_pool_works_with_in_memory_sqlite() {
        let pool = test_pool().await;
        let user_id = test_user(&pool, "alice").await;
        let state = sync_state::GetOrCreate { user_id }
            .execute(&pool)
            .await
            .expect("sync_state missing");
        assert_eq!(state.sync_status, "idle");
    }
}

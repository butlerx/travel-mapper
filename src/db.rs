use crate::models::{TravelHop, TravelType};
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};

#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
pub struct SyncState {
    pub last_sync_at: Option<String>,
    pub last_modified_since: Option<i64>,
    pub sync_status: String,
    pub trips_fetched: i64,
    pub hops_fetched: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserRow {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionRow {
    pub user_id: i64,
    pub expires_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TripItCredentialRow {
    pub access_token_enc: Vec<u8>,
    pub access_token_secret_enc: Vec<u8>,
    pub nonce_token: Vec<u8>,
    pub nonce_secret: Vec<u8>,
}

#[derive(Debug, thiserror::Error)]
#[error("unknown travel_type '{value}': expected air|rail|cruise|transport")]
struct ParseTravelTypeError {
    value: String,
}

/// Create a configured `SQLite` pool and run all migrations.
///
/// # Errors
///
/// Returns an error if the pool cannot be created, pragmas cannot be applied,
/// or migrations fail.
pub async fn create_pool(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    let pool = SqlitePoolOptions::new()
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

/// # Errors
///
/// Returns an error if the insert fails.
pub async fn create_user(
    pool: &SqlitePool,
    username: &str,
    password_hash: &str,
) -> Result<i64, sqlx::Error> {
    let result = sqlx::query!(
        "INSERT INTO users (username, password_hash) VALUES (?, ?)",
        username,
        password_hash,
    )
    .execute(pool)
    .await?;
    Ok(result.last_insert_rowid())
}

/// # Errors
///
/// Returns an error if the query fails.
pub async fn get_user_by_username(
    pool: &SqlitePool,
    username: &str,
) -> Result<Option<UserRow>, sqlx::Error> {
    sqlx::query_as!(
        UserRow,
        r#"SELECT id as "id!: i64", username, password_hash FROM users WHERE username = ?"#,
        username,
    )
    .fetch_optional(pool)
    .await
}

/// # Errors
///
/// Returns an error if the insert fails.
pub async fn create_session(
    pool: &SqlitePool,
    token: &str,
    user_id: i64,
    expires_at: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO sessions (token, user_id, expires_at) VALUES (?, ?, ?)",
        token,
        user_id,
        expires_at,
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// # Errors
///
/// Returns an error if the query fails.
pub async fn get_session(
    pool: &SqlitePool,
    token: &str,
) -> Result<Option<SessionRow>, sqlx::Error> {
    sqlx::query_as!(
        SessionRow,
        "SELECT user_id, expires_at FROM sessions WHERE token = ?",
        token,
    )
    .fetch_optional(pool)
    .await
}

/// # Errors
///
/// Returns an error if the delete fails.
pub async fn delete_session(pool: &SqlitePool, token: &str) -> Result<(), sqlx::Error> {
    sqlx::query!("DELETE FROM sessions WHERE token = ?", token)
        .execute(pool)
        .await?;
    Ok(())
}

/// # Errors
///
/// Returns an error if the insert fails.
pub async fn create_api_key(
    pool: &SqlitePool,
    user_id: i64,
    key_hash: &str,
    label: &str,
) -> Result<i64, sqlx::Error> {
    let result = sqlx::query!(
        "INSERT INTO api_keys (user_id, key_hash, label) VALUES (?, ?, ?)",
        user_id,
        key_hash,
        label,
    )
    .execute(pool)
    .await?;
    Ok(result.last_insert_rowid())
}

/// # Errors
///
/// Returns an error if the query fails.
pub async fn get_user_id_by_api_key_hash(
    pool: &SqlitePool,
    key_hash: &str,
) -> Result<Option<i64>, sqlx::Error> {
    let row = sqlx::query_scalar!("SELECT user_id FROM api_keys WHERE key_hash = ?", key_hash,)
        .fetch_optional(pool)
        .await?;
    Ok(row)
}

/// # Errors
///
/// Returns an error if the write fails.
pub async fn upsert_tripit_credentials(
    pool: &SqlitePool,
    user_id: i64,
    access_token_enc: &[u8],
    access_token_secret_enc: &[u8],
    nonce_token: &[u8],
    nonce_secret: &[u8],
) -> Result<(), sqlx::Error> {
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
        user_id,
        access_token_enc,
        access_token_secret_enc,
        nonce_token,
        nonce_secret,
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// # Errors
///
/// Returns an error if the query fails.
pub async fn get_tripit_credentials(
    pool: &SqlitePool,
    user_id: i64,
) -> Result<Option<TripItCredentialRow>, sqlx::Error> {
    sqlx::query_as!(
        TripItCredentialRow,
        r"SELECT
               access_token_enc,
               access_token_secret_enc,
               nonce_token,
               nonce_secret
           FROM user_tripit_credentials
           WHERE user_id = ?",
        user_id,
    )
    .fetch_optional(pool)
    .await
}

/// # Errors
///
/// Returns an error if either insert-or-ignore or lookup fails.
pub async fn get_or_create_sync_state(
    pool: &SqlitePool,
    user_id: i64,
) -> Result<SyncState, sqlx::Error> {
    sqlx::query!(
        "INSERT INTO sync_state (user_id) VALUES (?) ON CONFLICT(user_id) DO NOTHING",
        user_id,
    )
    .execute(pool)
    .await?;

    get_sync_state(pool, user_id).await
}

fn scoped_trip_id(user_id: i64, trip_id: &str) -> String {
    format!("{user_id}:{trip_id}")
}

/// # Errors
///
/// Returns an error if inserting any hop fails.
pub async fn insert_hops(
    pool: &SqlitePool,
    trip_id: &str,
    user_id: i64,
    hops: &[TravelHop],
) -> Result<u64, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let mut inserted = 0_u64;
    let db_trip_id = scoped_trip_id(user_id, trip_id);

    for hop in hops {
        let travel_type = hop.travel_type.to_string();
        let result = sqlx::query!(
            r"INSERT OR REPLACE INTO hops (
                   trip_id,
                   user_id,
                   travel_type,
                   origin_name,
                   origin_lat,
                   origin_lng,
                   dest_name,
                   dest_lat,
                   dest_lng,
                   start_date,
                   end_date,
                   raw_json,
                   updated_at
               ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, NULL, datetime('now'))",
            db_trip_id,
            user_id,
            travel_type,
            hop.origin_name,
            hop.origin_lat,
            hop.origin_lng,
            hop.dest_name,
            hop.dest_lat,
            hop.dest_lng,
            hop.start_date,
            hop.end_date,
        )
        .execute(&mut *tx)
        .await?;
        inserted += result.rows_affected();
    }

    tx.commit().await?;
    Ok(inserted)
}

fn parse_travel_type(value: &str) -> Result<TravelType, sqlx::Error> {
    match value {
        "air" => Ok(TravelType::Air),
        "rail" => Ok(TravelType::Rail),
        "cruise" => Ok(TravelType::Cruise),
        "transport" => Ok(TravelType::Transport),
        other => Err(sqlx::Error::Decode(Box::new(ParseTravelTypeError {
            value: other.to_string(),
        }))),
    }
}

struct HopRow {
    travel_type: String,
    origin_name: String,
    origin_lat: Option<f64>,
    origin_lng: Option<f64>,
    dest_name: String,
    dest_lat: Option<f64>,
    dest_lng: Option<f64>,
    start_date: String,
    end_date: String,
}

impl TryFrom<HopRow> for TravelHop {
    type Error = sqlx::Error;

    fn try_from(row: HopRow) -> Result<Self, Self::Error> {
        Ok(Self {
            travel_type: parse_travel_type(&row.travel_type)?,
            origin_name: row.origin_name,
            origin_lat: row.origin_lat,
            origin_lng: row.origin_lng,
            dest_name: row.dest_name,
            dest_lat: row.dest_lat,
            dest_lng: row.dest_lng,
            start_date: row.start_date,
            end_date: row.end_date,
        })
    }
}

/// # Errors
///
/// Returns an error if the query fails or a row cannot be mapped.
pub async fn get_all_hops(
    pool: &SqlitePool,
    user_id: i64,
    travel_type_filter: Option<&str>,
) -> Result<Vec<TravelHop>, sqlx::Error> {
    let rows = match travel_type_filter {
        Some(filter) => {
            sqlx::query_as!(
                HopRow,
                r"SELECT
                       travel_type,
                       origin_name,
                       origin_lat,
                       origin_lng,
                       dest_name,
                       dest_lat,
                       dest_lng,
                       start_date,
                       end_date
                   FROM hops
                   WHERE user_id = ? AND travel_type = ?
                   ORDER BY start_date ASC",
                user_id,
                filter,
            )
            .fetch_all(pool)
            .await?
        }
        None => {
            sqlx::query_as!(
                HopRow,
                r"SELECT
                       travel_type,
                       origin_name,
                       origin_lat,
                       origin_lng,
                       dest_name,
                       dest_lat,
                       dest_lng,
                       start_date,
                       end_date
                   FROM hops
                   WHERE user_id = ?
                   ORDER BY start_date ASC",
                user_id,
            )
            .fetch_all(pool)
            .await?
        }
    };

    rows.into_iter().map(TravelHop::try_from).collect()
}

/// # Errors
///
/// Returns an error if the delete operation fails.
pub async fn delete_hops_for_trip(
    pool: &SqlitePool,
    trip_id: &str,
    user_id: i64,
) -> Result<u64, sqlx::Error> {
    let db_trip_id = scoped_trip_id(user_id, trip_id);
    let result = sqlx::query!(
        "DELETE FROM hops WHERE trip_id = ? AND user_id = ?",
        db_trip_id,
        user_id,
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}

/// # Errors
///
/// Returns an error if the query fails or no row exists.
pub async fn get_sync_state(pool: &SqlitePool, user_id: i64) -> Result<SyncState, sqlx::Error> {
    sqlx::query_as!(
        SyncState,
        r"SELECT
               last_sync_at,
               last_modified_since,
               sync_status,
               trips_fetched,
               hops_fetched
           FROM sync_state
           WHERE user_id = ?",
        user_id,
    )
    .fetch_one(pool)
    .await
}

/// # Errors
///
/// Returns an error if updating the row fails.
pub async fn update_sync_state(
    pool: &SqlitePool,
    user_id: i64,
    state: &SyncState,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r"UPDATE sync_state
           SET
               last_sync_at = ?,
               last_modified_since = ?,
               sync_status = ?,
               trips_fetched = ?,
               hops_fetched = ?
           WHERE user_id = ?",
        state.last_sync_at,
        state.last_modified_since,
        state.sync_status,
        state.trips_fetched,
        state.hops_fetched,
        user_id,
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// # Errors
///
/// Returns an error if the insert fails.
pub async fn store_oauth_request_token(
    pool: &SqlitePool,
    token: &str,
    token_secret_enc: &[u8],
    nonce: &[u8],
    user_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r"INSERT OR REPLACE INTO oauth_request_tokens (token, token_secret_enc, nonce, user_id)
           VALUES (?, ?, ?, ?)",
        token,
        token_secret_enc,
        nonce,
        user_id,
    )
    .execute(pool)
    .await?;
    Ok(())
}

#[derive(Debug, Clone)]
pub struct OAuthRequestTokenRow {
    pub token_secret_enc: Vec<u8>,
    pub nonce: Vec<u8>,
    pub user_id: i64,
}

/// # Errors
///
/// Returns an error if the query fails.
pub async fn get_oauth_request_token(
    pool: &SqlitePool,
    token: &str,
) -> Result<Option<OAuthRequestTokenRow>, sqlx::Error> {
    sqlx::query_as!(
        OAuthRequestTokenRow,
        r"SELECT token_secret_enc, nonce, user_id FROM oauth_request_tokens WHERE token = ?",
        token,
    )
    .fetch_optional(pool)
    .await
}

/// # Errors
///
/// Returns an error if the delete fails.
pub async fn delete_oauth_request_token(pool: &SqlitePool, token: &str) -> Result<(), sqlx::Error> {
    sqlx::query!("DELETE FROM oauth_request_tokens WHERE token = ?", token)
        .execute(pool)
        .await?;
    Ok(())
}

/// # Errors
///
/// Returns an error if the query fails.
pub async fn has_tripit_credentials(pool: &SqlitePool, user_id: i64) -> Result<bool, sqlx::Error> {
    let count = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM user_tripit_credentials WHERE user_id = ?",
        user_id,
    )
    .fetch_one(pool)
    .await?;
    Ok(count > 0)
}

#[derive(Debug, Clone)]
pub struct SyncJobRow {
    pub id: i64,
    pub user_id: i64,
    pub status: String,
    pub error_message: Option<String>,
    pub created_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
}

/// # Errors
///
/// Returns an error if the insert fails.
pub async fn enqueue_sync_job(pool: &SqlitePool, user_id: i64) -> Result<i64, sqlx::Error> {
    let result = sqlx::query!(
        "INSERT INTO sync_jobs (user_id, status) VALUES (?, 'pending')",
        user_id,
    )
    .execute(pool)
    .await?;
    Ok(result.last_insert_rowid())
}

/// # Errors
///
/// Returns an error if the query fails.
pub async fn has_pending_or_running_sync_job(
    pool: &SqlitePool,
    user_id: i64,
) -> Result<bool, sqlx::Error> {
    let count = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM sync_jobs WHERE user_id = ? AND status IN ('pending', 'running')",
        user_id,
    )
    .fetch_one(pool)
    .await?;
    Ok(count > 0)
}

/// # Errors
///
/// Returns an error if the update fails or no pending job exists.
pub async fn claim_next_sync_job(pool: &SqlitePool) -> Result<Option<SyncJobRow>, sqlx::Error> {
    let row = sqlx::query_as!(
        SyncJobRow,
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

/// # Errors
///
/// Returns an error if the update fails.
pub async fn complete_sync_job(pool: &SqlitePool, job_id: i64) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE sync_jobs SET status = 'completed', completed_at = datetime('now') WHERE id = ?",
        job_id,
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// # Errors
///
/// Returns an error if the update fails.
pub async fn fail_sync_job(
    pool: &SqlitePool,
    job_id: i64,
    error_message: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE sync_jobs SET status = 'failed', error_message = ?, completed_at = datetime('now') WHERE id = ?",
        error_message,
        job_id,
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// # Errors
///
/// Returns an error if the update fails.
pub async fn reset_stale_running_jobs(pool: &SqlitePool) -> Result<u64, sqlx::Error> {
    let result = sqlx::query!(
        "UPDATE sync_jobs SET status = 'pending', started_at = NULL WHERE status = 'running'",
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}

/// # Errors
///
/// Returns an error if the update fails.
pub async fn reset_stale_sync_states(pool: &SqlitePool) -> Result<u64, sqlx::Error> {
    let result =
        sqlx::query!("UPDATE sync_state SET sync_status = 'idle' WHERE sync_status = 'running'",)
            .execute(pool)
            .await?;
    Ok(result.rows_affected())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{TravelHop, TravelType};
    use uuid::Uuid;

    fn sample_hop(
        travel_type: TravelType,
        origin: &str,
        dest: &str,
        start_date: &str,
        end_date: &str,
    ) -> TravelHop {
        TravelHop {
            travel_type,
            origin_name: origin.to_string(),
            origin_lat: Some(1.0),
            origin_lng: Some(2.0),
            dest_name: dest.to_string(),
            dest_lat: Some(3.0),
            dest_lng: Some(4.0),
            start_date: start_date.to_string(),
            end_date: end_date.to_string(),
        }
    }

    async fn test_pool() -> SqlitePool {
        let db_name = Uuid::new_v4();
        let url = format!("sqlite:file:{db_name}?mode=memory&cache=shared");
        create_pool(&url).await.expect("failed to create test pool")
    }

    async fn test_user(pool: &SqlitePool, username: &str) -> i64 {
        create_user(pool, username, "hash")
            .await
            .expect("failed to create test user")
    }

    #[tokio::test]
    async fn create_pool_works_with_in_memory_sqlite() {
        let pool = test_pool().await;
        let user_id = test_user(&pool, "alice").await;
        let state = get_or_create_sync_state(&pool, user_id)
            .await
            .expect("sync_state missing");
        assert_eq!(state.sync_status, "idle");
    }

    #[tokio::test]
    async fn insert_and_get_all_hops_roundtrip() {
        let pool = test_pool().await;
        let user_id = test_user(&pool, "alice").await;
        let hops = vec![
            sample_hop(
                TravelType::Rail,
                "Paris",
                "London",
                "2024-01-01",
                "2024-01-01",
            ),
            sample_hop(TravelType::Air, "LHR", "JFK", "2024-02-01", "2024-02-01"),
        ];

        let inserted = insert_hops(&pool, "trip-1", user_id, &hops)
            .await
            .expect("insert failed");
        assert_eq!(inserted, 2);

        let fetched = get_all_hops(&pool, user_id, None)
            .await
            .expect("fetch failed");
        assert_eq!(fetched.len(), 2);
        assert_eq!(fetched[0].start_date, "2024-01-01");
        assert_eq!(fetched[1].start_date, "2024-02-01");
    }

    #[tokio::test]
    async fn get_all_hops_filters_by_travel_type() {
        let pool = test_pool().await;
        let user_id = test_user(&pool, "alice").await;
        let hops = vec![
            sample_hop(
                TravelType::Rail,
                "Paris",
                "London",
                "2024-01-01",
                "2024-01-01",
            ),
            sample_hop(TravelType::Air, "LHR", "JFK", "2024-02-01", "2024-02-01"),
        ];

        insert_hops(&pool, "trip-1", user_id, &hops)
            .await
            .expect("insert failed");

        let filtered = get_all_hops(&pool, user_id, Some("rail"))
            .await
            .expect("filter failed");
        assert_eq!(filtered.len(), 1);
        assert!(matches!(filtered[0].travel_type, TravelType::Rail));
    }

    #[tokio::test]
    async fn get_all_hops_isolated_per_user() {
        let pool = test_pool().await;
        let alice_id = test_user(&pool, "alice").await;
        let bob_id = test_user(&pool, "bob").await;

        insert_hops(
            &pool,
            "trip-1",
            alice_id,
            &[sample_hop(
                TravelType::Rail,
                "Paris",
                "London",
                "2024-01-01",
                "2024-01-01",
            )],
        )
        .await
        .expect("insert alice failed");

        insert_hops(
            &pool,
            "trip-1",
            bob_id,
            &[sample_hop(
                TravelType::Air,
                "LHR",
                "JFK",
                "2024-02-01",
                "2024-02-01",
            )],
        )
        .await
        .expect("insert bob failed");

        let alice_hops = get_all_hops(&pool, alice_id, None)
            .await
            .expect("fetch alice failed");
        let bob_hops = get_all_hops(&pool, bob_id, None)
            .await
            .expect("fetch bob failed");

        assert_eq!(alice_hops.len(), 1);
        assert_eq!(bob_hops.len(), 1);
        assert_eq!(alice_hops[0].origin_name, "Paris");
        assert_eq!(bob_hops[0].origin_name, "LHR");
    }

    #[tokio::test]
    async fn delete_hops_for_trip_removes_only_target_trip() {
        let pool = test_pool().await;
        let user_id = test_user(&pool, "alice").await;

        let trip_1_hops = vec![sample_hop(
            TravelType::Rail,
            "Paris",
            "London",
            "2024-01-01",
            "2024-01-01",
        )];
        let trip_2_hops = vec![sample_hop(
            TravelType::Air,
            "LHR",
            "JFK",
            "2024-02-01",
            "2024-02-01",
        )];

        insert_hops(&pool, "trip-1", user_id, &trip_1_hops)
            .await
            .expect("insert trip-1 failed");
        insert_hops(&pool, "trip-2", user_id, &trip_2_hops)
            .await
            .expect("insert trip-2 failed");

        let deleted = delete_hops_for_trip(&pool, "trip-1", user_id)
            .await
            .expect("delete failed");
        assert_eq!(deleted, 1);

        let remaining = get_all_hops(&pool, user_id, None)
            .await
            .expect("fetch failed");
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].origin_name, "LHR");
    }

    #[tokio::test]
    async fn get_sync_state_returns_defaults() {
        let pool = test_pool().await;
        let user_id = test_user(&pool, "alice").await;

        let state = get_or_create_sync_state(&pool, user_id)
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
        let _ = get_or_create_sync_state(&pool, user_id)
            .await
            .expect("sync state setup failed");

        let updated = SyncState {
            last_sync_at: Some("2025-12-01T12:00:00Z".to_string()),
            last_modified_since: Some(123_456),
            sync_status: "running".to_string(),
            trips_fetched: 10,
            hops_fetched: 42,
        };

        update_sync_state(&pool, user_id, &updated)
            .await
            .expect("update sync state failed");

        let state = get_sync_state(&pool, user_id)
            .await
            .expect("sync state fetch after update failed");
        assert_eq!(state, updated);
    }

    #[tokio::test]
    async fn insert_hops_duplicate_unique_key_replaces_row() {
        let pool = test_pool().await;
        let user_id = test_user(&pool, "alice").await;

        let first = sample_hop(TravelType::Air, "LHR", "JFK", "2024-03-01", "2024-03-01");
        let mut replacement = first.clone();
        replacement.origin_lat = Some(99.9);

        insert_hops(&pool, "trip-1", user_id, &[first])
            .await
            .expect("first insert failed");
        insert_hops(&pool, "trip-1", user_id, &[replacement])
            .await
            .expect("replacement insert failed");

        let fetched = get_all_hops(&pool, user_id, Some("air"))
            .await
            .expect("fetch failed");
        assert_eq!(fetched.len(), 1);
        assert_eq!(fetched[0].origin_lat, Some(99.9));
    }

    #[tokio::test]
    async fn user_and_session_crud() {
        let pool = test_pool().await;
        let user_id = create_user(&pool, "alice", "hash")
            .await
            .expect("user create failed");

        let user = get_user_by_username(&pool, "alice")
            .await
            .expect("user lookup failed")
            .expect("user should exist");
        assert_eq!(user.id, user_id);

        create_session(&pool, "session-token", user_id, "2999-01-01 00:00:00")
            .await
            .expect("session create failed");

        let session = get_session(&pool, "session-token")
            .await
            .expect("session lookup failed")
            .expect("session should exist");
        assert_eq!(session.user_id, user_id);

        delete_session(&pool, "session-token")
            .await
            .expect("session delete failed");
        let deleted = get_session(&pool, "session-token")
            .await
            .expect("session lookup after delete failed");
        assert!(deleted.is_none());
    }

    #[tokio::test]
    async fn api_key_and_credentials_crud() {
        let pool = test_pool().await;
        let user_id = test_user(&pool, "alice").await;

        let api_key_id = create_api_key(&pool, user_id, "hash1", "default")
            .await
            .expect("api key create failed");
        assert!(api_key_id > 0);

        let resolved_user_id = get_user_id_by_api_key_hash(&pool, "hash1")
            .await
            .expect("api key lookup failed")
            .expect("api key should resolve");
        assert_eq!(resolved_user_id, user_id);

        upsert_tripit_credentials(&pool, user_id, &[1, 2, 3], &[4, 5, 6], &[7; 12], &[8; 12])
            .await
            .expect("credentials upsert failed");

        let creds = get_tripit_credentials(&pool, user_id)
            .await
            .expect("credential lookup failed")
            .expect("credentials should exist");
        assert_eq!(creds.access_token_enc, vec![1, 2, 3]);
        assert_eq!(creds.access_token_secret_enc, vec![4, 5, 6]);
        assert_eq!(creds.nonce_token, vec![7; 12]);
        assert_eq!(creds.nonce_secret, vec![8; 12]);
    }
}

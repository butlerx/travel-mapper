use crate::{
    auth::decrypt_token,
    db,
    tripit::{self, FetchError, TripItApi, TripItAuth, TripItClient},
};
use schemars::JsonSchema;
use serde::Serialize;
use serde_json::Value;
use sqlx::SqlitePool;
use std::{collections::HashSet, time::Instant};
use tokio::sync::watch;

const FULL_SYNC_TRIP_ID: &str = "full-sync";

#[derive(Debug, Serialize, JsonSchema)]
pub struct SyncResult {
    pub trips_fetched: u64,
    pub hops_fetched: u64,
    pub duration_ms: u64,
}

#[derive(Debug, thiserror::Error)]
pub enum SyncError {
    #[error("TripIt API error: {0}")]
    Fetch(#[from] FetchError),

    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("count overflow for {0}")]
    CountOverflow(&'static str),
}

/// Run a full sync from `TripIt` for a specific user.
///
/// # Errors
///
/// Returns [`SyncError::Fetch`] on `TripIt` API failures, [`SyncError::Database`]
/// on database errors, or [`SyncError::CountOverflow`] if counts exceed `i64`.
pub async fn sync_all(
    api: &dyn TripItApi,
    pool: &SqlitePool,
    user_id: i64,
) -> Result<SyncResult, SyncError> {
    let started_at = Instant::now();
    let mut state = db::get_or_create_sync_state(pool, user_id).await?;

    tracing::info!(user_id, "sync started");

    state.sync_status = "running".to_string();
    db::update_sync_state(pool, user_id, &state).await?;

    let sync_result = async {
        let trips_fetched = fetch_unique_trip_count(api).await?;
        tracing::info!(user_id, trips_fetched, "fetched trip count");

        let hops = tripit::fetch_all_hops(api).await?;
        tracing::info!(user_id, hops = hops.len(), "fetched hops");

        db::delete_hops_for_trip(pool, FULL_SYNC_TRIP_ID, user_id).await?;
        let hops_fetched = db::insert_hops(pool, FULL_SYNC_TRIP_ID, user_id, &hops).await?;

        let now = sqlx::query_scalar!("SELECT datetime('now')")
            .fetch_one(pool)
            .await?;

        state.last_sync_at = now;
        state.sync_status = "idle".to_string();
        state.trips_fetched =
            i64::try_from(trips_fetched).map_err(|_| SyncError::CountOverflow("trips_fetched"))?;
        state.hops_fetched =
            i64::try_from(hops_fetched).map_err(|_| SyncError::CountOverflow("hops_fetched"))?;
        db::update_sync_state(pool, user_id, &state).await?;

        Ok(SyncResult {
            trips_fetched,
            hops_fetched,
            duration_ms: u64::try_from(started_at.elapsed().as_millis()).unwrap_or(u64::MAX),
        })
    }
    .await;

    match &sync_result {
        Ok(result) => tracing::info!(
            user_id,
            trips = result.trips_fetched,
            hops = result.hops_fetched,
            duration_ms = result.duration_ms,
            "sync completed",
        ),
        Err(err) => tracing::error!(user_id, error = %err, "sync failed"),
    }

    if sync_result.is_err() {
        state.sync_status = "idle".to_string();
        let _ = db::update_sync_state(pool, user_id, &state).await;
    }

    sync_result
}

pub struct SyncWorkerConfig {
    pub pool: SqlitePool,
    pub encryption_key: [u8; 32],
    pub consumer_key: String,
    pub consumer_secret: String,
    pub poll_interval: std::time::Duration,
}

/// # Errors
///
/// Returns an error if resetting stale jobs fails on startup.
pub async fn run_sync_worker(
    config: SyncWorkerConfig,
    mut shutdown: watch::Receiver<bool>,
) -> Result<(), sqlx::Error> {
    let reset_jobs = db::reset_stale_running_jobs(&config.pool).await?;
    let reset_states = db::reset_stale_sync_states(&config.pool).await?;
    if reset_jobs > 0 || reset_states > 0 {
        tracing::info!(
            reset_jobs,
            reset_states,
            "reset stale state from previous run"
        );
    }

    loop {
        tokio::select! {
            _ = shutdown.changed() => {
                tracing::info!("sync worker shutting down");
                return Ok(());
            }
            () = tokio::time::sleep(config.poll_interval) => {}
        }

        match db::claim_next_sync_job(&config.pool).await {
            Ok(Some(job)) => {
                tracing::info!(job_id = job.id, user_id = job.user_id, "claimed sync job");
                process_sync_job(&config, &job).await;
            }
            Ok(None) => {}
            Err(err) => {
                tracing::error!(error = %err, "failed to claim sync job");
            }
        }
    }
}

async fn process_sync_job(config: &SyncWorkerConfig, job: &db::SyncJobRow) {
    let result = build_client_and_sync(config, job).await;

    match result {
        Ok(ref sync_result) => {
            tracing::info!(
                job_id = job.id,
                user_id = job.user_id,
                trips = sync_result.trips_fetched,
                hops = sync_result.hops_fetched,
                duration_ms = sync_result.duration_ms,
                "sync job completed",
            );
            if let Err(err) = db::complete_sync_job(&config.pool, job.id).await {
                tracing::error!(job_id = job.id, error = %err, "failed to mark job completed");
            }
        }
        Err(err) => {
            tracing::error!(job_id = job.id, user_id = job.user_id, error = %err, "sync job failed");
            if let Err(db_err) = db::fail_sync_job(&config.pool, job.id, &err.to_string()).await {
                tracing::error!(job_id = job.id, error = %db_err, "failed to mark job failed");
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
enum WorkerError {
    #[error("no TripIt credentials for user {0}")]
    MissingCredentials(i64),

    #[error("failed to decrypt credentials: {0}")]
    Decrypt(#[from] crate::auth::CryptoError),

    #[error(transparent)]
    Sync(#[from] SyncError),
}

async fn build_client_and_sync(
    config: &SyncWorkerConfig,
    job: &db::SyncJobRow,
) -> Result<SyncResult, WorkerError> {
    let creds = db::get_tripit_credentials(&config.pool, job.user_id)
        .await
        .map_err(SyncError::Database)?
        .ok_or(WorkerError::MissingCredentials(job.user_id))?;

    let access_token = decrypt_token(
        &creds.access_token_enc,
        &creds.nonce_token,
        &config.encryption_key,
    )?;

    let access_token_secret = decrypt_token(
        &creds.access_token_secret_enc,
        &creds.nonce_secret,
        &config.encryption_key,
    )?;

    let auth = TripItAuth::new(
        config.consumer_key.clone(),
        config.consumer_secret.clone(),
        access_token,
        access_token_secret,
    );
    let client = TripItClient::new(auth);

    Ok(sync_all(&client, &config.pool, job.user_id).await?)
}

fn list_field_as_vec(value: &Value, key: &str) -> Vec<Value> {
    match value.get(key) {
        Some(Value::Array(items)) => items.clone(),
        Some(Value::Null) | None => Vec::new(),
        Some(other) => vec![other.clone()],
    }
}

fn parse_max_page(value: &Value) -> u64 {
    value
        .get("max_page")
        .and_then(|max_page| {
            max_page
                .as_u64()
                .or_else(|| max_page.as_str().and_then(|v| v.parse::<u64>().ok()))
        })
        .unwrap_or(1)
}

fn parse_trip_id(trip: &Value) -> Option<String> {
    trip.get("id").and_then(|id| {
        id.as_str()
            .map(std::string::ToString::to_string)
            .or_else(|| id.as_u64().map(|num| num.to_string()))
    })
}

async fn fetch_unique_trip_count(api: &dyn TripItApi) -> Result<u64, FetchError> {
    let mut seen = HashSet::new();

    for past in [true, false] {
        let mut page = 1_u64;
        loop {
            let data = api.list_trips(past, page, 25).await?;

            for trip in list_field_as_vec(&data, "Trip") {
                if let Some(id) = parse_trip_id(&trip) {
                    let _ = seen.insert(id);
                }
            }

            let max_page = parse_max_page(&data);
            if page >= max_page {
                break;
            }
            page += 1;
        }
    }

    Ok(u64::try_from(seen.len()).unwrap_or(u64::MAX))
}

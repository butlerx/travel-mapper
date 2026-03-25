//! Background sync worker — polls for pending sync jobs and runs `TripIt` imports.

use crate::{
    auth::decrypt_token,
    db,
    geocode::Geocoder,
    integrations::{
        flight_status::{AviationStackClient, FlightStatusApi},
        tripit::{self, FetchError, TripItApi, TripItAuth, TripItClient},
    },
};
use sqlx::SqlitePool;
use std::time::Instant;
use tokio::sync::watch;

/// Result of a successful `TripIt` sync — counts and duration.
#[derive(Debug)]
pub struct SyncOutcome {
    pub trips_fetched: u64,
    pub hops_fetched: u64,
    pub duration_ms: u64,
}

/// Errors that can occur during a `TripIt` sync.
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
    geocoder: &crate::geocode::Geocoder,
    pool: &SqlitePool,
    user_id: i64,
) -> Result<SyncOutcome, SyncError> {
    let started_at = Instant::now();
    let mut state = db::sync_state::GetOrCreate { user_id }
        .execute(pool)
        .await?;

    tracing::info!(user_id, "sync started");

    state.sync_status = "running".to_string();
    db::sync_state::Update {
        user_id,
        state: &state,
    }
    .execute(pool)
    .await?;

    let sync_result = async {
        let trips = tripit::fetch_trips(api, geocoder).await?;
        let trips_fetched = u64::try_from(trips.len()).unwrap_or(u64::MAX);
        tracing::info!(user_id, trips_fetched, "fetched trips from TripIt");

        let mut hops_fetched = 0_u64;
        let mut active_trip_ids = Vec::with_capacity(trips.len());
        for trip in &trips {
            let tripit_trip_id = format!("tripit:{}", trip.trip_id);
            active_trip_ids.push(trip.trip_id.clone());

            let inserted = db::hops::ReplaceForTrip {
                trip_id: &tripit_trip_id,
                user_id,
                hops: &trip.hops,
            }
            .execute(pool)
            .await?;
            hops_fetched += inserted;
            tracing::debug!(
                user_id,
                trip_id = trip.trip_id,
                display_name = trip.display_name,
                inserted,
                "imported trip",
            );
        }

        let stale_deleted = db::hops::DeleteStaleTripItTrips {
            user_id,
            active_trip_ids: &active_trip_ids,
        }
        .execute(pool)
        .await?;
        if stale_deleted > 0 {
            tracing::info!(user_id, stale_deleted, "removed stale tripit hops");
        }

        let now = sqlx::query_scalar!("SELECT datetime('now')")
            .fetch_one(pool)
            .await?;

        state.last_sync_at = now;
        state.sync_status = "idle".to_string();
        state.trips_fetched =
            i64::try_from(trips_fetched).map_err(|_| SyncError::CountOverflow("trips_fetched"))?;
        state.hops_fetched =
            i64::try_from(hops_fetched).map_err(|_| SyncError::CountOverflow("hops_fetched"))?;
        db::sync_state::Update {
            user_id,
            state: &state,
        }
        .execute(pool)
        .await?;

        Ok(SyncOutcome {
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
        let _ = db::sync_state::Update {
            user_id,
            state: &state,
        }
        .execute(pool)
        .await;
    }

    sync_result
}

/// Configuration for the long-running sync worker process.
pub struct SyncWorkerConfig {
    pub pool: SqlitePool,
    pub encryption_key: [u8; 32],
    pub consumer_key: String,
    pub consumer_secret: String,
    pub poll_interval: std::time::Duration,
    pub aviationstack_api_key: Option<String>,
}

/// Run the sync worker loop, polling for pending jobs until shutdown.
///
/// # Errors
///
/// Returns an error if resetting stale jobs fails on startup.
pub async fn run_sync_worker(
    config: SyncWorkerConfig,
    mut shutdown: watch::Receiver<bool>,
) -> Result<(), sqlx::Error> {
    let reset_jobs = db::sync_jobs::ResetStaleRunning
        .execute(&config.pool)
        .await?;
    let reset_states = db::sync_jobs::ResetStaleSyncStates
        .execute(&config.pool)
        .await?;
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

        match db::sync_jobs::ClaimNext.execute(&config.pool).await {
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

async fn process_sync_job(config: &SyncWorkerConfig, job: &db::sync_jobs::Row) {
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
            enrich_flight_statuses(config, job.user_id).await;
            if let Err(err) = (db::sync_jobs::Complete { job_id: job.id })
                .execute(&config.pool)
                .await
            {
                tracing::error!(job_id = job.id, error = %err, "failed to mark job completed");
            }
        }
        Err(err) => {
            tracing::error!(job_id = job.id, user_id = job.user_id, error = %err, "sync job failed");
            if let Err(db_err) = (db::sync_jobs::Fail {
                job_id: job.id,
                error_message: &err.to_string(),
            })
            .execute(&config.pool)
            .await
            {
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
    job: &db::sync_jobs::Row,
) -> Result<SyncOutcome, WorkerError> {
    let creds = db::credentials::Get {
        user_id: job.user_id,
    }
    .execute(&config.pool)
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

    let geocoder = Geocoder::default();
    Ok(sync_all(&client, &geocoder, &config.pool, job.user_id).await?)
}

/// Enrich air-type hops with flight status data from `AviationStack`.
///
/// Failures are logged and never propagated — enrichment is optional and must
/// not block sync completion.
async fn enrich_flight_statuses(config: &SyncWorkerConfig, user_id: i64) {
    let Some(ref api_key) = config.aviationstack_api_key else {
        return;
    };

    let hops = match (db::hops::GetAll {
        user_id,
        travel_type_filter: Some("air"),
    })
    .execute(&config.pool)
    .await
    {
        Ok(hops) => hops,
        Err(err) => {
            tracing::warn!(user_id, error = %err, "failed to fetch air hops for enrichment");
            return;
        }
    };

    if hops.is_empty() {
        return;
    }

    let client = AviationStackClient::new(api_key.clone());
    let mut enriched = 0_u64;

    for hop in &hops {
        let detail = match (db::hops::GetById {
            id: hop.id,
            user_id,
        })
        .execute(&config.pool)
        .await
        {
            Ok(Some(d)) => d,
            Ok(None) => continue,
            Err(err) => {
                tracing::warn!(hop_id = hop.id, error = %err, "failed to fetch hop detail for enrichment");
                continue;
            }
        };

        let flight_number = detail
            .flight_detail
            .as_ref()
            .map(|d| d.flight_number.as_str())
            .unwrap_or_default();
        if flight_number.is_empty() || hop.start_date.is_empty() {
            continue;
        }

        match client
            .get_flight_status(flight_number, &hop.start_date)
            .await
        {
            Ok(Some(status)) => {
                let delay = status.dep_delay_minutes.or(status.arr_delay_minutes);
                if let Err(err) = (db::status_enrichments::Upsert {
                    hop_id: hop.id,
                    provider: "aviationstack",
                    status: &status.flight_status,
                    delay_minutes: delay,
                    dep_gate: &status.dep_gate,
                    dep_terminal: &status.dep_terminal,
                    arr_gate: &status.arr_gate,
                    arr_terminal: &status.arr_terminal,
                    raw_json: &status.raw_json,
                })
                .execute(&config.pool)
                .await
                {
                    tracing::warn!(
                        hop_id = hop.id,
                        error = %err,
                        "failed to upsert flight status enrichment",
                    );
                } else {
                    enriched += 1;
                }
            }
            Ok(None) => {
                tracing::debug!(
                    hop_id = hop.id,
                    flight_number,
                    "no flight status data returned",
                );
            }
            Err(err) => {
                tracing::warn!(
                    hop_id = hop.id,
                    flight_number,
                    error = %err,
                    "flight status API request failed",
                );
            }
        }
    }

    if enriched > 0 {
        tracing::info!(user_id, enriched, "flight status enrichment complete");
    }
}

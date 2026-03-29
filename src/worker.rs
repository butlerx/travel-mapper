//! Background sync worker — polls for pending sync jobs and runs `TripIt` imports.

use crate::{
    auth::decrypt_token,
    db,
    geocode::Geocoder,
    integrations::{
        airlabs::AirLabsClient,
        darwin::DarwinClient,
        db_ris::DbRisClient,
        flight_status::{FlightStatusApi, FlightStatusError},
        opensky::OpenSkyClient,
        rail_status::RailStatusApi,
        transitland::rail_status_impl::TransitlandRailClient,
        tripit::{self, FetchError, TripItApi, TripItAuth, TripItClient},
    },
};
use sqlx::SqlitePool;
use std::time::Instant;
use tokio::sync::watch;

/// Default enrichment TTL for non-realtime data (24 hours).
pub(crate) const ENRICHMENT_TTL_SECS: i64 = 24 * 60 * 60;
/// Shorter TTL for flights departing within 1 day (2 hours).
pub(crate) const REALTIME_TTL_SECS: i64 = 2 * 60 * 60;

/// How often the worker runs a periodic enrichment sweep across all users.
const PERIODIC_ENRICHMENT_INTERVAL: std::time::Duration =
    std::time::Duration::from_secs(4 * 60 * 60);

/// Result of a successful `TripIt` sync — counts and duration.
#[derive(Debug)]
pub(crate) struct SyncOutcome {
    pub(crate) trips_fetched: u64,
    pub(crate) hops_fetched: u64,
    pub(crate) duration_ms: u64,
}

fn trip_date_envelope(hops: &[crate::db::hops::Row]) -> (Option<String>, Option<String>) {
    let start = hops.iter().map(|h| h.start_date.as_str()).min();
    let end = hops.iter().map(|h| h.end_date.as_str()).max();
    (start.map(String::from), end.map(String::from))
}

/// Errors that can occur during a `TripIt` sync.
#[derive(Debug, thiserror::Error)]
pub(crate) enum SyncError {
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
pub(crate) async fn sync_all(
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

        let hops_fetched = import_tripit_trips(pool, user_id, &trips).await?;

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

async fn import_tripit_trips(
    pool: &SqlitePool,
    user_id: i64,
    trips: &[tripit::Trip],
) -> Result<u64, SyncError> {
    let mut hops_fetched = 0_u64;
    let mut active_trip_ids = Vec::with_capacity(trips.len());

    for trip in trips {
        let tripit_trip_id = format!("tripit:{}", trip.id);
        active_trip_ids.push(trip.id.clone());

        let inserted = db::hops::ReplaceForTrip {
            trip_id: &tripit_trip_id,
            user_id,
            hops: &trip.hops,
        }
        .execute(pool)
        .await?;
        hops_fetched += inserted;

        let (start_date, end_date) = trip_date_envelope(&trip.hops);
        let local_trip_id = db::trips::UpsertFromTripIt {
            user_id,
            tripit_id: &trip.id,
            name: &trip.display_name,
            start_date: start_date.as_deref(),
            end_date: end_date.as_deref(),
        }
        .execute(pool)
        .await?;

        let scoped = format!("{user_id}:{tripit_trip_id}");
        let assigned = db::trips::AssignHopsBySourceTrip {
            user_id,
            source_trip_id: &scoped,
            local_trip_id,
        }
        .execute(pool)
        .await?;

        tracing::debug!(
            user_id,
            trip_id = trip.id,
            display_name = trip.display_name,
            inserted,
            assigned,
            local_trip_id,
            "imported trip",
        );
    }

    let stale_hops_deleted = db::hops::DeleteStaleTripItTrips {
        user_id,
        active_trip_ids: &active_trip_ids,
    }
    .execute(pool)
    .await?;
    let stale_trips_deleted = db::trips::DeleteStaleTripItTrips {
        user_id,
        active_tripit_ids: &active_trip_ids,
    }
    .execute(pool)
    .await?;
    if stale_hops_deleted > 0 || stale_trips_deleted > 0 {
        tracing::info!(
            user_id,
            stale_hops_deleted,
            stale_trips_deleted,
            "removed stale tripit data",
        );
    }

    Ok(hops_fetched)
}

/// Configuration for the long-running sync worker process.
pub struct SyncWorkerConfig {
    pub pool: SqlitePool,
    pub encryption_key: [u8; 32],
    pub consumer_key: String,
    pub consumer_secret: String,
    pub poll_interval: std::time::Duration,
    pub airlabs_api_key: Option<String>,
    pub opensky_client_id: Option<String>,
    pub opensky_client_secret: Option<String>,
    pub darwin_api_token: Option<String>,
    pub db_ris_api_key: Option<String>,
    pub db_ris_client_id: Option<String>,
    pub transitland_api_key: Option<String>,
    pub vapid_private_key: Option<Vec<u8>>,
}

async fn is_enrichment_fresh(
    pool: &SqlitePool,
    hop_id: i64,
    provider: &str,
    start_date: &str,
) -> bool {
    let existing = (db::status_enrichments::GetByHopIdAndProvider { hop_id, provider })
        .execute(pool)
        .await;
    let Some(Ok(Some(row))) = Some(existing) else {
        return false;
    };

    let ttl = departure_aware_ttl(start_date);
    let fresh = sqlx::query_scalar!(
        "SELECT (strftime('%s', datetime('now')) - strftime('%s', ?)) < ?",
        row.fetched_at,
        ttl,
    )
    .fetch_one(pool)
    .await;
    matches!(fresh, Ok(1))
}

pub(crate) fn departure_aware_ttl(start_date: &str) -> i64 {
    let dep = start_date
        .get(..10)
        .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());
    let Some(dep) = dep else {
        return ENRICHMENT_TTL_SECS;
    };
    let today = chrono::Utc::now().date_naive();
    let days_until = (dep - today).num_days();
    if days_until <= 1 {
        REALTIME_TTL_SECS
    } else {
        ENRICHMENT_TTL_SECS
    }
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

    let mut last_enrichment_sweep = Instant::now()
        .checked_sub(PERIODIC_ENRICHMENT_INTERVAL)
        .unwrap_or_else(Instant::now);

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

        if last_enrichment_sweep.elapsed() >= PERIODIC_ENRICHMENT_INTERVAL {
            run_periodic_enrichment_sweep(&config).await;
            last_enrichment_sweep = Instant::now();
        }
    }
}

async fn run_periodic_enrichment_sweep(config: &SyncWorkerConfig) {
    let user_ids = match db::sync_state::GetAllUserIds.execute(&config.pool).await {
        Ok(ids) => ids,
        Err(err) => {
            tracing::error!(error = %err, "failed to fetch user ids for periodic enrichment");
            return;
        }
    };

    if user_ids.is_empty() {
        return;
    }

    tracing::info!(users = user_ids.len(), "starting periodic enrichment sweep");

    for user_id in &user_ids {
        enrich_flight_statuses(config, *user_id).await;
        verify_flight_routes(config, *user_id).await;
        enrich_rail_statuses(config, *user_id).await;
    }

    tracing::info!(users = user_ids.len(), "periodic enrichment sweep complete");
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
            verify_flight_routes(config, job.user_id).await;
            enrich_rail_statuses(config, job.user_id).await;
            if let Some(ref vapid_key) = config.vapid_private_key {
                let duration_secs =
                    std::time::Duration::from_millis(sync_result.duration_ms).as_secs_f64();
                let body = format!(
                    "Synced {} trips with {} journeys in {:.1}s",
                    sync_result.trips_fetched, sync_result.hops_fetched, duration_secs,
                );
                crate::server::push::send_to_user(
                    &config.pool,
                    vapid_key,
                    job.user_id,
                    "Sync Complete",
                    &body,
                    "/dashboard",
                )
                .await;
            }
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

    let geocoder = Geocoder::new(config.pool.clone());
    Ok(sync_all(&client, &geocoder, &config.pool, job.user_id).await?)
}

/// Enrich air-type hops with flight status data from `AirLabs`.
///
/// Failures are logged and never propagated — enrichment is optional and must
/// not block sync completion.
async fn enrich_flight_statuses(config: &SyncWorkerConfig, user_id: i64) {
    let Some(ref api_key) = config.airlabs_api_key else {
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

    let client = AirLabsClient::new(api_key.clone());
    let mut enriched = 0_u64;

    for hop in &hops {
        match enrich_single_flight_hop(config, &client, user_id, hop).await {
            FlightEnrichResult::Enriched => enriched += 1,
            FlightEnrichResult::RateLimited => {
                tracing::info!(
                    user_id,
                    "airlabs rate limit reached, stopping flight enrichment"
                );
                break;
            }
            FlightEnrichResult::Skipped => {}
        }
    }

    if enriched > 0 {
        tracing::info!(user_id, enriched, "flight status enrichment complete");
    }
}

enum FlightEnrichResult {
    Enriched,
    RateLimited,
    Skipped,
}

async fn enrich_single_flight_hop(
    config: &SyncWorkerConfig,
    client: &AirLabsClient,
    user_id: i64,
    hop: &db::hops::Row,
) -> FlightEnrichResult {
    let detail = match (db::hops::GetById {
        id: hop.id,
        user_id,
    })
    .execute(&config.pool)
    .await
    {
        Ok(Some(d)) => d,
        Ok(None) => return FlightEnrichResult::Skipped,
        Err(err) => {
            tracing::warn!(hop_id = hop.id, error = %err, "failed to fetch hop detail for enrichment");
            return FlightEnrichResult::Skipped;
        }
    };

    let flight_number = detail
        .flight_detail
        .as_ref()
        .map(|d| d.flight_number.as_str())
        .unwrap_or_default();
    if flight_number.is_empty() || hop.start_date.is_empty() {
        return FlightEnrichResult::Skipped;
    }

    if is_enrichment_fresh(&config.pool, hop.id, "airlabs", &hop.start_date).await {
        tracing::debug!(hop_id = hop.id, "airlabs enrichment still fresh, skipping");
        return FlightEnrichResult::Skipped;
    }

    match client
        .get_flight_status(flight_number, &hop.start_date)
        .await
    {
        Ok(Some(status)) => {
            let delay = status.dep_delay_minutes.or(status.arr_delay_minutes);
            if let Err(err) = (db::status_enrichments::Upsert {
                hop_id: hop.id,
                provider: "airlabs",
                status: &status.status,
                delay_minutes: delay,
                dep_gate: &status.dep_gate,
                dep_terminal: &status.dep_terminal,
                arr_gate: &status.arr_gate,
                arr_terminal: &status.arr_terminal,
                dep_platform: "",
                arr_platform: "",
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
                FlightEnrichResult::Skipped
            } else {
                FlightEnrichResult::Enriched
            }
        }
        Ok(None) => {
            tracing::debug!(
                hop_id = hop.id,
                flight_number,
                "no flight status data returned, writing sentinel",
            );
            if let Err(err) = (db::status_enrichments::Upsert {
                hop_id: hop.id,
                provider: "airlabs",
                status: "",
                delay_minutes: None,
                dep_gate: "",
                dep_terminal: "",
                arr_gate: "",
                arr_terminal: "",
                dep_platform: "",
                arr_platform: "",
                raw_json: "",
            })
            .execute(&config.pool)
            .await
            {
                tracing::warn!(hop_id = hop.id, error = %err, "failed to write no-data sentinel");
            }
            FlightEnrichResult::Skipped
        }
        Err(FlightStatusError::RateLimited) => FlightEnrichResult::RateLimited,
        Err(err) => {
            tracing::warn!(
                hop_id = hop.id,
                flight_number,
                error = %err,
                "flight status API request failed",
            );
            FlightEnrichResult::Skipped
        }
    }
}

/// Verify air-hop routes against `OpenSky` ADS-B observations.
///
/// Failures are logged and never propagated — verification is optional and must
/// not block sync completion.
async fn verify_flight_routes(config: &SyncWorkerConfig, user_id: i64) {
    let (Some(client_id), Some(client_secret)) =
        (&config.opensky_client_id, &config.opensky_client_secret)
    else {
        return;
    };

    let Some(hops) = fetch_air_hops_for_route_verification(config, user_id).await else {
        return;
    };

    if hops.is_empty() {
        return;
    }

    let client = OpenSkyClient::new(client_id.clone(), client_secret.clone());
    let mut verified = 0_u64;

    for hop in &hops {
        match verify_air_hop_route(config, &client, user_id, hop).await {
            RouteVerificationOutcome::Verified => {
                verified += 1;
            }
            RouteVerificationOutcome::RateLimited => {
                tracing::info!(
                    user_id,
                    requests = client.requests_made(),
                    "opensky rate limit reached, stopping verification",
                );
                break;
            }
            RouteVerificationOutcome::Noop => {}
        }
    }

    if verified > 0 {
        tracing::info!(user_id, verified, "opensky route verification complete");
    }
}

async fn fetch_air_hops_for_route_verification(
    config: &SyncWorkerConfig,
    user_id: i64,
) -> Option<Vec<db::hops::Row>> {
    match (db::hops::GetAll {
        user_id,
        travel_type_filter: Some("air"),
    })
    .execute(&config.pool)
    .await
    {
        Ok(hops) => Some(hops),
        Err(err) => {
            tracing::warn!(user_id, error = %err, "failed to fetch air hops for route verification");
            None
        }
    }
}

enum RouteVerificationOutcome {
    Verified,
    RateLimited,
    Noop,
}

async fn verify_air_hop_route(
    config: &SyncWorkerConfig,
    client: &OpenSkyClient,
    user_id: i64,
    hop: &db::hops::Row,
) -> RouteVerificationOutcome {
    let detail = match (db::hops::GetById {
        id: hop.id,
        user_id,
    })
    .execute(&config.pool)
    .await
    {
        Ok(Some(d)) => d,
        Ok(None) => return RouteVerificationOutcome::Noop,
        Err(err) => {
            tracing::warn!(hop_id = hop.id, error = %err, "failed to fetch hop detail for route verification");
            return RouteVerificationOutcome::Noop;
        }
    };

    let flight_number = detail
        .flight_detail
        .as_ref()
        .map(|d| d.flight_number.as_str())
        .unwrap_or_default();
    if flight_number.is_empty() || hop.start_date.is_empty() {
        return RouteVerificationOutcome::Noop;
    }

    if is_enrichment_fresh(&config.pool, hop.id, "opensky", &hop.start_date).await {
        tracing::debug!(hop_id = hop.id, "opensky enrichment still fresh, skipping");
        return RouteVerificationOutcome::Noop;
    }

    match client
        .verify_route(
            flight_number,
            &hop.start_date,
            &hop.origin_name,
            &hop.dest_name,
        )
        .await
    {
        Ok(Some(verification)) => {
            let status = if verification.operated {
                "verified"
            } else {
                "unverified"
            };
            if let Err(err) = (db::status_enrichments::Upsert {
                hop_id: hop.id,
                provider: "opensky",
                status,
                delay_minutes: None,
                dep_gate: "",
                dep_terminal: "",
                arr_gate: "",
                arr_terminal: "",
                dep_platform: "",
                arr_platform: "",
                raw_json: &verification.raw_json,
            })
            .execute(&config.pool)
            .await
            {
                tracing::warn!(hop_id = hop.id, error = %err, "failed to upsert opensky route verification");
                RouteVerificationOutcome::Noop
            } else {
                RouteVerificationOutcome::Verified
            }
        }
        Ok(None) => {
            tracing::debug!(
                hop_id = hop.id,
                flight_number,
                "no opensky route data found"
            );
            RouteVerificationOutcome::Noop
        }
        Err(crate::integrations::opensky::OpenSkyError::RateLimited) => {
            RouteVerificationOutcome::RateLimited
        }
        Err(err) => {
            tracing::warn!(hop_id = hop.id, flight_number, error = %err, "opensky route verification failed");
            RouteVerificationOutcome::Noop
        }
    }
}

fn select_rail_provider<'a>(
    origin_country: Option<&'a str>,
    dest_country: Option<&'a str>,
) -> &'static str {
    if origin_country != dest_country {
        return "transitland";
    }
    match origin_country {
        Some("gb") => "darwin",
        Some("de") => "db_ris",
        Some("us") => "amtrak",
        _ => "transitland",
    }
}

async fn enrich_rail_statuses(config: &SyncWorkerConfig, user_id: i64) {
    let has_any_provider = config.darwin_api_token.is_some()
        || config.db_ris_api_key.is_some()
        || config.transitland_api_key.is_some();

    if !has_any_provider {
        return;
    }

    let hops = match (db::hops::GetAll {
        user_id,
        travel_type_filter: Some("rail"),
    })
    .execute(&config.pool)
    .await
    {
        Ok(hops) => hops,
        Err(err) => {
            tracing::warn!(user_id, error = %err, "failed to fetch rail hops for enrichment");
            return;
        }
    };

    if hops.is_empty() {
        return;
    }

    let mut enriched = 0_u64;

    for hop in &hops {
        match enrich_single_rail_hop(config, user_id, hop).await {
            Some(RailEnrichResult::Enriched) => enriched += 1,
            Some(RailEnrichResult::RateLimited) => break,
            Some(RailEnrichResult::Skipped) | None => {}
        }
    }

    if enriched > 0 {
        tracing::info!(user_id, enriched, "rail status enrichment complete",);
    }
}

enum RailEnrichResult {
    Enriched,
    RateLimited,
    Skipped,
}

async fn enrich_single_rail_hop(
    config: &SyncWorkerConfig,
    user_id: i64,
    hop: &db::hops::Row,
) -> Option<RailEnrichResult> {
    let detail = match (db::hops::GetById {
        id: hop.id,
        user_id,
    })
    .execute(&config.pool)
    .await
    {
        Ok(Some(d)) => d,
        Ok(None) => return None,
        Err(err) => {
            tracing::warn!(hop_id = hop.id, error = %err, "failed to fetch hop detail for rail enrichment");
            return None;
        }
    };

    let train_number = detail
        .rail_detail
        .as_ref()
        .map(|d| d.train_number.as_str())
        .unwrap_or_default();
    if train_number.is_empty() || hop.start_date.is_empty() {
        return None;
    }

    let provider = select_rail_provider(hop.origin_country.as_deref(), hop.dest_country.as_deref());

    let provider_configured = match provider {
        "darwin" => config.darwin_api_token.is_some(),
        "db_ris" => config.db_ris_api_key.is_some(),
        "transitland" => config.transitland_api_key.is_some(),
        _ => false,
    };

    if !provider_configured {
        tracing::debug!(
            hop_id = hop.id,
            provider,
            "rail provider not configured, skipping",
        );
        return Some(RailEnrichResult::Skipped);
    }

    if is_enrichment_fresh(&config.pool, hop.id, provider, &hop.start_date).await {
        tracing::debug!(
            hop_id = hop.id,
            provider,
            "rail enrichment still fresh, skipping"
        );
        return Some(RailEnrichResult::Skipped);
    }

    let carrier = detail
        .rail_detail
        .as_ref()
        .map(|d| d.carrier.as_str())
        .unwrap_or_default();

    let query = crate::integrations::rail_status::RailStatusQuery {
        carrier,
        train_number,
        origin_name: &hop.origin_name,
        dest_name: &hop.dest_name,
        origin_country: hop.origin_country.as_deref(),
        dest_country: hop.dest_country.as_deref(),
        start_date: &hop.start_date,
        end_date: &hop.end_date,
        origin_lat: hop.origin_lat,
        origin_lng: hop.origin_lng,
        dest_lat: hop.dest_lat,
        dest_lng: hop.dest_lng,
    };

    dispatch_and_upsert_rail_status(config, user_id, hop, provider, train_number, &query).await
}

async fn dispatch_and_upsert_rail_status(
    config: &SyncWorkerConfig,
    user_id: i64,
    hop: &db::hops::Row,
    provider: &str,
    train_number: &str,
    query: &crate::integrations::rail_status::RailStatusQuery<'_>,
) -> Option<RailEnrichResult> {
    let result = if provider == "db_ris" {
        let (Some(api_key), Some(client_id)) = (&config.db_ris_api_key, &config.db_ris_client_id)
        else {
            return Some(RailEnrichResult::Skipped);
        };
        let client = DbRisClient::new(api_key.clone(), client_id.clone(), config.pool.clone());
        client.get_rail_status(query).await
    } else if provider == "darwin" {
        let Some(api_token) = &config.darwin_api_token else {
            return Some(RailEnrichResult::Skipped);
        };
        let client = DarwinClient::new(api_token.clone());
        client.get_rail_status(query).await
    } else if provider == "transitland" {
        let Some(api_key) = &config.transitland_api_key else {
            return Some(RailEnrichResult::Skipped);
        };
        match TransitlandRailClient::new(api_key.clone(), config.pool.clone()) {
            Ok(client) => client.get_rail_status(query).await,
            Err(err) => {
                tracing::warn!(error = %err, "failed to create transitland client");
                return Some(RailEnrichResult::Skipped);
            }
        }
    } else {
        tracing::debug!(
            hop_id = hop.id,
            provider,
            train_number,
            "rail provider not yet implemented",
        );
        return Some(RailEnrichResult::Skipped);
    };

    match result {
        Ok(Some(status)) => {
            let delay = status.dep_delay_minutes.or(status.arr_delay_minutes);
            if let Err(err) = (db::status_enrichments::Upsert {
                hop_id: hop.id,
                provider,
                status: &status.status,
                delay_minutes: delay,
                dep_gate: "",
                dep_terminal: "",
                arr_gate: "",
                arr_terminal: "",
                dep_platform: &status.dep_platform,
                arr_platform: &status.arr_platform,
                raw_json: &status.raw_json,
            })
            .execute(&config.pool)
            .await
            {
                tracing::warn!(
                    hop_id = hop.id,
                    error = %err,
                    "failed to upsert rail status enrichment",
                );
            } else {
                return Some(RailEnrichResult::Enriched);
            }
            Some(RailEnrichResult::Skipped)
        }
        Ok(None) => {
            tracing::debug!(
                hop_id = hop.id,
                train_number,
                "no rail status data returned",
            );
            Some(RailEnrichResult::Skipped)
        }
        Err(crate::integrations::rail_status::RailStatusError::RateLimited) => {
            tracing::info!(
                user_id,
                provider,
                "rail provider rate limit reached, stopping enrichment",
            );
            Some(RailEnrichResult::RateLimited)
        }
        Err(err) => {
            tracing::warn!(
                hop_id = hop.id,
                train_number,
                error = %err,
                "rail status API request failed",
            );
            Some(RailEnrichResult::Skipped)
        }
    }
}

use crate::auth::{CryptoError, parse_encryption_key};
use clap::Args as ClapArgs;
use std::path::PathBuf;
use std::time::Duration;
use tokio::sync::watch;

#[derive(ClapArgs)]
pub struct Args {
    #[arg(long, env = "TRIPIT_CONSUMER_KEY")]
    consumer_key: String,

    #[arg(long, env = "TRIPIT_CONSUMER_SECRET")]
    consumer_secret: String,

    #[arg(long, env = "ENCRYPTION_KEY")]
    encryption_key: String,

    #[arg(long, env = "DATABASE_URL", default_value = "sqlite:travel.db")]
    database_url: String,

    #[arg(long, env = "SYNC_POLL_INTERVAL_SECS", default_value_t = 5)]
    poll_interval_secs: u64,

    #[arg(long, env = "AIRLABS_API_KEY")]
    airlabs_api_key: Option<String>,

    #[arg(long, env = "OPENSKY_CLIENT_ID")]
    opensky_client_id: Option<String>,

    #[arg(long, env = "OPENSKY_CLIENT_SECRET")]
    opensky_client_secret: Option<String>,

    #[arg(long, env = "DARWIN_API_TOKEN")]
    darwin_api_token: Option<String>,

    #[arg(long, env = "DB_RIS_API_KEY")]
    db_ris_api_key: Option<String>,

    #[arg(long, env = "DB_RIS_CLIENT_ID")]
    db_ris_client_id: Option<String>,

    #[arg(long, env = "TRANSITLAND_API_KEY")]
    transitland_api_key: Option<String>,

    #[arg(long, env = "VAPID_PRIVATE_KEY_PATH")]
    vapid_private_key_path: Option<PathBuf>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Crypto(#[from] CryptoError),

    #[error("failed to create database pool: {0}")]
    Database(#[from] sqlx::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Start the background sync worker, polling for pending sync jobs.
///
/// # Errors
///
/// Returns an error if database access, encryption key parsing, or worker startup fails.
pub async fn run(args: Args) -> Result<(), Error> {
    let encryption_key = parse_encryption_key(&args.encryption_key)?;
    let pool = crate::db::create_pool(&args.database_url).await?;

    let (_shutdown_tx, shutdown_rx) = watch::channel(false);

    let config = crate::worker::SyncWorkerConfig {
        pool,
        encryption_key,
        consumer_key: args.consumer_key,
        consumer_secret: args.consumer_secret,
        poll_interval: Duration::from_secs(args.poll_interval_secs),
        airlabs_api_key: args.airlabs_api_key,
        opensky_client_id: args.opensky_client_id,
        opensky_client_secret: args.opensky_client_secret,
        darwin_api_token: args.darwin_api_token,
        db_ris_api_key: args.db_ris_api_key,
        db_ris_client_id: args.db_ris_client_id,
        transitland_api_key: args.transitland_api_key,
        vapid_private_key: args
            .vapid_private_key_path
            .as_ref()
            .map(std::fs::read)
            .transpose()
            .map_err(Error::Io)?,
    };

    tracing::info!(
        poll_interval_secs = args.poll_interval_secs,
        "starting sync worker"
    );

    tokio::select! {
        result = crate::worker::run_sync_worker(config, shutdown_rx) => {
            result.map_err(Error::Database)?;
        }
        () = crate::shutdown_signal() => {
            tracing::info!("shutting down sync worker");
        }
    }

    Ok(())
}

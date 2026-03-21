use clap::Parser;
use std::time::Duration;
use tokio::sync::watch;
use tracing_subscriber::prelude::*;
use travel_export::{db, worker::SyncWorkerConfig};

#[derive(Parser)]
#[command(about = "Background sync worker that processes TripIt sync jobs")]
struct Cli {
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
}

#[derive(Debug, thiserror::Error)]
enum WorkerError {
    #[error("invalid ENCRYPTION_KEY: expected exactly 32 bytes hex")]
    InvalidEncryptionKey,

    #[error("failed to create database pool: {0}")]
    Database(#[from] sqlx::Error),
}

fn init_tracing() {
    #[cfg(debug_assertions)]
    let log_layer = tracing_subscriber::fmt::layer()
        .pretty()
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE);

    #[cfg(not(debug_assertions))]
    let log_layer = tracing_subscriber::fmt::layer()
        .json()
        .set_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE);

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with(log_layer)
        .init();
}

fn parse_encryption_key(hex: &str) -> Result<[u8; 32], WorkerError> {
    if hex.len() != 64 {
        return Err(WorkerError::InvalidEncryptionKey);
    }

    let mut out = [0_u8; 32];
    for (idx, chunk) in hex.as_bytes().chunks(2).enumerate() {
        let pair = std::str::from_utf8(chunk).map_err(|_| WorkerError::InvalidEncryptionKey)?;
        out[idx] = u8::from_str_radix(pair, 16).map_err(|_| WorkerError::InvalidEncryptionKey)?;
    }
    Ok(out)
}

async fn run() -> Result<(), WorkerError> {
    let cli = Cli::parse();

    let encryption_key = parse_encryption_key(&cli.encryption_key)?;
    let pool = db::create_pool(&cli.database_url).await?;

    let (_shutdown_tx, shutdown_rx) = watch::channel(false);

    let config = SyncWorkerConfig {
        pool,
        encryption_key,
        consumer_key: cli.consumer_key,
        consumer_secret: cli.consumer_secret,
        poll_interval: Duration::from_secs(cli.poll_interval_secs),
    };

    tracing::info!(
        poll_interval_secs = cli.poll_interval_secs,
        "starting sync worker"
    );

    tokio::select! {
        result = travel_export::worker::run_sync_worker(config, shutdown_rx) => {
            result.map_err(WorkerError::Database)?;
        }
        _ = shutdown_signal() => {
            tracing::info!("shutting down sync worker");
        }
    }

    Ok(())
}

async fn shutdown_signal() {
    if let Err(error) = tokio::signal::ctrl_c().await {
        tracing::error!("failed to install ctrl+c handler: {error}");
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    tracing::info!("shutdown signal received");
}

#[tokio::main]
async fn main() {
    init_tracing();

    if let Err(error) = run().await {
        eprintln!("Error: {error}");
        std::process::exit(1);
    }
}

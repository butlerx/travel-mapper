use clap::Parser;
use leptos::prelude::LeptosOptions;
use std::time::Duration;
use tokio::sync::watch;
use tracing_subscriber::prelude::*;
use travel_export::{
    db,
    server::{self, AppState, SyncWorkerConfig, run_sync_worker},
};

#[derive(Parser)]
#[command(about = "Run the travel-export Axum server")]
struct Cli {
    #[arg(long, env = "TRIPIT_CONSUMER_KEY")]
    consumer_key: String,

    #[arg(long, env = "TRIPIT_CONSUMER_SECRET")]
    consumer_secret: String,

    #[arg(long, env = "ENCRYPTION_KEY")]
    encryption_key: String,

    #[arg(long, env = "DATABASE_URL", default_value = "sqlite:travel.db")]
    database_url: String,

    #[arg(long, env = "PORT", default_value_t = 3000)]
    port: u16,
}

#[derive(Debug, thiserror::Error)]
enum ServerError {
    #[error("invalid ENCRYPTION_KEY: expected exactly 32 bytes hex")]
    InvalidEncryptionKey,

    #[error("failed to create database pool: {0}")]
    Database(#[from] sqlx::Error),

    #[error("failed to bind TCP listener: {0}")]
    Bind(#[from] std::io::Error),
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
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info,tower_http=debug")),
        )
        .with(log_layer)
        .init()
}

fn parse_encryption_key(hex: &str) -> Result<[u8; 32], ServerError> {
    if hex.len() != 64 {
        return Err(ServerError::InvalidEncryptionKey);
    }

    let mut out = [0_u8; 32];
    for (idx, chunk) in hex.as_bytes().chunks(2).enumerate() {
        let pair = std::str::from_utf8(chunk).map_err(|_| ServerError::InvalidEncryptionKey)?;
        out[idx] = u8::from_str_radix(pair, 16).map_err(|_| ServerError::InvalidEncryptionKey)?;
    }
    Ok(out)
}

async fn run() -> Result<(), ServerError> {
    let cli = Cli::parse();

    let encryption_key = parse_encryption_key(&cli.encryption_key)?;
    let pool = db::create_pool(&cli.database_url).await?;

    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    let worker_config = SyncWorkerConfig {
        pool: pool.clone(),
        encryption_key,
        consumer_key: cli.consumer_key.clone(),
        consumer_secret: cli.consumer_secret.clone(),
        poll_interval: Duration::from_secs(5),
    };

    let worker_handle = tokio::spawn(async move {
        if let Err(err) = run_sync_worker(worker_config, shutdown_rx).await {
            tracing::error!(error = %err, "sync worker failed");
        }
    });

    let state = AppState {
        leptos_options: LeptosOptions::builder()
            .output_name("travel-mapper")
            .build(),
        db: pool,
        encryption_key,
        tripit_consumer_key: cli.consumer_key,
        tripit_consumer_secret: cli.consumer_secret,
        tripit_override: None,
    };
    let app = server::create_router(state);

    let address = format!("0.0.0.0:{}", cli.port);
    let listener = tokio::net::TcpListener::bind(&address).await?;
    tracing::info!("Listening on http://{address}");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(ServerError::Bind)?;

    tracing::info!("HTTP server stopped, shutting down sync worker");
    let _ = shutdown_tx.send(true);
    let _ = worker_handle.await;

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

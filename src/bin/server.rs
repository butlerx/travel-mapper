use clap::Parser;
use leptos::prelude::LeptosOptions;
use std::{path::PathBuf, time::Duration};

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

    #[arg(long, env = "REGISTRATION_ENABLED", default_value_t = true)]
    registration_enabled: bool,

    #[arg(long, env = "AIRLABS_API_KEY")]
    airlabs_api_key: Option<String>,

    #[arg(long, env = "ATTACHMENTS_PATH")]
    storage_path: Option<PathBuf>,

    #[arg(long, env = "SMTP_HOST")]
    smtp_host: Option<String>,

    #[arg(long, env = "SMTP_PORT", default_value_t = 587)]
    smtp_port: u16,

    #[arg(long, env = "SMTP_USERNAME")]
    smtp_username: Option<String>,

    #[arg(long, env = "SMTP_PASSWORD")]
    smtp_password: Option<String>,

    #[arg(long, env = "EMAIL_FROM")]
    email_from: Option<String>,
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
    let pool = travel_mapper::db::create_pool(&cli.database_url).await?;

    let smtp_config = match (
        cli.smtp_host,
        cli.smtp_username,
        cli.smtp_password,
        cli.email_from,
    ) {
        (Some(host), Some(username), Some(password), Some(from)) => {
            Some(travel_mapper::server::SmtpConfig {
                host,
                port: cli.smtp_port,
                username,
                password,
                from,
            })
        }
        _ => None,
    };

    let state = travel_mapper::server::AppState {
        leptos_options: LeptosOptions::builder()
            .output_name("travel-mapper")
            .build(),
        db: pool,
        encryption_key,
        tripit_consumer_key: cli.consumer_key,
        tripit_consumer_secret: cli.consumer_secret,
        tripit_override: None,
        registration_enabled: cli.registration_enabled,
        airlabs_api_key: cli.airlabs_api_key,
        storage_path: cli.storage_path,
        smtp_config,
    };
    let app = travel_mapper::server::create_router(state);

    let address = format!("0.0.0.0:{}", cli.port);
    let listener = tokio::net::TcpListener::bind(&address).await?;
    tracing::info!(address, "listening");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(ServerError::Bind)?;

    Ok(())
}

async fn shutdown_signal() {
    if let Err(error) = tokio::signal::ctrl_c().await {
        tracing::error!(error = %error, "failed to install ctrl+c handler");
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    tracing::info!("shutdown signal received");
}

#[tokio::main]
async fn main() {
    travel_mapper::telemetry::init();

    if let Err(error) = run().await {
        tracing::error!(%error, "server failed");
        std::process::exit(1);
    }
}

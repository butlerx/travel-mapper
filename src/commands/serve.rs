use crate::auth::{CryptoError, parse_encryption_key};
use clap::Args as ClapArgs;
use leptos::prelude::LeptosOptions;
use std::path::PathBuf;

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

    #[arg(long, env = "PORT", default_value_t = 3000)]
    port: u16,

    #[arg(long, env = "REGISTRATION_ENABLED", default_value_t = true)]
    registration_enabled: bool,

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

    #[arg(long, env = "VAPID_PRIVATE_KEY_PATH")]
    vapid_private_key_path: Option<PathBuf>,

    #[arg(long, env = "VAPID_PUBLIC_KEY")]
    vapid_public_key: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Crypto(#[from] CryptoError),

    #[error("failed to create database pool: {0}")]
    Database(#[from] sqlx::Error),

    #[error("failed to bind TCP listener: {0}")]
    Bind(#[from] std::io::Error),
}

/// Start the web server, binding to the configured address and serving all routes.
///
/// # Errors
///
/// Returns an error if database access, TCP binding, or server startup fails.
pub async fn run(args: Args) -> Result<(), Error> {
    let encryption_key = parse_encryption_key(&args.encryption_key)?;
    let pool = crate::db::create_pool(&args.database_url).await?;

    let smtp_config = match (
        args.smtp_host,
        args.smtp_username,
        args.smtp_password,
        args.email_from,
    ) {
        (Some(host), Some(username), Some(password), Some(from)) => {
            Some(crate::server::SmtpConfig {
                host,
                port: args.smtp_port,
                username,
                password,
                from,
            })
        }
        _ => None,
    };

    let vapid_private_key = args
        .vapid_private_key_path
        .as_ref()
        .map(std::fs::read)
        .transpose()
        .map_err(Error::Bind)?;

    // Registration requires working email verification, so disable it when
    // SMTP is not configured — even if the operator explicitly enabled it.
    let registration_enabled = args.registration_enabled && smtp_config.is_some();
    if args.registration_enabled && smtp_config.is_none() {
        tracing::warn!(
            "REGISTRATION_ENABLED is true but SMTP is not configured — registration disabled"
        );
    }

    let state = crate::server::AppState {
        leptos_options: LeptosOptions::builder()
            .output_name(env!("CARGO_PKG_NAME"))
            .build(),
        db: pool,
        encryption_key,
        tripit_consumer_key: args.consumer_key,
        tripit_consumer_secret: args.consumer_secret,
        tripit_override: None,
        registration_enabled,
        airlabs_api_key: args.airlabs_api_key,
        opensky_client_id: args.opensky_client_id,
        opensky_client_secret: args.opensky_client_secret,
        darwin_api_token: args.darwin_api_token,
        db_ris_api_key: args.db_ris_api_key,
        db_ris_client_id: args.db_ris_client_id,
        transitland_api_key: args.transitland_api_key,
        storage_path: args.storage_path,
        smtp_config,
        vapid_private_key,
        vapid_public_key: args.vapid_public_key,
    };
    let app = crate::server::create_router(state);

    let address = format!("0.0.0.0:{}", args.port);
    let listener = tokio::net::TcpListener::bind(&address).await?;
    tracing::info!(address, "listening");

    axum::serve(listener, app)
        .with_graceful_shutdown(crate::shutdown_signal())
        .await
        .map_err(Error::Bind)?;

    Ok(())
}

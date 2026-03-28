use clap::Parser;
use sqlx::SqlitePool;
use std::io::{self, Write};
use travel_mapper::db;

#[derive(Parser)]
#[command(about = "Create a new user interactively")]
struct Cli {
    #[arg(long, env = "DATABASE_URL", default_value = "sqlite:travel.db")]
    database_url: String,
    /// Hex-encoded 32-byte AES-256-GCM key (64 hex chars). Required to store
    /// TripIt credentials; ignored when absent.
    #[arg(long, env = "ENCRYPTION_KEY")]
    encryption_key: Option<String>,
    /// TripIt OAuth access token. Both token fields plus the encryption key
    /// must be set to store credentials.
    #[arg(long, env = "TRIPIT_ACCESS_TOKEN")]
    tripit_access_token: Option<String>,
    /// TripIt OAuth access token secret.
    #[arg(long, env = "TRIPIT_ACCESS_TOKEN_SECRET")]
    tripit_access_token_secret: Option<String>,
    username: String,
}

#[derive(Debug, thiserror::Error)]
enum CreateUserError {
    #[error("invalid ENCRYPTION_KEY: expected exactly 64 hex characters (32 bytes)")]
    InvalidEncryptionKey,
    #[error("{0}")]
    Database(#[from] sqlx::Error),
    #[error("failed to encrypt token: {0}")]
    Encrypt(#[from] travel_mapper::auth::CryptoError),
    #[error("failed to hash password: {0}")]
    HashPassword(argon2::password_hash::Error),
    #[error("TRIPIT_ACCESS_TOKEN and TRIPIT_ACCESS_TOKEN_SECRET must both be set or both be unset")]
    IncompleteCredentials,
    #[error("user {0:?} not found after unique violation")]
    UserNotFound(String),
    #[error("passwords do not match")]
    PasswordMismatch,
    #[error("password cannot be empty")]
    EmptyPassword,
    #[error("{0}")]
    Io(#[from] io::Error),
}

impl From<argon2::password_hash::Error> for CreateUserError {
    fn from(err: argon2::password_hash::Error) -> Self {
        Self::HashPassword(err)
    }
}

fn parse_encryption_key(hex: &str) -> Result<[u8; 32], CreateUserError> {
    if hex.len() != 64 {
        return Err(CreateUserError::InvalidEncryptionKey);
    }
    let mut out = [0_u8; 32];
    for (idx, chunk) in hex.as_bytes().chunks(2).enumerate() {
        let pair = std::str::from_utf8(chunk).map_err(|_| CreateUserError::InvalidEncryptionKey)?;
        out[idx] =
            u8::from_str_radix(pair, 16).map_err(|_| CreateUserError::InvalidEncryptionKey)?;
    }
    Ok(out)
}

async fn store_tripit_credentials(
    pool: &SqlitePool,
    user_id: i64,
    username: &str,
    access_token: &str,
    access_token_secret: &str,
    encryption_key: &[u8; 32],
) -> Result<(), CreateUserError> {
    let (token_enc, nonce_token) =
        travel_mapper::auth::encrypt_token(access_token, encryption_key)?;
    let (secret_enc, nonce_secret) =
        travel_mapper::auth::encrypt_token(access_token_secret, encryption_key)?;

    (db::credentials::Upsert {
        user_id,
        access_token_enc: &token_enc,
        access_token_secret_enc: &secret_enc,
        nonce_token: &nonce_token,
        nonce_secret: &nonce_secret,
    })
    .execute(pool)
    .await?;

    tracing::info!(username, "stored TripIt credentials");
    Ok(())
}

fn prompt(label: &str) -> Result<String, io::Error> {
    print!("{label}: ");
    io::stdout().flush()?;
    let mut buf = String::new();
    io::stdin().read_line(&mut buf)?;
    Ok(buf.trim().to_owned())
}

fn prompt_password() -> Result<String, CreateUserError> {
    print!("Password: ");
    io::stdout().flush()?;
    let password = rpassword::read_password()?;
    if password.is_empty() {
        return Err(CreateUserError::EmptyPassword);
    }
    print!("Confirm password: ");
    io::stdout().flush()?;
    let confirm = rpassword::read_password()?;
    if password != confirm {
        return Err(CreateUserError::PasswordMismatch);
    }
    Ok(password)
}

async fn run() -> Result<(), CreateUserError> {
    let cli = Cli::parse();
    let pool = travel_mapper::db::create_pool(&cli.database_url).await?;

    let encryption_key = cli
        .encryption_key
        .as_deref()
        .map(parse_encryption_key)
        .transpose()?;

    let tripit_creds: Option<(&str, &str)> =
        match (&cli.tripit_access_token, &cli.tripit_access_token_secret) {
            (Some(token), Some(secret)) => Some((token.as_str(), secret.as_str())),
            (None, None) => None,
            _ => return Err(CreateUserError::IncompleteCredentials),
        };

    let username = cli.username.as_str();
    let password = prompt_password()?;
    let email = prompt("Email")?;
    let first_name = prompt("First name")?;
    let last_name = prompt("Last name")?;

    let hash = travel_mapper::auth::hash_password(&password)?;

    let user_id = match (db::users::Create {
        username,
        password_hash: &hash,
        email: &email,
        first_name: &first_name,
        last_name: &last_name,
    })
    .execute(&pool)
    .await
    {
        Ok(id) => {
            tracing::info!(username, id, "created user");
            id
        }
        Err(sqlx::Error::Database(err)) if err.is_unique_violation() => {
            tracing::info!(username, "user already exists, skipping creation");
            let user = (db::users::GetByUsername { username })
                .execute(&pool)
                .await?
                .ok_or_else(|| CreateUserError::UserNotFound(username.to_owned()))?;
            user.id
        }
        Err(err) => return Err(CreateUserError::Database(err)),
    };

    if !email.is_empty() {
        (db::users::SetEmailVerified { user_id })
            .execute(&pool)
            .await?;
        tracing::info!(username, "marked email as verified");
    }

    if let (Some((access_token, access_token_secret)), Some(key)) = (tripit_creds, &encryption_key)
    {
        store_tripit_credentials(
            &pool,
            user_id,
            username,
            access_token,
            access_token_secret,
            key,
        )
        .await?;
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    travel_mapper::telemetry::init();

    if let Err(error) = run().await {
        tracing::error!(%error, "create-user failed");
        std::process::exit(1);
    }
}

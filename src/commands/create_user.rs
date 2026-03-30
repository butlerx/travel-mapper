use crate::auth::{CryptoError, parse_encryption_key};
use crate::db;
use clap::Args as ClapArgs;
use sqlx::SqlitePool;
use std::io::{self, Write};

#[derive(ClapArgs)]
pub struct Args {
    #[arg(long, env = "DATABASE_URL", default_value = "sqlite:travel.db")]
    database_url: String,

    #[arg(long, env = "ENCRYPTION_KEY")]
    encryption_key: Option<String>,

    #[arg(long, env = "TRIPIT_ACCESS_TOKEN")]
    tripit_access_token: Option<String>,

    #[arg(long, env = "TRIPIT_ACCESS_TOKEN_SECRET")]
    tripit_access_token_secret: Option<String>,

    username: String,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Crypto(#[from] CryptoError),

    #[error("{0}")]
    Database(#[from] sqlx::Error),

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

impl From<argon2::password_hash::Error> for Error {
    fn from(err: argon2::password_hash::Error) -> Self {
        Self::HashPassword(err)
    }
}

async fn store_tripit_credentials(
    pool: &SqlitePool,
    user_id: i64,
    username: &str,
    access_token: &str,
    access_token_secret: &str,
    encryption_key: &[u8; 32],
) -> Result<(), Error> {
    let (token_enc, nonce_token) = crate::auth::encrypt_token(access_token, encryption_key)?;
    let (secret_enc, nonce_secret) =
        crate::auth::encrypt_token(access_token_secret, encryption_key)?;

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

fn prompt_password() -> Result<String, Error> {
    print!("Password: ");
    io::stdout().flush()?;
    let password = rpassword::read_password()?;
    if password.is_empty() {
        return Err(Error::EmptyPassword);
    }
    print!("Confirm password: ");
    io::stdout().flush()?;
    let confirm = rpassword::read_password()?;
    if password != confirm {
        return Err(Error::PasswordMismatch);
    }
    Ok(password)
}

/// Create a new user interactively, prompting for username, email, and password.
///
/// # Errors
///
/// Returns an error if database access, encryption key parsing, or user creation fails.
pub async fn run(args: Args) -> Result<(), Error> {
    let pool = crate::db::create_pool(&args.database_url).await?;

    let encryption_key = args
        .encryption_key
        .as_deref()
        .map(parse_encryption_key)
        .transpose()?;

    let tripit_creds: Option<(&str, &str)> =
        match (&args.tripit_access_token, &args.tripit_access_token_secret) {
            (Some(token), Some(secret)) => Some((token.as_str(), secret.as_str())),
            (None, None) => None,
            _ => return Err(Error::IncompleteCredentials),
        };

    let username = args.username.as_str();
    let password = prompt_password()?;
    let email = prompt("Email")?;
    let first_name = prompt("First name")?;
    let last_name = prompt("Last name")?;

    let hash = crate::auth::hash_password(&password)?;

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
                .ok_or_else(|| Error::UserNotFound(username.to_owned()))?;
            user.id
        }
        Err(err) => return Err(Error::Database(err)),
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

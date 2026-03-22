use clap::Parser;
use sqlx::SqlitePool;

#[derive(Parser)]
#[command(about = "Seed the database with test users for local development")]
struct Cli {
    #[arg(long, env = "DATABASE_URL", default_value = "sqlite:travel.db")]
    database_url: String,

    /// Hex-encoded 32-byte AES-256-GCM key (64 hex chars). Required to seed
    /// TripIt credentials; ignored when absent.
    #[arg(long, env = "ENCRYPTION_KEY")]
    encryption_key: Option<String>,

    /// TripIt OAuth access token. Both token fields plus the encryption key
    /// must be set to seed credentials.
    #[arg(long, env = "TRIPIT_ACCESS_TOKEN")]
    tripit_access_token: Option<String>,

    /// TripIt OAuth access token secret.
    #[arg(long, env = "TRIPIT_ACCESS_TOKEN_SECRET")]
    tripit_access_token_secret: Option<String>,
}

const SEED_USERS: &[(&str, &str)] = &[("test", "test")];

fn parse_encryption_key(hex: &str) -> Result<[u8; 32], String> {
    if hex.len() != 64 {
        return Err("ENCRYPTION_KEY must be exactly 64 hex characters (32 bytes)".into());
    }
    let mut out = [0_u8; 32];
    for (idx, chunk) in hex.as_bytes().chunks(2).enumerate() {
        let pair =
            std::str::from_utf8(chunk).map_err(|_| "ENCRYPTION_KEY contains invalid UTF-8")?;
        out[idx] = u8::from_str_radix(pair, 16)
            .map_err(|_| "ENCRYPTION_KEY contains non-hex characters")?;
    }
    Ok(out)
}

async fn seed_tripit_credentials(
    pool: &SqlitePool,
    user_id: i64,
    username: &str,
    access_token: &str,
    access_token_secret: &str,
    encryption_key: &[u8; 32],
) -> Result<(), Box<dyn std::error::Error>> {
    let (token_enc, nonce_token) = travel_mapper::auth::encrypt_token(access_token, encryption_key)
        .map_err(|e| format!("failed to encrypt access token: {e}"))?;
    let (secret_enc, nonce_secret) =
        travel_mapper::auth::encrypt_token(access_token_secret, encryption_key)
            .map_err(|e| format!("failed to encrypt access token secret: {e}"))?;

    (travel_mapper::db::credentials::Upsert {
        user_id,
        access_token_enc: &token_enc,
        access_token_secret_enc: &secret_enc,
        nonce_token: &nonce_token,
        nonce_secret: &nonce_secret,
    })
    .execute(pool)
    .await?;

    println!("Stored TripIt credentials for user {username:?}");
    Ok(())
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let pool = travel_mapper::db::create_pool(&cli.database_url).await?;

    let encryption_key = cli
        .encryption_key
        .as_deref()
        .map(parse_encryption_key)
        .transpose()?;

    let tripit_creds: Option<(&str, &str)> = match (
        &cli.tripit_access_token,
        &cli.tripit_access_token_secret,
    ) {
        (Some(token), Some(secret)) => Some((token.as_str(), secret.as_str())),
        (None, None) => None,
        _ => {
            return Err(
                    "TRIPIT_ACCESS_TOKEN and TRIPIT_ACCESS_TOKEN_SECRET must both be set or both be unset".into(),
                );
        }
    };

    for &(username, password) in SEED_USERS {
        let hash = travel_mapper::auth::hash_password(password)
            .map_err(|e| format!("failed to hash password: {e}"))?;

        let user_id = match (travel_mapper::db::users::Create {
            username,
            password_hash: &hash,
        })
        .execute(&pool)
        .await
        {
            Ok(id) => {
                println!("Created user {username:?} (id={id}, password={password:?})");
                id
            }
            Err(sqlx::Error::Database(err)) if err.is_unique_violation() => {
                println!("User {username:?} already exists, skipping creation");
                let user = (travel_mapper::db::users::GetByUsername { username })
                    .execute(&pool)
                    .await?
                    .ok_or_else(|| format!("user {username:?} not found after unique violation"))?;
                user.id
            }
            Err(err) => return Err(err.into()),
        };

        if let (Some((access_token, access_token_secret)), Some(key)) =
            (tripit_creds, &encryption_key)
        {
            seed_tripit_credentials(
                &pool,
                user_id,
                username,
                access_token,
                access_token_secret,
                key,
            )
            .await?;
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    travel_mapper::telemetry::init();

    if let Err(error) = run().await {
        eprintln!("Error: {error}");
        std::process::exit(1);
    }
}

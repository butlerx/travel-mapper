use super::{
    crypto::{CryptoError, decrypt_token, encrypt_token},
    middleware::AuthUser,
    password::{hash_password, verify_password},
};
use crate::{db, routes::ErrorResponse, server::AppState, tripit::TripItConsumer};
use aide::transform::TransformOperation;
use axum::{
    Json,
    extract::{Query, State},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::{
    CookieJar, Host,
    cookie::{Cookie, SameSite},
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use rand::RngCore;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use uuid::Uuid;

#[derive(Deserialize, JsonSchema)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct ApiKeyRequest {
    pub label: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct TripItCredentialsRequest {
    pub access_token: String,
    pub access_token_secret: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct AuthResponse {
    pub id: i64,
    pub username: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ApiKeyResponse {
    pub id: i64,
    pub key: String,
    pub label: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct StatusResponse {
    pub status: String,
}

fn sha256_hex(value: &str) -> String {
    let digest = Sha256::digest(value.as_bytes());
    digest
        .iter()
        .fold(String::with_capacity(digest.len() * 2), |mut acc, byte| {
            use std::fmt::Write;
            let _ = write!(acc, "{byte:02x}");
            acc
        })
}

fn session_cookie(token: &str) -> Cookie<'static> {
    Cookie::build(("session_id", token.to_string()))
        .http_only(true)
        .same_site(SameSite::Lax)
        .path("/")
        .build()
}

fn clear_session_cookie() -> Cookie<'static> {
    Cookie::build(("session_id", String::new()))
        .http_only(true)
        .same_site(SameSite::Lax)
        .path("/")
        .build()
}

fn is_form_request(headers: &HeaderMap) -> bool {
    headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|ct| ct.starts_with("application/x-www-form-urlencoded"))
}

/// Register a new user account.
///
/// Accepts JSON or form-encoded body. On success, sets a session cookie.
pub async fn register_handler(
    State(state): State<AppState>,
    jar: CookieJar,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> (CookieJar, Response) {
    let parsed: Result<RegisterRequest, String> = if is_form_request(&headers) {
        serde_urlencoded::from_bytes(&body).map_err(|e| e.to_string())
    } else {
        serde_json::from_slice(&body).map_err(|e| e.to_string())
    };

    let body = match parsed {
        Ok(b) => b,
        Err(err) => {
            return if is_form_request(&headers) {
                (
                    jar,
                    Redirect::to("/register?error=Invalid+form+data").into_response(),
                )
            } else {
                (
                    jar,
                    (
                        StatusCode::BAD_REQUEST,
                        Json(json!({ "error": format!("invalid request body: {err}") })),
                    )
                        .into_response(),
                )
            };
        }
    };

    let is_form = is_form_request(&headers);

    let hash = match hash_password(&body.password) {
        Ok(hash) => hash,
        Err(err) => {
            return (
                jar,
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": format!("failed to hash password: {err}") })),
                )
                    .into_response(),
            );
        }
    };

    match db::create_user(&state.db, &body.username, &hash).await {
        Ok(id) => {
            let token = match create_user_session(&state.db, id).await {
                Ok((t, _)) => t,
                Err((status, msg)) => {
                    return (jar, (status, Json(json!({ "error": msg }))).into_response());
                }
            };

            let updated_jar = jar.add(session_cookie(&token));
            (
                updated_jar,
                if is_form {
                    Redirect::to("/dashboard").into_response()
                } else {
                    (
                        StatusCode::CREATED,
                        Json(json!({ "id": id, "username": body.username })),
                    )
                        .into_response()
                },
            )
        }
        Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => (
            jar,
            if is_form {
                Redirect::to("/register?error=Username+already+exists").into_response()
            } else {
                (
                    StatusCode::CONFLICT,
                    Json(json!({ "error": "username already exists" })),
                )
                    .into_response()
            },
        ),
        Err(err) => (
            jar,
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("failed to create user: {err}") })),
            )
                .into_response(),
        ),
    }
}

pub fn register_handler_docs(op: TransformOperation) -> TransformOperation {
    op.description("Register a new user account.")
        .response::<201, Json<AuthResponse>>()
        .response::<400, Json<ErrorResponse>>()
        .response::<409, Json<ErrorResponse>>()
        .response::<500, Json<ErrorResponse>>()
        .tag("auth")
}

async fn create_user_session(
    db: &sqlx::SqlitePool,
    user_id: i64,
) -> Result<(String, String), (StatusCode, String)> {
    let token = Uuid::new_v4().to_string();
    let expires_at =
        match sqlx::query_scalar::<_, Option<String>>("SELECT datetime('now', '+7 days')")
            .fetch_one(db)
            .await
        {
            Ok(Some(value)) => value,
            Ok(None) => {
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to generate session expiry".to_string(),
                ));
            }
            Err(err) => {
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("failed to generate session expiry: {err}"),
                ));
            }
        };

    db::create_session(db, &token, user_id, &expires_at)
        .await
        .map_err(|err| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed to create session: {err}"),
            )
        })?;

    Ok((token, expires_at))
}

async fn verify_credentials(
    db: &sqlx::SqlitePool,
    username: &str,
    password: &str,
) -> Result<db::UserRow, (StatusCode, String)> {
    let user = db::get_user_by_username(db, username)
        .await
        .map_err(|err| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed to lookup user: {err}"),
            )
        })?
        .ok_or((StatusCode::UNAUTHORIZED, "invalid credentials".to_string()))?;

    let verified = verify_password(password, &user.password_hash).map_err(|err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("failed to verify password: {err}"),
        )
    })?;

    if verified {
        Ok(user)
    } else {
        Err((StatusCode::UNAUTHORIZED, "invalid credentials".to_string()))
    }
}

/// Log in with username and password.
///
/// Accepts JSON or form-encoded body. On success, sets a session cookie.
pub async fn login_handler(
    State(state): State<AppState>,
    jar: CookieJar,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> (CookieJar, Response) {
    let parsed: Result<LoginRequest, String> = if is_form_request(&headers) {
        serde_urlencoded::from_bytes(&body).map_err(|e| e.to_string())
    } else {
        serde_json::from_slice(&body).map_err(|e| e.to_string())
    };

    let body = match parsed {
        Ok(b) => b,
        Err(err) => {
            return if is_form_request(&headers) {
                (
                    jar,
                    Redirect::to("/login?error=Invalid+form+data").into_response(),
                )
            } else {
                (
                    jar,
                    (
                        StatusCode::BAD_REQUEST,
                        Json(json!({ "error": format!("invalid request body: {err}") })),
                    )
                        .into_response(),
                )
            };
        }
    };

    let is_form = is_form_request(&headers);
    let user = match verify_credentials(&state.db, &body.username, &body.password).await {
        Ok(user) => user,
        Err((status, msg)) => {
            return (
                jar,
                if is_form && status == StatusCode::UNAUTHORIZED {
                    Redirect::to("/login?error=Invalid+credentials").into_response()
                } else {
                    (status, Json(json!({ "error": msg }))).into_response()
                },
            );
        }
    };

    let token = match create_user_session(&state.db, user.id).await {
        Ok((t, _)) => t,
        Err((status, msg)) => {
            return (jar, (status, Json(json!({ "error": msg }))).into_response());
        }
    };

    let updated_jar = jar.add(session_cookie(&token));
    (
        updated_jar,
        if is_form {
            Redirect::to("/dashboard").into_response()
        } else {
            (
                StatusCode::OK,
                Json(json!({ "id": user.id, "username": user.username })),
            )
                .into_response()
        },
    )
}

pub fn login_handler_docs(op: TransformOperation) -> TransformOperation {
    op.description("Log in with username and password.")
        .response::<200, Json<AuthResponse>>()
        .response::<400, Json<ErrorResponse>>()
        .response::<401, Json<ErrorResponse>>()
        .response::<500, Json<ErrorResponse>>()
        .tag("auth")
}

/// Log out and invalidate the current session.
pub async fn logout_handler(
    State(state): State<AppState>,
    jar: CookieJar,
    headers: HeaderMap,
    auth: AuthUser,
) -> (CookieJar, Response) {
    if let Some(cookie) = jar.get("session_id") {
        let _ = db::delete_session(&state.db, cookie.value()).await;
    }
    let _ = auth;

    let wants_html = headers
        .get(header::ACCEPT)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.contains("text/html"));

    let updated_jar = jar.remove(clear_session_cookie());
    (
        updated_jar,
        if wants_html {
            Redirect::to("/login").into_response()
        } else {
            (StatusCode::OK, Json(json!({ "status": "ok" }))).into_response()
        },
    )
}

pub fn logout_handler_docs(op: TransformOperation) -> TransformOperation {
    op.description("Log out and invalidate the current session.")
        .response::<200, Json<StatusResponse>>()
        .tag("auth")
}

/// Create a new API key for programmatic access.
pub async fn create_api_key_handler(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<ApiKeyRequest>,
) -> Response {
    let mut key_bytes = [0_u8; 32];
    rand::thread_rng().fill_bytes(&mut key_bytes);
    let key = URL_SAFE_NO_PAD.encode(key_bytes);
    let key_hash = sha256_hex(&key);
    let label = body.label.unwrap_or_default();

    match db::create_api_key(&state.db, auth.user_id, &key_hash, &label).await {
        Ok(id) => (
            StatusCode::OK,
            Json(json!({ "id": id, "key": key, "label": label })),
        )
            .into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("failed to create api key: {err}") })),
        )
            .into_response(),
    }
}

pub fn create_api_key_handler_docs(op: TransformOperation) -> TransformOperation {
    op.description("Create a new API key for programmatic access.")
        .response::<200, Json<ApiKeyResponse>>()
        .response::<500, Json<ErrorResponse>>()
        .tag("auth")
}

/// Store `TripIt` OAuth access tokens (encrypted at rest).
pub async fn store_tripit_credentials_handler(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<TripItCredentialsRequest>,
) -> Response {
    let token_encrypted = encrypt_token(&body.access_token, &state.encryption_key);
    let secret_encrypted = encrypt_token(&body.access_token_secret, &state.encryption_key);

    let (access_token_enc, nonce_token, access_token_secret_enc, nonce_secret) =
        match (token_encrypted, secret_encrypted) {
            (Ok((token_ct, token_nonce)), Ok((secret_ct, secret_nonce))) => {
                (token_ct, token_nonce, secret_ct, secret_nonce)
            }
            (Err(err), _) | (_, Err(err)) => {
                let message = match err {
                    CryptoError::Encrypt => "encryption failed".to_string(),
                    other => format!("failed to encrypt credentials: {other}"),
                };
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": message })),
                )
                    .into_response();
            }
        };

    match db::upsert_tripit_credentials(
        &state.db,
        auth.user_id,
        &access_token_enc,
        &access_token_secret_enc,
        &nonce_token,
        &nonce_secret,
    )
    .await
    {
        Ok(()) => (StatusCode::OK, Json(json!({ "status": "ok" }))).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("failed to store credentials: {err}") })),
        )
            .into_response(),
    }
}

pub fn store_tripit_credentials_handler_docs(op: TransformOperation) -> TransformOperation {
    op.description("Store TripIt OAuth access tokens (encrypted at rest).")
        .response::<200, Json<StatusResponse>>()
        .response::<500, Json<ErrorResponse>>()
        .tag("tripit")
}

#[derive(Deserialize, JsonSchema)]
pub struct TripItCallbackQuery {
    pub oauth_token: String,
}

/// Start `TripIt` OAuth flow — redirects to `TripIt` authorization page.
pub async fn tripit_connect_handler(
    State(state): State<AppState>,
    Host(host): Host,
    auth: AuthUser,
) -> Response {
    let consumer = TripItConsumer::new(
        state.tripit_consumer_key.clone(),
        state.tripit_consumer_secret.clone(),
    );

    let client = reqwest::Client::new();
    let request_token = match consumer.request_token(&client).await {
        Ok(pair) => pair,
        Err(err) => {
            tracing::error!("TripIt request_token failed: {err}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("failed to obtain request token: {err}") })),
            )
                .into_response();
        }
    };

    let (secret_enc, nonce) =
        match encrypt_token(&request_token.token_secret, &state.encryption_key) {
            Ok(pair) => pair,
            Err(err) => {
                tracing::error!("encrypt request token secret: {err}");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "encryption failed" })),
                )
                    .into_response();
            }
        };

    if let Err(err) = db::store_oauth_request_token(
        &state.db,
        &request_token.token,
        &secret_enc,
        &nonce,
        auth.user_id,
    )
    .await
    {
        tracing::error!("store oauth request token: {err}");
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("failed to store request token: {err}") })),
        )
            .into_response();
    }

    let scheme = if host.contains("localhost") || host.contains("127.0.0.1") {
        "http"
    } else {
        "https"
    };
    let callback_url = format!("{scheme}://{host}/auth/tripit/callback");
    let authorize_url = TripItConsumer::authorize_url(&request_token.token, &callback_url);

    Redirect::temporary(&authorize_url).into_response()
}

pub fn tripit_connect_handler_docs(op: TransformOperation) -> TransformOperation {
    op.description("Start TripIt OAuth flow — redirects to TripIt authorization page.")
        .response::<302, ()>()
        .response::<500, Json<ErrorResponse>>()
        .tag("tripit")
}

/// `TripIt` OAuth callback — exchanges request token for access token.
pub async fn tripit_callback_handler(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(query): Query<TripItCallbackQuery>,
) -> Response {
    let stored = match db::get_oauth_request_token(&state.db, &query.oauth_token).await {
        Ok(Some(row)) => row,
        Ok(None) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "unknown or expired oauth_token" })),
            )
                .into_response();
        }
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("failed to lookup request token: {err}") })),
            )
                .into_response();
        }
    };

    if stored.user_id != auth.user_id {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "request token belongs to another user" })),
        )
            .into_response();
    }

    let token_secret = match decrypt_token(
        &stored.token_secret_enc,
        &stored.nonce,
        &state.encryption_key,
    ) {
        Ok(secret) => secret,
        Err(err) => {
            tracing::error!("decrypt request token secret: {err}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "failed to decrypt request token secret" })),
            )
                .into_response();
        }
    };

    let _ = db::delete_oauth_request_token(&state.db, &query.oauth_token).await;

    let consumer = TripItConsumer::new(
        state.tripit_consumer_key.clone(),
        state.tripit_consumer_secret.clone(),
    );

    let request_pair = crate::tripit::OAuthTokenPair {
        token: query.oauth_token.clone(),
        token_secret,
    };

    let client = reqwest::Client::new();
    let access_token = match consumer.access_token(&client, &request_pair).await {
        Ok(pair) => pair,
        Err(err) => {
            tracing::error!("TripIt access_token exchange failed: {err}");
            return Redirect::to("/settings?error=TripIt+authorization+failed").into_response();
        }
    };

    let token_encrypted = encrypt_token(&access_token.token, &state.encryption_key);
    let secret_encrypted = encrypt_token(&access_token.token_secret, &state.encryption_key);

    let (access_token_enc, nonce_token, access_token_secret_enc, nonce_secret) =
        match (token_encrypted, secret_encrypted) {
            (Ok((token_ct, token_nonce)), Ok((secret_ct, secret_nonce))) => {
                (token_ct, token_nonce, secret_ct, secret_nonce)
            }
            (Err(err), _) | (_, Err(err)) => {
                tracing::error!("encrypt access token: {err}");
                return Redirect::to("/settings?error=Failed+to+store+credentials").into_response();
            }
        };

    if let Err(err) = db::upsert_tripit_credentials(
        &state.db,
        auth.user_id,
        &access_token_enc,
        &access_token_secret_enc,
        &nonce_token,
        &nonce_secret,
    )
    .await
    {
        tracing::error!("upsert tripit credentials: {err}");
        return Redirect::to("/settings?error=Failed+to+store+credentials").into_response();
    }

    Redirect::to("/settings?tripit=connected").into_response()
}

pub fn tripit_callback_handler_docs(op: TransformOperation) -> TransformOperation {
    op.description("TripIt OAuth callback — exchanges request token for access token.")
        .response::<302, ()>()
        .response::<400, Json<ErrorResponse>>()
        .response::<403, Json<ErrorResponse>>()
        .response::<500, Json<ErrorResponse>>()
        .tag("tripit")
}

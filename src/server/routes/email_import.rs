//! Inbound email webhook for booking confirmation import.
//!
//! Accepts forwarded booking confirmation emails from providers like SendGrid or
//! Mailgun, extracts travel details, and creates journey records.

use super::ics_import::create_journeys;
use crate::{
    db,
    integrations::email_import,
    server::{AppState, extractors::AuthUser, session::sha256_hex},
};
use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};
use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};
use serde::Deserialize;
use tracing::{info, warn};

const MAX_EMAIL_BYTES: usize = 25 * 1024 * 1024;

#[derive(Debug, Deserialize)]
pub struct GenerateTokenForm {
    pub label: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DeleteTokenForm {
    pub id: i64,
}

/// Public webhook endpoint for inbound email providers (SendGrid/Mailgun).
///
/// Accepts `multipart/form-data` with provider-specific fields. Identifies the
/// user by matching the recipient address token against `inbound_email_tokens`.
pub async fn webhook_handler(State(state): State<AppState>, mut multipart: Multipart) -> Response {
    let mut raw_email: Option<Vec<u8>> = None;
    let mut recipient: Option<String> = None;
    let mut html_body: Option<String> = None;

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            // SendGrid raw MIME in "email" field
            "email" => {
                if let Ok(bytes) = field.bytes().await
                    && bytes.len() <= MAX_EMAIL_BYTES
                {
                    raw_email = Some(bytes.to_vec());
                }
            }
            // Recipient address for user identification
            "to" | "recipient" => {
                if let Ok(text) = field.text().await {
                    recipient = Some(text);
                }
            }
            // Pre-parsed HTML body (SendGrid/Mailgun)
            "html" | "body-html" => {
                if let Ok(text) = field.text().await {
                    html_body = Some(text);
                }
            }
            _ => {}
        }
    }

    let Some(ref to_addr) = recipient else {
        warn!("email webhook: no recipient address");
        return (StatusCode::BAD_REQUEST, "missing recipient").into_response();
    };

    let Some(user_id) = resolve_user_from_recipient(&state, to_addr).await else {
        warn!(to = %to_addr, "email webhook: unknown recipient token");
        return (StatusCode::NOT_FOUND, "unknown recipient").into_response();
    };

    let journeys = if let Some(raw) = raw_email {
        email_import::parse_email(&raw)
    } else if let Some(html) = html_body {
        email_import::extract_json_ld(&html)
    } else {
        warn!("email webhook: no email content in payload");
        return (StatusCode::BAD_REQUEST, "missing email content").into_response();
    };

    if journeys.is_empty() {
        info!(user_id, "email webhook: no travel bookings found in email");
        return (StatusCode::OK, "no bookings found").into_response();
    }

    match create_journeys(&state.db, user_id, journeys).await {
        Ok(count) => {
            info!(user_id, count, "email webhook: imported journeys");
            (StatusCode::OK, format!("imported {count} journey(s)")).into_response()
        }
        Err(e) => {
            tracing::error!(user_id, error = %e, "email webhook: DB error creating journeys");
            (StatusCode::INTERNAL_SERVER_ERROR, "import failed").into_response()
        }
    }
}

/// Authenticated endpoint to manually submit a raw email for import.
///
/// Accepts `Content-Type: message/rfc822` (raw email bytes) or `text/plain`.
pub async fn manual_handler(
    State(state): State<AppState>,
    auth: AuthUser,
    body: axum::body::Bytes,
) -> Response {
    if body.len() > MAX_EMAIL_BYTES {
        return (StatusCode::PAYLOAD_TOO_LARGE, "email too large").into_response();
    }

    let journeys = email_import::parse_email(&body);

    if journeys.is_empty() {
        return (StatusCode::OK, "no bookings found in email").into_response();
    }

    match create_journeys(&state.db, auth.user_id, journeys).await {
        Ok(count) => {
            info!(
                user_id = auth.user_id,
                count, "manual email import: created journeys"
            );
            (StatusCode::OK, format!("imported {count} journey(s)")).into_response()
        }
        Err(e) => {
            tracing::error!(user_id = auth.user_id, error = %e, "manual email import: DB error");
            (StatusCode::INTERNAL_SERVER_ERROR, "import failed").into_response()
        }
    }
}

/// Generate a new inbound email token for the authenticated user.
pub async fn generate_token_handler(
    State(state): State<AppState>,
    auth: AuthUser,
    axum::extract::Form(form): axum::extract::Form<GenerateTokenForm>,
) -> Response {
    let raw_token = uuid::Uuid::new_v4().to_string();
    let token_hash = sha256_hex(&raw_token);
    let label = form.label.as_deref().unwrap_or("email import");

    match (db::inbound_email_tokens::Create {
        user_id: auth.user_id,
        token_hash: &token_hash,
        label,
    })
    .execute(&state.db)
    .await
    {
        Ok(_) => {
            let encoded = utf8_percent_encode(&raw_token, NON_ALPHANUMERIC);
            Redirect::to(&format!("/settings?email_token={encoded}")).into_response()
        }
        Err(e) => {
            let msg = format!("Failed to generate token: {e}");
            let encoded = utf8_percent_encode(&msg, NON_ALPHANUMERIC);
            Redirect::to(&format!("/settings?error={encoded}")).into_response()
        }
    }
}

/// Delete an inbound email token.
pub async fn delete_token_handler(
    State(state): State<AppState>,
    auth: AuthUser,
    axum::extract::Form(form): axum::extract::Form<DeleteTokenForm>,
) -> Response {
    let _ = (db::inbound_email_tokens::Delete {
        id: form.id,
        user_id: auth.user_id,
    })
    .execute(&state.db)
    .await;

    Redirect::to("/settings").into_response()
}

/// Extract the token from a recipient address like `import-{token}@domain.com`
/// or `{token}@domain.com`, hash it, and look up the owning user.
async fn resolve_user_from_recipient(state: &AppState, to_addr: &str) -> Option<i64> {
    // Strip angle brackets and extract the local part
    let addr = to_addr.trim().trim_start_matches('<').trim_end_matches('>');
    let local_part = addr.split('@').next()?;

    // Support formats: "import-{token}" or bare "{token}"
    let token = local_part.strip_prefix("import-").unwrap_or(local_part);

    let token_hash = sha256_hex(token);
    (db::inbound_email_tokens::GetUserIdByHash {
        token_hash: &token_hash,
    })
    .execute(&state.db)
    .await
    .ok()?
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::tests::{test_pool, test_user};
    use crate::server::test_helpers::test_app_state;

    #[tokio::test]
    async fn resolve_user_from_recipient_with_prefix() {
        let pool = test_pool().await;
        let state = test_app_state(pool.clone());
        let user_id = test_user(&pool, "alice").await;

        let raw_token = "test-uuid-1234";
        let token_hash = sha256_hex(raw_token);
        db::inbound_email_tokens::Create {
            user_id,
            token_hash: &token_hash,
            label: "test",
        }
        .execute(&pool)
        .await
        .expect("create token");

        // With prefix
        let addr = format!("import-{raw_token}@travel.example.com");
        let resolved = resolve_user_from_recipient(&state, &addr).await;
        assert_eq!(resolved, Some(user_id));

        // Bare token
        let addr2 = format!("{raw_token}@travel.example.com");
        let resolved2 = resolve_user_from_recipient(&state, &addr2).await;
        assert_eq!(resolved2, Some(user_id));

        // With angle brackets
        let addr3 = format!("<import-{raw_token}@travel.example.com>");
        let resolved3 = resolve_user_from_recipient(&state, &addr3).await;
        assert_eq!(resolved3, Some(user_id));

        // Unknown token
        let resolved4 = resolve_user_from_recipient(&state, "unknown@example.com").await;
        assert_eq!(resolved4, None);
    }

    #[tokio::test]
    async fn manual_import_parses_and_creates() {
        let pool = test_pool().await;
        let user_id = test_user(&pool, "alice").await;

        let raw_email = b"From: bookings@airline.com\r\n\
Subject: Booking Confirmed\r\n\
Content-Type: text/html; charset=utf-8\r\n\
\r\n\
<html><body>\
<script type=\"application/ld+json\">\
{\"@type\":\"FlightReservation\",\"reservationFor\":{\"@type\":\"Flight\",\
\"departureAirport\":{\"iataCode\":\"DUB\"},\"departureTime\":\"2027-06-01T07:00:00Z\",\
\"arrivalAirport\":{\"iataCode\":\"LHR\"},\"provider\":{\"name\":\"Aer Lingus\"}}}\
</script>\
</body></html>";

        let journeys = email_import::parse_email(raw_email);
        assert_eq!(journeys.len(), 1);

        let count = create_journeys(&pool, user_id, journeys)
            .await
            .expect("import failed");
        assert_eq!(count, 1);

        // Reimport is idempotent
        let journeys2 = email_import::parse_email(raw_email);
        let count2 = create_journeys(&pool, user_id, journeys2)
            .await
            .expect("reimport failed");
        assert_eq!(count2, 0);
    }
}

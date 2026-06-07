//! Import journeys from a user-supplied iCalendar (`.ics`) URL.

use crate::{
    db::{
        self,
        hops::{BoatDetail, FlightDetail, ManualDetail, RailDetail, TransportDetail, TravelType},
    },
    integrations::ics_import::{ParsedJourney, parse_ics},
    server::{AppState, extractors::AuthUser, extractors::FormOrJson},
};
use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
};
use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};
use serde::Deserialize;
use std::collections::HashSet;
use std::time::Duration;

/// Cap on the fetched calendar body to avoid unbounded memory use.
const MAX_ICS_BYTES: usize = 5 * 1024 * 1024;
const FETCH_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Default, Deserialize)]
pub struct ImportIcsForm {
    pub url: String,
}

pub async fn handler(
    State(state): State<AppState>,
    auth: AuthUser,
    FormOrJson(form): FormOrJson<ImportIcsForm>,
) -> Response {
    match import_from_url(&state, auth.user_id, &form.url).await {
        Ok(count) => Redirect::to(&format!("/settings?ics={count}")).into_response(),
        Err(msg) => {
            let encoded = utf8_percent_encode(&msg, NON_ALPHANUMERIC);
            Redirect::to(&format!("/settings?error={encoded}")).into_response()
        }
    }
}

/// Normalise a calendar URL: accept `http(s)` and map `webcal://` (the common
/// calendar-subscription scheme) to `https://`.
fn normalize_url(raw: &str) -> Result<String, String> {
    let trimmed = raw.trim();
    if let Some(rest) = trimmed.strip_prefix("webcal://") {
        return Ok(format!("https://{rest}"));
    }
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        return Ok(trimmed.to_owned());
    }
    Err("Calendar URL must start with http://, https://, or webcal://".to_owned())
}

async fn fetch_ics(url: &str) -> Result<String, String> {
    // NOTE: this fetches a user-supplied URL server-side. For this self-hosted,
    // single-tenant app that's acceptable; a multi-tenant deployment should add
    // SSRF protection (block private/loopback address ranges).
    let client = reqwest::Client::builder()
        .timeout(FETCH_TIMEOUT)
        .build()
        .map_err(|e| format!("could not build HTTP client: {e}"))?;
    let resp = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("could not fetch calendar: {e}"))?;
    if !resp.status().is_success() {
        return Err(format!("calendar URL returned HTTP {}", resp.status()));
    }
    let max_bytes = u64::try_from(MAX_ICS_BYTES).unwrap_or(u64::MAX);
    if resp.content_length().is_some_and(|len| len > max_bytes) {
        return Err("calendar file is too large".to_owned());
    }
    let text = resp
        .text()
        .await
        .map_err(|e| format!("could not read calendar: {e}"))?;
    if text.len() > MAX_ICS_BYTES {
        return Err("calendar file is too large".to_owned());
    }
    Ok(text)
}

async fn import_from_url(state: &AppState, user_id: i64, raw_url: &str) -> Result<u64, String> {
    let url = normalize_url(raw_url)?;
    let text = fetch_ics(&url).await?;
    let journeys = parse_ics(&text);
    if journeys.is_empty() {
        return Err("No importable journeys found in that calendar".to_owned());
    }
    create_journeys(&state.db, user_id, journeys)
        .await
        .map_err(|e| format!("database error: {e}"))
}

/// Create the parsed journeys, skipping any that duplicate an existing hop with
/// the same date and route (so re-running an import is idempotent).
async fn create_journeys(
    pool: &sqlx::SqlitePool,
    user_id: i64,
    journeys: Vec<ParsedJourney>,
) -> Result<u64, sqlx::Error> {
    let existing = (db::hops::GetAll {
        user_id,
        travel_type_filter: None,
    })
    .execute(pool)
    .await?;
    let mut seen: HashSet<(String, String, String)> = existing
        .iter()
        .map(|h| {
            (
                h.start_date.clone(),
                h.origin_name.clone(),
                h.dest_name.clone(),
            )
        })
        .collect();

    let mut imported = 0_u64;
    for journey in journeys {
        let key = (
            journey.date.clone(),
            journey.origin.clone(),
            journey.destination.clone(),
        );
        if !seen.insert(key) {
            continue;
        }
        db::hops::CreateManual {
            user_id,
            origin: journey.origin,
            destination: journey.destination,
            date: journey.date,
            detail: detail_for(&journey.travel_type, journey.carrier),
            cost_amount: None,
            cost_currency: None,
            loyalty_program: None,
            miles_earned: None,
        }
        .execute(pool)
        .await?;
        imported += 1;
    }
    Ok(imported)
}

fn detail_for(travel_type: &TravelType, carrier: String) -> ManualDetail {
    match travel_type {
        TravelType::Air => ManualDetail::Air(FlightDetail {
            airline: carrier,
            ..Default::default()
        }),
        TravelType::Rail => ManualDetail::Rail(RailDetail {
            carrier,
            ..Default::default()
        }),
        TravelType::Boat => ManualDetail::Boat(BoatDetail {
            ship_name: carrier,
            ..Default::default()
        }),
        TravelType::Transport => ManualDetail::Transport(TransportDetail {
            carrier_name: carrier,
            ..Default::default()
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{
        hops::{GetAll, TravelType},
        tests::{test_pool, test_user},
    };

    #[test]
    fn normalize_url_maps_webcal_and_rejects_other_schemes() {
        assert_eq!(
            normalize_url("webcal://example.com/cal.ics").unwrap(),
            "https://example.com/cal.ics"
        );
        assert_eq!(
            normalize_url("https://example.com/cal.ics").unwrap(),
            "https://example.com/cal.ics"
        );
        assert!(normalize_url("ftp://example.com/cal.ics").is_err());
        assert!(normalize_url("file:///etc/passwd").is_err());
    }

    #[tokio::test]
    async fn create_journeys_imports_and_dedups() {
        let pool = test_pool().await;
        let user_id = test_user(&pool, "alice").await;

        let parsed = vec![
            ParsedJourney {
                date: "2024-06-01".to_owned(),
                origin: "DUB".to_owned(),
                destination: "LHR".to_owned(),
                travel_type: TravelType::Air,
                carrier: "BA".to_owned(),
            },
            // Duplicate of the first — should be skipped.
            ParsedJourney {
                date: "2024-06-01".to_owned(),
                origin: "DUB".to_owned(),
                destination: "LHR".to_owned(),
                travel_type: TravelType::Air,
                carrier: "BA".to_owned(),
            },
            ParsedJourney {
                date: "2024-07-02".to_owned(),
                origin: "London Euston".to_owned(),
                destination: "Manchester".to_owned(),
                travel_type: TravelType::Rail,
                carrier: "Avanti".to_owned(),
            },
        ];

        let imported = create_journeys(&pool, user_id, parsed)
            .await
            .expect("import failed");
        assert_eq!(imported, 2, "duplicate should be skipped");

        // Re-importing the same set creates nothing new.
        let again = create_journeys(
            &pool,
            user_id,
            vec![ParsedJourney {
                date: "2024-06-01".to_owned(),
                origin: "DUB".to_owned(),
                destination: "LHR".to_owned(),
                travel_type: TravelType::Air,
                carrier: "BA".to_owned(),
            }],
        )
        .await
        .expect("reimport failed");
        assert_eq!(again, 0, "existing journey should be skipped on reimport");

        let all = GetAll {
            user_id,
            travel_type_filter: None,
        }
        .execute(&pool)
        .await
        .expect("fetch failed");
        assert_eq!(all.len(), 2);
    }
}

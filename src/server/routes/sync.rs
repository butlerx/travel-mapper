use super::{ErrorResponse, MultiFormatResponse, multi_format_docs, negotiate_format};
use crate::{
    db,
    geocode::Geocoder,
    server::{AppState, error::AppError, extractors::AuthUser},
    worker::{SyncOutcome, sync_all},
};
use aide::transform::TransformOperation;
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Redirect, Response},
};
use schemars::JsonSchema;
use serde::Serialize;

/// Minimum interval between user-triggered syncs (15 minutes).
const SYNC_COOLDOWN_SECS: i64 = 15 * 60;

/// Response returned when a sync completes immediately.
#[derive(Debug, Default, Serialize, JsonSchema)]
pub struct SyncResponse {
    /// Number of trips fetched from `TripIt`.
    pub trips_fetched: u64,
    /// Number of individual travel journeys extracted.
    #[serde(rename = "journeys_fetched")]
    pub hops_fetched: u64,
    /// Wall-clock duration of the sync in milliseconds.
    pub duration_ms: u64,
}

impl From<SyncOutcome> for SyncResponse {
    fn from(outcome: SyncOutcome) -> Self {
        Self {
            trips_fetched: outcome.trips_fetched,
            hops_fetched: outcome.hops_fetched,
            duration_ms: outcome.duration_ms,
        }
    }
}

impl MultiFormatResponse for SyncResponse {
    const HTML_TITLE: &'static str = "Sync Result";
    const CSV_HEADERS: &'static [&'static str] =
        &["trips_fetched", "journeys_fetched", "duration_ms"];

    fn csv_row(&self) -> Vec<String> {
        vec![
            self.trips_fetched.to_string(),
            self.hops_fetched.to_string(),
            self.duration_ms.to_string(),
        ]
    }
}

/// Response returned when a sync job is enqueued for background processing.
#[derive(Debug, Default, Serialize, JsonSchema)]
pub struct SyncQueuedResponse {
    /// Status message, e.g. `"sync queued"`.
    pub status: String,
    /// Identifier of the enqueued sync job.
    pub job_id: i64,
}

impl MultiFormatResponse for SyncQueuedResponse {
    const HTML_TITLE: &'static str = "Sync Queued";
    const CSV_HEADERS: &'static [&'static str] = &["status", "job_id"];

    fn csv_row(&self) -> Vec<String> {
        vec![self.status.clone(), self.job_id.to_string()]
    }
}

fn is_form_request(headers: &HeaderMap) -> bool {
    headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|ct| ct.starts_with("application/x-www-form-urlencoded"))
}

pub async fn handler(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
) -> Response {
    let is_form = is_form_request(&headers);

    let format = negotiate_format(&headers);

    if let Some(override_api) = &state.tripit_override {
        let geocoder = Geocoder::new(state.db.clone());
        let result = sync_all(override_api.as_ref(), &geocoder, &state.db, auth.user_id).await;
        return match result {
            Ok(r) => {
                let response = SyncResponse::from(r);
                SyncResponse::single_format_response(&response, format, StatusCode::OK)
            }
            Err(err) => AppError::from(err).into_format_response(format),
        };
    }

    match (db::sync_state::GetOrCreate {
        user_id: auth.user_id,
    })
    .execute(&state.db)
    .await
    {
        Ok(state_row) => {
            if let Some(ref last_sync) = state_row.last_sync_at {
                let too_soon = sqlx::query_scalar!(
                    "SELECT (strftime('%s', datetime('now')) - strftime('%s', ?)) < ?",
                    last_sync,
                    SYNC_COOLDOWN_SECS,
                )
                .fetch_one(&state.db)
                .await;
                if matches!(too_soon, Ok(1)) {
                    return if is_form {
                        Redirect::to("/dashboard?error=Please+wait+before+syncing+again")
                            .into_response()
                    } else {
                        AppError::TooManyRequests(
                            "sync cooldown active — please wait 15 minutes between syncs"
                                .to_string(),
                        )
                        .into_format_response(format)
                    };
                }
            }
        }
        Err(err) => {
            return AppError::from(err).into_format_response(format);
        }
    }

    match (db::sync_jobs::HasPendingOrRunning {
        user_id: auth.user_id,
    })
    .execute(&state.db)
    .await
    {
        Ok(true) => {
            return if is_form {
                Redirect::to("/dashboard?error=Sync+already+queued").into_response()
            } else {
                AppError::Conflict("sync already queued or running").into_format_response(format)
            };
        }
        Ok(false) => {}
        Err(err) => {
            return AppError::from(err).into_format_response(format);
        }
    }

    match (db::sync_jobs::Enqueue {
        user_id: auth.user_id,
    })
    .execute(&state.db)
    .await
    {
        Ok(job_id) => {
            tracing::info!(user_id = auth.user_id, job_id, "sync job enqueued");
            if is_form {
                Redirect::to("/dashboard").into_response()
            } else {
                let response = SyncQueuedResponse {
                    status: "sync queued".to_string(),
                    job_id,
                };
                SyncQueuedResponse::single_format_response(&response, format, StatusCode::ACCEPTED)
            }
        }
        Err(err) => AppError::from(err).into_format_response(format),
    }
}

/// `OpenAPI` metadata for the sync endpoint.
pub fn handler_docs(op: TransformOperation) -> TransformOperation {
    multi_format_docs!(
        op.description("Trigger a TripIt sync for the authenticated user."),
        200 => SyncResponse,
        202 => SyncQueuedResponse,
        401 | 409 | 429 | 500 => ErrorResponse,
    )
    .tag("sync")
}

#[cfg(test)]
mod tests {
    use crate::{db, server::create_router, server::test_helpers::*};
    use axum::{
        body::Body,
        http::{Request, StatusCode, header},
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn post_sync_fetches_and_stores_hops() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let mut state = test_app_state(pool.clone());
        state.tripit_override = Some(mock_tripit_api_with_data());

        let app = create_router(state);
        let sync_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/sync")
                    .header(header::COOKIE, cookie.clone())
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");
        assert_eq!(sync_response.status(), StatusCode::OK);

        let sync_body = axum::body::to_bytes(sync_response.into_body(), usize::MAX)
            .await
            .expect("failed to read response body");
        let sync_json: serde_json::Value =
            serde_json::from_slice(&sync_body).expect("body should be valid JSON");
        assert_eq!(sync_json["trips_fetched"], serde_json::json!(2));

        let hops_response = app
            .oneshot(
                Request::builder()
                    .uri("/journeys")
                    .header(header::COOKIE, cookie)
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");
        assert_eq!(hops_response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn post_sync_updates_sync_state() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let user = db::users::GetByUsername { username: "alice" }
            .execute(&pool)
            .await
            .expect("lookup failed")
            .expect("missing user");

        let mut state = test_app_state(pool.clone());
        state.tripit_override = Some(mock_tripit_api_with_data());

        let app = create_router(state);
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/sync")
                    .header(header::COOKIE, cookie)
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);

        let sync_state = db::sync_state::GetOrCreate { user_id: user.id }
            .execute(&pool)
            .await
            .expect("failed to fetch sync state");
        assert!(sync_state.last_sync_at.is_some());
        assert_eq!(sync_state.sync_status, "idle");
    }
}

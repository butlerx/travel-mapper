use crate::{
    db,
    server::{AppState, middleware::AuthUser, routes::types::ErrorResponse},
    worker::{SyncResult, sync_all},
};
use aide::transform::TransformOperation;
use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Redirect, Response},
};
use schemars::JsonSchema;
use serde::Serialize;
use serde_json::json;

#[derive(Debug, Serialize, JsonSchema)]
pub struct SyncQueuedResponse {
    pub status: String,
    pub job_id: i64,
}

fn is_form_request(headers: &HeaderMap) -> bool {
    headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|ct| ct.starts_with("application/x-www-form-urlencoded"))
}

pub async fn sync_handler(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
) -> Response {
    let is_form = is_form_request(&headers);

    if let Some(override_api) = &state.tripit_override {
        let result = sync_all(override_api.as_ref(), &state.db, auth.user_id).await;
        return match result {
            Ok(r) => (StatusCode::OK, Json(json!(r))).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("sync failed: {err}") })),
            )
                .into_response(),
        };
    }

    match db::has_pending_or_running_sync_job(&state.db, auth.user_id).await {
        Ok(true) => {
            return if is_form {
                Redirect::to("/dashboard?error=Sync+already+queued").into_response()
            } else {
                (
                    StatusCode::CONFLICT,
                    Json(json!({ "error": "sync already queued or running" })),
                )
                    .into_response()
            };
        }
        Ok(false) => {}
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("database error: {err}") })),
            )
                .into_response();
        }
    }

    match db::enqueue_sync_job(&state.db, auth.user_id).await {
        Ok(job_id) => {
            tracing::info!(user_id = auth.user_id, job_id, "sync job enqueued");
            if is_form {
                Redirect::to("/dashboard").into_response()
            } else {
                (
                    StatusCode::ACCEPTED,
                    Json(json!({ "status": "sync queued", "job_id": job_id })),
                )
                    .into_response()
            }
        }
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("failed to enqueue sync: {err}") })),
        )
            .into_response(),
    }
}

pub fn sync_handler_docs(op: TransformOperation) -> TransformOperation {
    op.description("Trigger a TripIt sync for the authenticated user.")
        .response::<200, Json<SyncResult>>()
        .response::<202, Json<SyncQueuedResponse>>()
        .response::<409, Json<ErrorResponse>>()
        .response::<500, Json<ErrorResponse>>()
        .tag("sync")
}

#[cfg(test)]
mod tests {
    use crate::{db, server::create_router, server::test_helpers::helpers::*};
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
                    .uri("/hops")
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
        let user = db::get_user_by_username(&pool, "alice")
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

        let sync_state = db::get_or_create_sync_state(&pool, user.id)
            .await
            .expect("failed to fetch sync state");
        assert!(sync_state.last_sync_at.is_some());
        assert_eq!(sync_state.sync_status, "idle");
    }
}

use crate::server::AppState;
use aide::{axum::IntoApiResponse, transform::TransformOperation};
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use schemars::JsonSchema;
use serde::Serialize;
use serde_json::json;

#[derive(Debug, Serialize, JsonSchema)]
pub struct HealthResponse {
    pub status: String,
    pub last_sync: Option<String>,
}

pub async fn health_handler(State(state): State<AppState>) -> impl IntoApiResponse {
    let last_sync =
        sqlx::query_scalar::<_, Option<String>>("SELECT MAX(last_sync_at) FROM sync_state")
            .fetch_one(&state.db)
            .await
            .ok()
            .flatten();

    let body = json!({
        "status": "ok",
        "last_sync": last_sync,
    });

    (StatusCode::OK, Json(body)).into_response()
}

pub fn health_handler_docs(op: TransformOperation) -> TransformOperation {
    op.description("Check server health and last sync timestamp.")
        .response::<200, Json<HealthResponse>>()
        .tag("health")
}

#[cfg(test)]
mod tests {
    use crate::server::create_router;
    use crate::server::test_helpers::helpers::*;
    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn get_health_returns_ok_with_null_last_sync() {
        let pool = test_pool().await;
        let state = test_app_state(pool);

        let app = create_router(state);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("failed to read response body");
        let body_text = String::from_utf8(body.to_vec()).expect("response body should be UTF-8");

        assert!(body_text.contains("\"status\":\"ok\""));
        assert!(body_text.contains("\"last_sync\":null"));
    }
}

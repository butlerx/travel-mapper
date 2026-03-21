use super::types::{MultiFormatResponse, multi_format_docs, negotiate_format};
use crate::server::AppState;
use aide::{axum::IntoApiResponse, transform::TransformOperation};
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use schemars::JsonSchema;
use serde::Serialize;

#[derive(Debug, Default, Serialize, JsonSchema)]
pub struct HealthResponse {
    pub status: String,
    pub last_sync: Option<String>,
}

impl MultiFormatResponse for HealthResponse {
    const HTML_TITLE: &'static str = "Health Check";
    const CSV_HEADERS: &'static [&'static str] = &["status", "last_sync"];

    fn csv_row(&self) -> Vec<String> {
        vec![
            self.status.clone(),
            self.last_sync.clone().unwrap_or_default(),
        ]
    }
}

pub async fn health_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoApiResponse {
    let last_sync =
        sqlx::query_scalar::<_, Option<String>>("SELECT MAX(last_sync_at) FROM sync_state")
            .fetch_one(&state.db)
            .await
            .ok()
            .flatten();

    let response = HealthResponse {
        status: "ok".to_string(),
        last_sync,
    };

    let format = negotiate_format(&headers);
    HealthResponse::single_format_response(&response, format, StatusCode::OK).into_response()
}

pub fn health_handler_docs(op: TransformOperation) -> TransformOperation {
    multi_format_docs!(
        op.description("Check server health and last sync timestamp."),
        200 => HealthResponse,
    )
    .tag("health")
}

#[cfg(test)]
mod tests {
    use crate::server::create_router;
    use crate::server::test_helpers::helpers::*;
    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode, header},
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

    #[tokio::test]
    async fn get_health_csv_returns_csv() {
        let pool = test_pool().await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .header(header::ACCEPT, "text/csv")
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_text(response).await;
        assert!(body.contains("status,last_sync"));
        assert!(body.contains("ok"));
    }

    #[tokio::test]
    async fn get_health_html_returns_html_table() {
        let pool = test_pool().await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .header(header::ACCEPT, "text/html")
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_text(response).await;
        assert!(body.contains("data-card"));
        assert!(body.contains("Health Check"));
        assert!(body.contains("ok"));
    }
}

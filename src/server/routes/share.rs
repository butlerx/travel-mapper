use super::{MultiFormatResponse, ResponseFormat, negotiate_format};
use crate::{
    db,
    server::{
        AppState,
        pages::stats::{StatsQuery, compute_detailed_stats},
        session::sha256_hex,
    },
};
use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};

pub async fn handler(
    State(state): State<AppState>,
    Path(token): Path<String>,
    Query(query): Query<StatsQuery>,
    headers: HeaderMap,
) -> Response {
    let token_hash = sha256_hex(&token);

    let user_id = match (db::share_tokens::GetUserIdByHash {
        token_hash: &token_hash,
    })
    .execute(&state.db)
    .await
    {
        Ok(Some(id)) => id,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(err) => {
            tracing::error!(error = %err, "share token lookup failed");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let all_rows = match (db::hops::GetAllForStats { user_id })
        .execute(&state.db)
        .await
    {
        Ok(rows) => rows,
        Err(err) => {
            tracing::error!(error = %err, "failed to fetch stats for share page");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let detailed = compute_detailed_stats(&all_rows, query.year.as_deref());

    let format = negotiate_format(&headers);
    if format == ResponseFormat::Html {
        crate::server::pages::stats::render_share_page(detailed, &token)
    } else {
        let response = super::stats::StatsResponse::from(detailed);
        super::stats::StatsResponse::single_format_response(&response, format, StatusCode::OK)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        db::{
            self,
            hops::{Create, FlightDetail, TravelType},
        },
        server::{create_router, session::sha256_hex, test_helpers::*},
    };
    use axum::{
        body::Body,
        http::{Request, StatusCode, header},
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn share_page_returns_404_for_invalid_token() {
        let pool = test_pool().await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/share/nonexistent-token")
                    .header(header::ACCEPT, "text/html")
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn share_page_renders_stats_for_valid_token() {
        let pool = test_pool().await;
        let user_id = db::tests::test_user(&pool, "alice").await;

        let mut journey = sample_hop(TravelType::Air, "DUB", "LHR", "2024-06-15", "2024-06-15");
        journey.flight_detail = Some(FlightDetail {
            airline: "Aer Lingus".to_string(),
            aircraft_type: "A320".to_string(),
            ..Default::default()
        });
        Create {
            trip_id: "trip-1",
            user_id,
            hops: &[journey],
        }
        .execute(&pool)
        .await
        .expect("insert hops failed");

        let token = "share-test-token-abc123";
        let token_hash = sha256_hex(token);
        db::share_tokens::Create {
            user_id,
            token_hash: &token_hash,
            label: "test",
        }
        .execute(&pool)
        .await
        .expect("create share token failed");

        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/share/{token}"))
                    .header(header::ACCEPT, "text/html")
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_text(response).await;
        assert!(body.contains("Aer Lingus"), "should show airline");
        assert!(body.contains("og:title"), "should include OG meta tags");
        assert!(!body.contains("<nav"), "should not include navbar");
    }

    #[tokio::test]
    async fn share_page_returns_json_when_requested() {
        let pool = test_pool().await;
        let user_id = db::tests::test_user(&pool, "bob").await;

        let token = "share-json-token";
        let token_hash = sha256_hex(token);
        db::share_tokens::Create {
            user_id,
            token_hash: &token_hash,
            label: "json test",
        }
        .execute(&pool)
        .await
        .expect("create share token failed");

        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/share/{token}"))
                    .header(header::ACCEPT, "application/json")
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_text(response).await;
        assert!(body.contains("total_journeys"));
    }

    #[tokio::test]
    async fn share_page_supports_year_filter() {
        let pool = test_pool().await;
        let user_id = db::tests::test_user(&pool, "alice").await;

        let journey_2024 = sample_hop(TravelType::Air, "DUB", "LHR", "2024-06-15", "2024-06-15");
        let journey_2023 = sample_hop(TravelType::Air, "SFO", "NRT", "2023-03-01", "2023-03-01");

        Create {
            trip_id: "trip-1",
            user_id,
            hops: &[journey_2024],
        }
        .execute(&pool)
        .await
        .expect("insert 2024 failed");
        Create {
            trip_id: "trip-2",
            user_id,
            hops: &[journey_2023],
        }
        .execute(&pool)
        .await
        .expect("insert 2023 failed");

        let token = "share-year-filter-token";
        let token_hash = sha256_hex(token);
        db::share_tokens::Create {
            user_id,
            token_hash: &token_hash,
            label: "year filter",
        }
        .execute(&pool)
        .await
        .expect("create share token failed");

        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/share/{token}?year=2024"))
                    .header(header::ACCEPT, "application/json")
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_text(response).await;
        let parsed: serde_json::Value = serde_json::from_str(&body).expect("json parse");
        assert_eq!(parsed["total_journeys"], 1);
        assert_eq!(parsed["selected_year"], "2024");
    }
}

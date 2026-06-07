use super::{MultiFormatResponse, ResponseFormat, negotiate_format};
use crate::{
    db,
    server::{
        AppState,
        pages::stats::{StatsQuery, compute_detailed_stats},
    },
};
use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};

pub async fn handler(
    State(state): State<AppState>,
    Path(token_hash): Path<String>,
    Query(query): Query<StatsQuery>,
    headers: HeaderMap,
) -> Response {
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

    let detailed = compute_detailed_stats(
        &all_rows,
        query.year.as_deref(),
        query.travel_type.as_deref(),
    );

    let format = negotiate_format(&headers);
    if format == ResponseFormat::Html {
        crate::server::pages::stats::render_share_page(detailed, &token_hash)
    } else {
        let response = super::stats::StatsResponse::from(detailed);
        super::stats::StatsResponse::single_format_response(&response, format, StatusCode::OK)
    }
}

/// Public Year-in-Review recap for a shared token. Defaults to the most recent
/// year with data when no `?year=` is given.
pub async fn review_handler(
    State(state): State<AppState>,
    Path(token_hash): Path<String>,
    Query(query): Query<StatsQuery>,
) -> Response {
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
            tracing::error!(error = %err, "failed to fetch stats for year-in-review");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    // Choose the year: explicit `?year=`, else the most recent year with data.
    let base = compute_detailed_stats(&all_rows, None, None);
    let year = query
        .year
        .filter(|y| !y.is_empty())
        .or_else(|| base.available_years.last().cloned());

    let (recap, year_label) = match year {
        Some(y) => (compute_detailed_stats(&all_rows, Some(&y), None), y),
        None => (base, "All Time".to_owned()),
    };

    crate::server::pages::year_in_review::render(recap, &token_hash, &year_label)
}

#[cfg(test)]
mod tests {
    use crate::{
        db::{
            self,
            hops::{Create, FlightDetail, TravelType},
        },
        server::{create_router, test_helpers::*},
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

        let token_hash = "share_hash_abc123";
        db::share_tokens::Create {
            user_id,
            token_hash,
            label: "test",
        }
        .execute(&pool)
        .await
        .expect("create share token failed");

        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/share/{token_hash}"))
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

        let token_hash = "share_hash_json";
        db::share_tokens::Create {
            user_id,
            token_hash,
            label: "json test",
        }
        .execute(&pool)
        .await
        .expect("create share token failed");

        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/share/{token_hash}"))
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
    async fn year_in_review_renders_recap_for_latest_year() {
        let pool = test_pool().await;
        let user_id = db::tests::test_user(&pool, "alice").await;

        let mut journey = sample_hop(TravelType::Air, "DUB", "JFK", "2024-06-15", "2024-06-15");
        journey.flight_detail = Some(FlightDetail {
            airline: "Aer Lingus".to_string(),
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

        let token_hash = "share_review_token";
        db::share_tokens::Create {
            user_id,
            token_hash,
            label: "review",
        }
        .execute(&pool)
        .await
        .expect("create share token failed");

        let app = create_router(test_app_state(pool));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/share/{token_hash}/review"))
                    .header(header::ACCEPT, "text/html")
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_text(response).await;
        assert!(body.contains("Year in Review"));
        assert!(body.contains("2024"), "should default to the latest year");
        assert!(
            body.contains("Aer Lingus"),
            "should surface the top airline"
        );
        assert!(
            !body.contains("nav-brand"),
            "recap should not include the app navbar"
        );
    }

    #[tokio::test]
    async fn year_in_review_returns_404_for_invalid_token() {
        let pool = test_pool().await;
        let app = create_router(test_app_state(pool));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/share/nope/review")
                    .header(header::ACCEPT, "text/html")
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn year_in_review_honours_explicit_year() {
        let pool = test_pool().await;
        let user_id = db::tests::test_user(&pool, "alice").await;

        Create {
            trip_id: "t24",
            user_id,
            hops: &[sample_hop(
                TravelType::Air,
                "DUB",
                "LHR",
                "2024-06-15",
                "2024-06-15",
            )],
        }
        .execute(&pool)
        .await
        .expect("insert 2024");
        Create {
            trip_id: "t23",
            user_id,
            hops: &[sample_hop(
                TravelType::Air,
                "SFO",
                "NRT",
                "2023-03-01",
                "2023-03-01",
            )],
        }
        .execute(&pool)
        .await
        .expect("insert 2023");

        let token_hash = "share_review_year";
        db::share_tokens::Create {
            user_id,
            token_hash,
            label: "review year",
        }
        .execute(&pool)
        .await
        .expect("create share token failed");

        let app = create_router(test_app_state(pool));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/share/{token_hash}/review?year=2023"))
                    .header(header::ACCEPT, "text/html")
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_text(response).await;
        assert!(body.contains("NRT"), "2023 recap should mention its route");
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

        let token_hash = "share_hash_year_filter";
        db::share_tokens::Create {
            user_id,
            token_hash,
            label: "year filter",
        }
        .execute(&pool)
        .await
        .expect("create share token failed");

        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/share/{token_hash}?year=2024"))
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

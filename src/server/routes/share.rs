use super::{ErrorResponse, MultiFormatResponse, ResponseFormat, negotiate_format};
use crate::{
    db,
    server::{
        AppState,
        pages::shared_journey,
        pages::stats::{StatsQuery, compute_detailed_stats},
        routes::enrichments::EnrichmentResponse,
    },
};
use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use chrono::NaiveDateTime;

fn is_expired(expires_at: &str) -> bool {
    let Ok(expires) = NaiveDateTime::parse_from_str(expires_at, "%Y-%m-%d %H:%M:%S") else {
        return true;
    };
    chrono::Utc::now().naive_utc() > expires
}

async fn resolve_shared_journey(
    state: &AppState,
    token_hash: &str,
) -> Result<Option<(i64, db::hops::DetailRow)>, sqlx::Error> {
    let token_row = (db::journey_share_tokens::GetByTokenHash { token_hash })
        .execute(&state.db)
        .await?;
    let Some(token_row) = token_row else {
        return Ok(None);
    };
    if is_expired(&token_row.expires_at) {
        return Ok(None);
    }
    let journey = (db::hops::GetById {
        id: token_row.hop_id,
        user_id: token_row.user_id,
    })
    .execute(&state.db)
    .await?;
    Ok(journey.map(|j| (token_row.hop_id, j)))
}

fn non_empty(value: &str) -> Option<String> {
    if value.is_empty() {
        None
    } else {
        Some(value.to_owned())
    }
}

fn enrichment_is_fresh(fetched_at: &str, start_date: &str) -> bool {
    let Ok(fetched) = NaiveDateTime::parse_from_str(fetched_at, "%Y-%m-%d %H:%M:%S") else {
        return false;
    };
    let ttl = crate::worker::departure_aware_ttl(start_date);
    (chrono::Utc::now().naive_utc() - fetched).num_seconds() < ttl
}

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

/// Public live journey share page for unauthenticated viewers.
pub async fn shared_journey_handler(
    State(state): State<AppState>,
    Path(token_hash): Path<String>,
) -> Response {
    let _ = db::journey_share_tokens::DeleteExpired
        .execute(&state.db)
        .await;
    let resolved = match resolve_shared_journey(&state, &token_hash).await {
        Ok(value) => value,
        Err(err) => {
            tracing::error!(error = %err, "failed to resolve shared journey token");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    let Some((hop_id, journey)) = resolved else {
        return (StatusCode::NOT_FOUND, "link expired").into_response();
    };
    let enrichment = (db::status_enrichments::GetByHopId { hop_id })
        .execute(&state.db)
        .await
        .ok()
        .flatten();
    shared_journey::render_page(&token_hash, journey, enrichment)
}

/// Public live enrichment polling endpoint for a shared journey.
pub async fn shared_journey_enrichments_handler(
    State(state): State<AppState>,
    Path(token_hash): Path<String>,
    Query(params): Query<crate::server::routes::enrichments::EnrichmentQuery>,
    headers: HeaderMap,
) -> Response {
    let _ = db::journey_share_tokens::DeleteExpired
        .execute(&state.db)
        .await;
    let format = negotiate_format(&headers);
    let resolved = match resolve_shared_journey(&state, &token_hash).await {
        Ok(value) => value,
        Err(err) => {
            tracing::error!(error = %err, "failed to resolve shared journey token");
            return ErrorResponse::into_format_response(
                "internal error",
                format,
                StatusCode::INTERNAL_SERVER_ERROR,
            );
        }
    };
    let Some((hop_id, journey)) = resolved else {
        return ErrorResponse::into_format_response("link expired", format, StatusCode::NOT_FOUND);
    };
    let rows = match (db::status_enrichments::GetAllByHopId { hop_id })
        .execute(&state.db)
        .await
    {
        Ok(rows) => rows,
        Err(err) => {
            tracing::error!(error = %err, "shared journey enrichments query failed");
            return ErrorResponse::into_format_response(
                "internal error",
                format,
                StatusCode::INTERNAL_SERVER_ERROR,
            );
        }
    };
    let items: Vec<EnrichmentResponse> = rows
        .into_iter()
        .map(|row| EnrichmentResponse {
            id: row.id,
            hop_id: row.hop_id,
            provider: row.provider,
            status: row.status,
            delay_minutes: row.delay_minutes,
            dep_gate: non_empty(&row.dep_gate),
            dep_terminal: non_empty(&row.dep_terminal),
            arr_gate: non_empty(&row.arr_gate),
            arr_terminal: non_empty(&row.arr_terminal),
            dep_platform: non_empty(&row.dep_platform),
            arr_platform: non_empty(&row.arr_platform),
            aircraft_reg: non_empty(&row.aircraft_reg),
            fetched_at: row.fetched_at.clone(),
            is_fresh: enrichment_is_fresh(&row.fetched_at, &journey.start_date),
            raw_json: if params.include_raw {
                Some(row.raw_json)
            } else {
                None
            },
        })
        .collect();

    EnrichmentResponse::into_format_response(&items, format, StatusCode::OK)
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
        body::{Body, to_bytes},
        http::{Request, StatusCode, header},
    };
    use tower::ServiceExt;

    async fn create_shared_journey_token(
        pool: &sqlx::SqlitePool,
        username: &str,
        start_date: &str,
        expires_at: &str,
    ) -> (i64, String) {
        let user_id = db::tests::test_user(pool, username).await;
        Create {
            trip_id: "trip-shared-journey",
            user_id,
            hops: &[sample_hop(
                TravelType::Air,
                "DUB",
                "LHR",
                start_date,
                start_date,
            )],
        }
        .execute(pool)
        .await
        .expect("insert hops failed");
        let hop_id = db::hops::GetAll {
            user_id,
            travel_type_filter: None,
        }
        .execute(pool)
        .await
        .expect("list hops failed")[0]
            .id;

        let token_hash = format!("journey_token_{username}_{start_date}");
        db::journey_share_tokens::Create {
            user_id,
            hop_id,
            token_hash: &token_hash,
            expires_at,
        }
        .execute(pool)
        .await
        .expect("create journey share token failed");

        (hop_id, token_hash)
    }

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

    #[tokio::test]
    async fn shared_journey_page_returns_404_for_invalid_token() {
        let pool = test_pool().await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/share/journey/does-not-exist")
                    .header(header::ACCEPT, "text/html")
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn shared_journey_page_returns_404_for_expired_token() {
        let pool = test_pool().await;
        let (_hop_id, token_hash) =
            create_shared_journey_token(&pool, "alice", "2024-06-15", "2000-01-01 00:00:00").await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/share/journey/{token_hash}"))
                    .header(header::ACCEPT, "text/html")
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let body = body_text(response).await;
        assert!(body.contains("link expired"));
    }

    #[tokio::test]
    async fn shared_journey_page_renders_for_valid_token() {
        let pool = test_pool().await;
        let (hop_id, token_hash) =
            create_shared_journey_token(&pool, "alice", "2026-06-15", "2999-01-01 00:00:00").await;
        db::status_enrichments::Upsert {
            hop_id,
            provider: "airlabs",
            status: "active",
            delay_minutes: Some(10),
            dep_gate: "B22",
            dep_terminal: "1",
            arr_gate: "C9",
            arr_terminal: "2",
            dep_platform: "",
            arr_platform: "",
            aircraft_reg: "EI-DEG",
            raw_json: "{}",
        }
        .execute(&pool)
        .await
        .expect("upsert enrichment failed");

        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/share/journey/{token_hash}"))
                    .header(header::ACCEPT, "text/html")
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_text(response).await;
        assert!(body.contains("DUB"));
        assert!(body.contains("LHR"));
        assert!(body.contains("shared-journey-poll.js"));
        assert!(!body.contains("<nav"));
    }

    #[tokio::test]
    async fn shared_journey_enrichments_returns_json_array() {
        let pool = test_pool().await;
        let (hop_id, token_hash) =
            create_shared_journey_token(&pool, "alice", "2026-06-15", "2999-01-01 00:00:00").await;
        db::status_enrichments::Upsert {
            hop_id,
            provider: "airlabs",
            status: "active",
            delay_minutes: Some(5),
            dep_gate: "A1",
            dep_terminal: "2",
            arr_gate: "",
            arr_terminal: "",
            dep_platform: "",
            arr_platform: "",
            aircraft_reg: "",
            raw_json: "{\"ok\":true}",
        }
        .execute(&pool)
        .await
        .expect("upsert enrichment failed");

        let app = create_router(test_app_state(pool));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/share/journey/{token_hash}/enrichments"))
                    .header(header::ACCEPT, "application/json")
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("read response body");
        let parsed: serde_json::Value = serde_json::from_slice(&body).expect("json parse");
        assert!(parsed.is_array());
        assert_eq!(parsed[0]["status"], "active");
    }
}

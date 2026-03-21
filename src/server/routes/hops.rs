use crate::{
    db,
    models::TravelHop,
    server::{AppState, middleware::AuthUser, routes::types::ErrorResponse},
};
use aide::transform::TransformOperation;
use axum::{
    Json,
    extract::{Query, State},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize, JsonSchema)]
pub struct HopQuery {
    #[serde(rename = "type")]
    travel_type: Option<String>,
}

pub async fn hops_handler(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(query): Query<HopQuery>,
    headers: HeaderMap,
) -> Response {
    let hops = match db::get_all_hops(&state.db, auth.user_id, query.travel_type.as_deref()).await {
        Ok(hops) => hops,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("failed to fetch hops: {err}") })),
            )
                .into_response();
        }
    };

    match negotiate_format(&headers) {
        ResponseFormat::Json => (StatusCode::OK, Json(json!(hops))).into_response(),
        ResponseFormat::Csv => build_csv_response(&hops),
        ResponseFormat::Html => build_html_response(&hops),
    }
}

pub fn hops_handler_docs(op: TransformOperation) -> TransformOperation {
    op.description("List travel hops for the authenticated user.")
        .response::<200, Json<Vec<TravelHop>>>()
        .response::<500, Json<ErrorResponse>>()
        .tag("hops")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ResponseFormat {
    Json,
    Csv,
    Html,
}

fn negotiate_format(headers: &HeaderMap) -> ResponseFormat {
    let accept = headers
        .get(header::ACCEPT)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    for part in accept.split(',') {
        let media = part.split(';').next().unwrap_or("").trim();
        match media {
            "text/html" => return ResponseFormat::Html,
            "text/csv" => return ResponseFormat::Csv,
            "application/json" => return ResponseFormat::Json,
            _ => {}
        }
    }

    ResponseFormat::Json
}

#[must_use]
fn build_csv_response(hops: &[TravelHop]) -> Response {
    let mut writer = csv::Writer::from_writer(Vec::new());
    if let Err(err) = writer.write_record([
        "travel_type",
        "origin_name",
        "origin_lat",
        "origin_lng",
        "dest_name",
        "dest_lat",
        "dest_lng",
        "start_date",
        "end_date",
    ]) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("failed to write CSV header: {err}"),
        )
            .into_response();
    }

    for hop in hops {
        if let Err(err) = writer.write_record([
            hop.travel_type.to_string(),
            hop.origin_name.clone(),
            opt_f64_to_string(hop.origin_lat),
            opt_f64_to_string(hop.origin_lng),
            hop.dest_name.clone(),
            opt_f64_to_string(hop.dest_lat),
            opt_f64_to_string(hop.dest_lng),
            hop.start_date.clone(),
            hop.end_date.clone(),
        ]) {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed to write CSV record: {err}"),
            )
                .into_response();
        }
    }

    if let Err(err) = writer.flush() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("failed to flush CSV writer: {err}"),
        )
            .into_response();
    }

    let body = match writer.into_inner() {
        Ok(bytes) => bytes,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed to build CSV response body: {}", err.into_error()),
            )
                .into_response();
        }
    };

    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "text/csv; charset=utf-8"),
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"hops.csv\"",
            ),
        ],
        body,
    )
        .into_response()
}

#[must_use]
fn build_html_response(hops: &[TravelHop]) -> Response {
    use leptos::prelude::*;

    let rows = hops
        .iter()
        .map(|hop| {
            let travel_type = format!("{} {}", hop.travel_type.emoji(), hop.travel_type);
            let origin = hop.origin_name.clone();
            let origin_lat = opt_f64_to_string(hop.origin_lat);
            let origin_lng = opt_f64_to_string(hop.origin_lng);
            let dest = hop.dest_name.clone();
            let dest_lat = opt_f64_to_string(hop.dest_lat);
            let dest_lng = opt_f64_to_string(hop.dest_lng);
            let start = hop.start_date.clone();
            let end = hop.end_date.clone();
            view! {
                <tr>
                    <td>{travel_type}</td>
                    <td>{origin}</td>
                    <td>{origin_lat}</td>
                    <td>{origin_lng}</td>
                    <td>{dest}</td>
                    <td>{dest_lat}</td>
                    <td>{dest_lng}</td>
                    <td>{start}</td>
                    <td>{end}</td>
                </tr>
            }
        })
        .collect::<Vec<_>>();

    let html = view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <title>"Travel Hops"</title>
                <link rel="stylesheet" href="/static/style.css" />
            </head>
            <body>
                <h1>"Travel Hops"</h1>
                <table>
                    <thead>
                        <tr>
                            <th>"Type"</th>
                            <th>"Origin"</th>
                            <th>"Origin Lat"</th>
                            <th>"Origin Lng"</th>
                            <th>"Destination"</th>
                            <th>"Dest Lat"</th>
                            <th>"Dest Lng"</th>
                            <th>"Start Date"</th>
                            <th>"End Date"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {rows}
                    </tbody>
                </table>
            </body>
        </html>
    };

    axum::response::Html(html.to_html()).into_response()
}

fn opt_f64_to_string(val: Option<f64>) -> String {
    val.map_or_else(String::new, |v| v.to_string())
}

#[cfg(test)]
mod tests {
    use crate::{
        db,
        models::{TravelHop, TravelType},
        server::create_router,
        server::test_helpers::helpers::*,
    };
    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode, header},
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn access_hops_without_auth_returns_unauthorized() {
        let pool = test_pool().await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/hops")
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn access_hops_with_session_cookie_returns_ok() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let user = db::get_user_by_username(&pool, "alice")
            .await
            .expect("lookup failed")
            .expect("missing user");
        db::insert_hops(
            &pool,
            "trip-1",
            user.id,
            &[sample_hop(
                TravelType::Air,
                "LHR",
                "JFK",
                "2024-01-01",
                "2024-01-01",
            )],
        )
        .await
        .expect("insert failed");

        let app = create_router(test_app_state(pool));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/hops")
                    .header(header::COOKIE, cookie)
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn access_hops_with_api_key_returns_ok() {
        let pool = test_pool().await;
        let _ = auth_cookie_for_user(&pool, "alice").await;
        let user = db::get_user_by_username(&pool, "alice")
            .await
            .expect("lookup failed")
            .expect("missing user");
        db::insert_hops(
            &pool,
            "trip-1",
            user.id,
            &[sample_hop(
                TravelType::Rail,
                "Paris",
                "London",
                "2024-01-01",
                "2024-01-01",
            )],
        )
        .await
        .expect("insert failed");
        api_key_for_user(&pool, "alice", "my-api-key").await;

        let app = create_router(test_app_state(pool));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/hops")
                    .header(header::AUTHORIZATION, "Bearer my-api-key")
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn get_hops_json_returns_inserted_hops() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let user = db::get_user_by_username(&pool, "alice")
            .await
            .expect("lookup failed")
            .expect("missing user");
        let hops = vec![
            sample_hop(
                TravelType::Rail,
                "Paris",
                "London",
                "2024-01-01",
                "2024-01-01",
            ),
            sample_hop(TravelType::Air, "LHR", "JFK", "2024-02-01", "2024-02-01"),
        ];
        db::insert_hops(&pool, "trip-1", user.id, &hops)
            .await
            .expect("insert failed");

        let app = create_router(test_app_state(pool));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/hops")
                    .header(header::COOKIE, cookie)
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("failed to read response body");
        let parsed: Vec<TravelHop> =
            serde_json::from_slice(&body).expect("body should be valid JSON array");

        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].travel_type, TravelType::Rail);
    }

    #[tokio::test]
    async fn get_hops_json_filters_by_type() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let user = db::get_user_by_username(&pool, "alice")
            .await
            .expect("lookup failed")
            .expect("missing user");
        let hops = vec![
            sample_hop(TravelType::Air, "LHR", "JFK", "2024-02-01", "2024-02-01"),
            sample_hop(
                TravelType::Rail,
                "Paris",
                "London",
                "2024-01-01",
                "2024-01-01",
            ),
        ];
        db::insert_hops(&pool, "trip-1", user.id, &hops)
            .await
            .expect("insert failed");

        let app = create_router(test_app_state(pool));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/hops?type=rail")
                    .header(header::COOKIE, cookie)
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("failed to read response body");
        let parsed: Vec<TravelHop> = serde_json::from_slice(&body).expect("valid json");
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].travel_type, TravelType::Rail);
    }

    #[tokio::test]
    async fn get_hops_with_accept_csv_returns_csv() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let user = db::get_user_by_username(&pool, "alice")
            .await
            .expect("lookup failed")
            .expect("missing user");
        db::insert_hops(
            &pool,
            "trip-1",
            user.id,
            &[sample_hop(
                TravelType::Rail,
                "Paris",
                "London",
                "2024-01-01",
                "2024-01-01",
            )],
        )
        .await
        .expect("insert failed");

        let app = create_router(test_app_state(pool));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/hops")
                    .header(header::COOKIE, cookie)
                    .header(header::ACCEPT, "text/csv")
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn get_hops_with_accept_html_returns_html_table() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let user = db::get_user_by_username(&pool, "alice")
            .await
            .expect("lookup failed")
            .expect("missing user");
        db::insert_hops(
            &pool,
            "trip-1",
            user.id,
            &[sample_hop(
                TravelType::Air,
                "LHR",
                "JFK",
                "2024-02-01",
                "2024-02-01",
            )],
        )
        .await
        .expect("insert failed");

        let app = create_router(test_app_state(pool));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/hops")
                    .header(header::COOKIE, cookie)
                    .header(header::ACCEPT, "text/html")
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn get_hops_with_accept_html_contains_table_headers_and_hop_data() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let user = db::get_user_by_username(&pool, "alice")
            .await
            .expect("lookup failed")
            .expect("missing user");
        db::insert_hops(
            &pool,
            "trip-2",
            user.id,
            &[sample_hop(
                TravelType::Rail,
                "Paris",
                "London",
                "2024-01-01",
                "2024-01-01",
            )],
        )
        .await
        .expect("insert failed");

        let app = create_router(test_app_state(pool));
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/hops")
                    .header(header::COOKIE, cookie)
                    .header(header::ACCEPT, "text/html")
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_text(response).await;
        assert!(body.contains("<table>"));
        assert!(body.contains("Travel Hops"));
        assert!(body.contains("Type"));
        assert!(body.contains("Origin"));
        assert!(body.contains("Destination"));
        assert!(body.contains("Paris"));
        assert!(body.contains("London"));
    }
}

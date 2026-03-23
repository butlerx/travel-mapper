use crate::{
    db,
    integrations::flighty::{FlightRow, parse_csv},
    server::{AppState, error::AppError, middleware::AuthUser},
};
use axum::{
    Json,
    body::Body,
    extract::{FromRequest, Multipart, State},
    http::{StatusCode, header},
    response::{IntoResponse, Redirect, Response},
};
use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};
use schemars::JsonSchema;
use serde::Serialize;
use sqlx::SqlitePool;

const QUERY_ENCODE_SET: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'_')
    .remove(b'.')
    .remove(b'~');

#[derive(Debug, Serialize, JsonSchema)]
pub struct ImportResponse {
    pub imported: u64,
}

async fn do_import(pool: &SqlitePool, user_id: i64, rows: &[FlightRow]) -> Result<u64, AppError> {
    Ok((db::hops::CreateFromFlighty { user_id, rows })
        .execute(pool)
        .await?)
}

fn encode_query_value(s: &str) -> String {
    utf8_percent_encode(s, QUERY_ENCODE_SET).to_string()
}

fn redirect_settings_error(msg: &str) -> Response {
    let encoded = encode_query_value(msg);
    Redirect::to(&format!("/settings?error={encoded}")).into_response()
}

pub async fn import_flighty_handler(
    State(state): State<AppState>,
    auth: AuthUser,
    request: axum::http::Request<Body>,
) -> Response {
    let is_multipart = request
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|ct| ct.starts_with("multipart/form-data"));

    if is_multipart {
        handle_multipart(state, auth, request).await
    } else {
        handle_raw_csv(state, auth, request).await
    }
}

async fn handle_raw_csv(
    state: AppState,
    auth: AuthUser,
    request: axum::http::Request<Body>,
) -> Response {
    match handle_raw_csv_inner(state, auth, request).await {
        Ok(resp) => resp,
        Err(err) => err.into_response(),
    }
}

async fn handle_raw_csv_inner(
    state: AppState,
    auth: AuthUser,
    request: axum::http::Request<Body>,
) -> Result<Response, AppError> {
    let bytes = axum::body::to_bytes(request.into_body(), 50 * 1024 * 1024).await?;
    let rows = parse_csv(bytes.as_ref())?;
    let imported = do_import(&state.db, auth.user_id, &rows).await?;
    Ok((StatusCode::OK, Json(ImportResponse { imported })).into_response())
}

async fn handle_multipart(
    state: AppState,
    auth: AuthUser,
    request: axum::http::Request<Body>,
) -> Response {
    let mut multipart = match Multipart::from_request(request, &()).await {
        Ok(m) => m,
        Err(err) => return redirect_settings_error(&format!("upload error: {err}")),
    };

    let csv_bytes = loop {
        match multipart.next_field().await {
            Ok(Some(field)) if field.name() == Some("file") => {
                break match field.bytes().await {
                    Ok(b) => b,
                    Err(err) => {
                        return redirect_settings_error(&format!("failed to read upload: {err}"));
                    }
                };
            }
            Ok(Some(_)) => {}
            Ok(None) => return redirect_settings_error("No file uploaded"),
            Err(err) => return redirect_settings_error(&format!("upload error: {err}")),
        }
    };

    let rows = match parse_csv(csv_bytes.as_ref()) {
        Ok(rows) => rows,
        Err(err) => return redirect_settings_error(&format!("invalid CSV: {err}")),
    };

    match do_import(&state.db, auth.user_id, &rows).await {
        Ok(imported) => Redirect::to(&format!("/settings?flighty={imported}")).into_response(),
        Err(err) => redirect_settings_error(&err.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use crate::server::{create_router, test_helpers::helpers::*};
    use axum::{
        body::Body,
        http::{Request, StatusCode, header},
    };
    use tower::ServiceExt;

    const SAMPLE_CSV: &str = "\
Date,Airline,Flight,From,To,Dep Terminal,Dep Gate,Arr Terminal,Arr Gate,Canceled,Diverted To,Gate Departure (Scheduled),Gate Departure (Actual),Take off (Scheduled),Take off (Actual),Landing (Scheduled),Landing (Actual),Gate Arrival (Scheduled),Gate Arrival (Actual),Aircraft Type Name,Tail Number,PNR,Seat,Seat Type,Cabin Class,Flight Reason,Notes,Flight Flighty ID,Airline Flighty ID,Departure Airport Flighty ID,Arrival Airport Flighty ID,Diverted To Airport Flighty ID,Aircraft Type Flighty ID
2024-12-31,EIN,709,RAK,DUB,1,,2,302,false,,2024-12-31T17:55,2024-12-31T18:07,2024-12-31T18:05,2024-12-31T18:29,2024-12-31T20:25,2024-12-31T20:52,2024-12-31T20:35,,Airbus A320,EIGAL,2IYS7F,,,,,,1d00499b-cab2-4830-bacc-d8ed713b9074,a4a016d4-2a72-44e9-8708-6ed6a5a9d2d1,afd2bec1-df2f-4126-95b8-fa33971a2afe,5dc01a9f-6855-4248-8e7b-0fef83144078,,dd390a1f-e92a-4411-ab72-04db16d42030";

    #[tokio::test]
    async fn import_flighty_without_auth_returns_unauthorized() {
        let pool = test_pool().await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/import/flighty")
                    .body(Body::from(SAMPLE_CSV))
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn import_flighty_csv_stores_hops() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let app = create_router(test_app_state(pool.clone()));

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/import/flighty")
                    .header(header::COOKIE, &cookie)
                    .body(Body::from(SAMPLE_CSV))
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);

        let body = body_text(response).await;
        let json: serde_json::Value = serde_json::from_str(&body).expect("valid json");
        assert_eq!(json["imported"], 1);

        let user = crate::db::users::GetByUsername { username: "alice" }
            .execute(&pool)
            .await
            .expect("lookup failed")
            .expect("missing user");
        let hops = crate::db::hops::GetAll {
            user_id: user.id,
            travel_type_filter: Some("air"),
        }
        .execute(&pool)
        .await
        .expect("fetch failed");
        assert_eq!(hops.len(), 1);
        assert_eq!(hops[0].origin_name, "RAK");
        assert_eq!(hops[0].dest_name, "DUB");
    }

    #[tokio::test]
    async fn import_flighty_invalid_csv_returns_bad_request() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let app = create_router(test_app_state(pool));

        let bad_csv = "Date,Airline,Flight,From,To,Dep Terminal,Dep Gate,Arr Terminal,Arr Gate,Canceled,Diverted To,Gate Departure (Scheduled),Gate Departure (Actual),Take off (Scheduled),Take off (Actual),Landing (Scheduled),Landing (Actual),Gate Arrival (Scheduled),Gate Arrival (Actual),Aircraft Type Name,Tail Number,PNR,Seat,Seat Type,Cabin Class,Flight Reason,Notes,Flight Flighty ID,Airline Flighty ID,Departure Airport Flighty ID,Arrival Airport Flighty ID,Diverted To Airport Flighty ID,Aircraft Type Flighty ID\n2024-01-01,RYR";

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/import/flighty")
                    .header(header::COOKIE, &cookie)
                    .body(Body::from(bad_csv))
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn import_flighty_idempotent_reimport() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;

        for _ in 0..2 {
            let app = create_router(test_app_state(pool.clone()));
            let response = app
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri("/import/flighty")
                        .header(header::COOKIE, &cookie)
                        .body(Body::from(SAMPLE_CSV))
                        .expect("failed to build request"),
                )
                .await
                .expect("router request failed");
            assert_eq!(response.status(), StatusCode::OK);
        }

        let user = crate::db::users::GetByUsername { username: "alice" }
            .execute(&pool)
            .await
            .expect("lookup failed")
            .expect("missing user");
        let hops = crate::db::hops::GetAll {
            user_id: user.id,
            travel_type_filter: None,
        }
        .execute(&pool)
        .await
        .expect("fetch failed");
        assert_eq!(hops.len(), 1, "reimport should not duplicate");
    }

    #[tokio::test]
    async fn import_flighty_multipart_redirects_with_count() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let app = create_router(test_app_state(pool.clone()));

        let boundary = "----TestBoundary7MA4YWxkTrZu0gW";
        let body = format!(
            "------TestBoundary7MA4YWxkTrZu0gW\r\n\
             Content-Disposition: form-data; name=\"file\"; filename=\"test.csv\"\r\n\
             Content-Type: text/csv\r\n\r\n\
             {SAMPLE_CSV}\r\n\
             ------TestBoundary7MA4YWxkTrZu0gW--\r\n"
        );

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/import/flighty")
                    .header(header::COOKIE, &cookie)
                    .header(
                        header::CONTENT_TYPE,
                        format!("multipart/form-data; boundary={boundary}"),
                    )
                    .body(Body::from(body))
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        let location = response
            .headers()
            .get(header::LOCATION)
            .expect("missing Location header")
            .to_str()
            .expect("non-ascii location");
        assert!(
            location.starts_with("/settings?flighty="),
            "expected redirect to /settings?flighty=N, got: {location}"
        );
    }

    #[tokio::test]
    async fn import_flighty_multipart_without_auth_returns_unauthorized() {
        let pool = test_pool().await;
        let app = create_router(test_app_state(pool));

        let boundary = "----TestBoundary7MA4YWxkTrZu0gW";
        let body = format!(
            "------TestBoundary7MA4YWxkTrZu0gW\r\n\
             Content-Disposition: form-data; name=\"file\"; filename=\"test.csv\"\r\n\
             Content-Type: text/csv\r\n\r\n\
             {SAMPLE_CSV}\r\n\
             ------TestBoundary7MA4YWxkTrZu0gW--\r\n"
        );

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/import/flighty")
                    .header(
                        header::CONTENT_TYPE,
                        format!("multipart/form-data; boundary={boundary}"),
                    )
                    .body(Body::from(body))
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn import_flighty_multipart_no_file_redirects_with_error() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let app = create_router(test_app_state(pool));

        let boundary = "----TestBoundary7MA4YWxkTrZu0gW";
        let body = "------TestBoundary7MA4YWxkTrZu0gW\r\n\
             Content-Disposition: form-data; name=\"other\"\r\n\r\n\
             nothing\r\n\
             ------TestBoundary7MA4YWxkTrZu0gW--\r\n"
            .to_string();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/import/flighty")
                    .header(header::COOKIE, &cookie)
                    .header(
                        header::CONTENT_TYPE,
                        format!("multipart/form-data; boundary={boundary}"),
                    )
                    .body(Body::from(body))
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        let location = response
            .headers()
            .get(header::LOCATION)
            .expect("missing Location header")
            .to_str()
            .expect("non-ascii location");
        assert!(
            location.starts_with("/settings?error="),
            "expected redirect to /settings?error=..., got: {location}"
        );
    }
}

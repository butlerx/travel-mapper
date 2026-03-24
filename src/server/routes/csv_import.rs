use crate::{
    db,
    integrations::generic_csv::{self, GenericRow, ImportFormat},
    server::{AppState, error::AppError, middleware::AuthUser},
};
use axum::{
    Json,
    body::Body,
    extract::{FromRequest, Multipart, Query, State},
    http::{StatusCode, header},
    response::{IntoResponse, Redirect, Response},
};
use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
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

#[derive(Debug, Deserialize, Default)]
pub struct ImportParams {
    pub format: Option<String>,
}

fn parse_format_param(s: &str) -> Option<ImportFormat> {
    match s.to_lowercase().as_str() {
        "myflightradar24" | "fr24" => Some(ImportFormat::MyFlightradar24),
        "openflights" => Some(ImportFormat::OpenFlights),
        "appintheair" | "app_in_the_air" => Some(ImportFormat::AppInTheAir),
        "flighty" => Some(ImportFormat::Flighty),
        _ => None,
    }
}

async fn do_import(pool: &SqlitePool, user_id: i64, rows: &[GenericRow]) -> Result<u64, AppError> {
    Ok((db::hops::CreateFromCsv { user_id, rows })
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

pub async fn handler(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(params): Query<ImportParams>,
    request: axum::http::Request<Body>,
) -> Response {
    let format_override = params.format.as_deref().and_then(parse_format_param);

    let is_multipart = request
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|ct| ct.starts_with("multipart/form-data"));

    if is_multipart {
        handle_multipart(state, auth, format_override, request).await
    } else {
        handle_raw_csv(state, auth, format_override, request).await
    }
}

async fn handle_raw_csv(
    state: AppState,
    auth: AuthUser,
    format_override: Option<ImportFormat>,
    request: axum::http::Request<Body>,
) -> Response {
    match handle_raw_csv_inner(state, auth, format_override, request).await {
        Ok(resp) => resp,
        Err(err) => err.into_response(),
    }
}

async fn handle_raw_csv_inner(
    state: AppState,
    auth: AuthUser,
    format_override: Option<ImportFormat>,
    request: axum::http::Request<Body>,
) -> Result<Response, AppError> {
    let bytes = axum::body::to_bytes(request.into_body(), 50 * 1024 * 1024).await?;
    let rows = generic_csv::parse_csv(bytes.as_ref(), format_override)?;
    let imported = do_import(&state.db, auth.user_id, &rows).await?;
    Ok((StatusCode::OK, Json(ImportResponse { imported })).into_response())
}

async fn handle_multipart(
    state: AppState,
    auth: AuthUser,
    format_override: Option<ImportFormat>,
    request: axum::http::Request<Body>,
) -> Response {
    let mut multipart = match Multipart::from_request(request, &()).await {
        Ok(m) => m,
        Err(err) => return redirect_settings_error(&format!("upload error: {err}")),
    };

    let mut format_from_field = format_override;
    let mut csv_bytes = None;

    loop {
        match multipart.next_field().await {
            Ok(Some(field)) => {
                let name = field.name().unwrap_or("").to_string();
                match name.as_str() {
                    "format" => {
                        if format_from_field.is_none()
                            && let Ok(val) = field.text().await
                        {
                            format_from_field = parse_format_param(&val);
                        }
                    }
                    "file" => {
                        csv_bytes = match field.bytes().await {
                            Ok(b) => Some(b),
                            Err(err) => {
                                return redirect_settings_error(&format!(
                                    "failed to read upload: {err}"
                                ));
                            }
                        };
                    }
                    _ => {}
                }
            }
            Ok(None) => break,
            Err(err) => return redirect_settings_error(&format!("upload error: {err}")),
        }
    }

    let Some(csv_bytes) = csv_bytes else {
        return redirect_settings_error("No file uploaded");
    };

    let rows = match generic_csv::parse_csv(csv_bytes.as_ref(), format_from_field) {
        Ok(rows) => rows,
        Err(err) => return redirect_settings_error(&format!("invalid CSV: {err}")),
    };

    match do_import(&state.db, auth.user_id, &rows).await {
        Ok(imported) => Redirect::to(&format!("/settings?csv={imported}")).into_response(),
        Err(err) => redirect_settings_error(&err.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use crate::server::{create_router, test_helpers::*};
    use axum::{
        body::Body,
        http::{Request, StatusCode, header},
    };
    use tower::ServiceExt;

    const FR24_CSV: &str = "\
Date,Flight number,From,To,Dep time,Arr time,Duration,Airline,Aircraft,Registration,Seat number,Seat type,Flight class,Flight reason,Note,Dep_id,Arr_id,Airline_id,Aircraft_id
2024-03-15,BA709,LHR (EGLL),DUB (EIDW),08:30,10:00,01:30,British Airways (BA/),Airbus A320,G-EUYL,12A,1,2,1,Nice flight,1234,5678,BA01,A320-01";

    const OPENFLIGHTS_CSV: &str = "\
Date,From,To,Flight_Number,Airline,Distance,Duration,Seat,Seat_Type,Class,Reason,Plane,Registration,Trip,Note,From_OID,To_OID,Airline_OID,Plane_OID
2024-06-01,DUB,JFK,EI105,Aer Lingus,5103,07:30,23A,W,Y,L,Airbus A330,EI-GAJ,Summer Trip,Transatlantic,1001,2002,EI01,A330-01";

    #[tokio::test]
    async fn import_csv_without_auth_returns_unauthorized() {
        let pool = test_pool().await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/import/csv")
                    .body(Body::from(FR24_CSV))
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn import_csv_auto_detect_fr24_stores_hops() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let app = create_router(test_app_state(pool.clone()));

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/import/csv")
                    .header(header::COOKIE, &cookie)
                    .body(Body::from(FR24_CSV))
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
        assert_eq!(hops[0].origin_name, "LHR");
        assert_eq!(hops[0].dest_name, "DUB");
    }

    #[tokio::test]
    async fn import_csv_with_format_param_stores_hops() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let app = create_router(test_app_state(pool.clone()));

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/import/csv?format=openflights")
                    .header(header::COOKIE, &cookie)
                    .body(Body::from(OPENFLIGHTS_CSV))
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);

        let body = body_text(response).await;
        let json: serde_json::Value = serde_json::from_str(&body).expect("valid json");
        assert_eq!(json["imported"], 1);
    }

    #[tokio::test]
    async fn import_csv_idempotent_reimport() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;

        for _ in 0..2 {
            let app = create_router(test_app_state(pool.clone()));
            let response = app
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri("/import/csv")
                        .header(header::COOKIE, &cookie)
                        .body(Body::from(FR24_CSV))
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
    async fn import_csv_multipart_redirects_with_count() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let app = create_router(test_app_state(pool.clone()));

        let boundary = "----TestBoundary7MA4YWxkTrZu0gW";
        let body = format!(
            "------TestBoundary7MA4YWxkTrZu0gW\r\n\
             Content-Disposition: form-data; name=\"file\"; filename=\"test.csv\"\r\n\
             Content-Type: text/csv\r\n\r\n\
             {FR24_CSV}\r\n\
             ------TestBoundary7MA4YWxkTrZu0gW--\r\n"
        );

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/import/csv")
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
            location.starts_with("/settings?csv="),
            "expected redirect to /settings?csv=N, got: {location}"
        );
    }

    #[tokio::test]
    async fn import_csv_multipart_without_auth_returns_unauthorized() {
        let pool = test_pool().await;
        let app = create_router(test_app_state(pool));

        let boundary = "----TestBoundary7MA4YWxkTrZu0gW";
        let body = format!(
            "------TestBoundary7MA4YWxkTrZu0gW\r\n\
             Content-Disposition: form-data; name=\"file\"; filename=\"test.csv\"\r\n\
             Content-Type: text/csv\r\n\r\n\
             {FR24_CSV}\r\n\
             ------TestBoundary7MA4YWxkTrZu0gW--\r\n"
        );

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/import/csv")
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
    async fn import_csv_multipart_no_file_redirects_with_error() {
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
                    .uri("/import/csv")
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

    const FLIGHTY_CSV: &str = "\
Date,Airline,Flight,From,To,Dep Terminal,Dep Gate,Arr Terminal,Arr Gate,Canceled,Diverted To,Gate Departure (Scheduled),Gate Departure (Actual),Take off (Scheduled),Take off (Actual),Landing (Scheduled),Landing (Actual),Gate Arrival (Scheduled),Gate Arrival (Actual),Aircraft Type Name,Tail Number,PNR,Seat,Seat Type,Cabin Class,Flight Reason,Notes,Flight Flighty ID,Airline Flighty ID,Departure Airport Flighty ID,Arrival Airport Flighty ID,Diverted To Airport Flighty ID,Aircraft Type Flighty ID
2024-12-31,EIN,709,RAK,DUB,1,,2,302,false,,2024-12-31T17:55,2024-12-31T18:07,2024-12-31T18:05,2024-12-31T18:29,2024-12-31T20:25,2024-12-31T20:52,2024-12-31T20:35,,Airbus A320,EIGAL,2IYS7F,,,,,,1d00499b-cab2-4830-bacc-d8ed713b9074,a4a016d4-2a72-44e9-8708-6ed6a5a9d2d1,afd2bec1-df2f-4126-95b8-fa33971a2afe,5dc01a9f-6855-4248-8e7b-0fef83144078,,dd390a1f-e92a-4411-ab72-04db16d42030";

    #[tokio::test]
    async fn import_csv_flighty_without_auth_returns_unauthorized() {
        let pool = test_pool().await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/import/csv?format=flighty")
                    .body(Body::from(FLIGHTY_CSV))
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn import_csv_flighty_stores_hops() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let app = create_router(test_app_state(pool.clone()));

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/import/csv?format=flighty")
                    .header(header::COOKIE, &cookie)
                    .body(Body::from(FLIGHTY_CSV))
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
    async fn import_csv_flighty_auto_detect_stores_hops() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let app = create_router(test_app_state(pool.clone()));

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/import/csv")
                    .header(header::COOKIE, &cookie)
                    .body(Body::from(FLIGHTY_CSV))
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);

        let body = body_text(response).await;
        let json: serde_json::Value = serde_json::from_str(&body).expect("valid json");
        assert_eq!(json["imported"], 1);
    }

    #[tokio::test]
    async fn import_csv_flighty_idempotent_reimport() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;

        for _ in 0..2 {
            let app = create_router(test_app_state(pool.clone()));
            let response = app
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri("/import/csv?format=flighty")
                        .header(header::COOKIE, &cookie)
                        .body(Body::from(FLIGHTY_CSV))
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
    async fn import_csv_flighty_multipart_redirects_with_count() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let app = create_router(test_app_state(pool.clone()));

        let boundary = "----TestBoundary7MA4YWxkTrZu0gW";
        let body = format!(
            "------TestBoundary7MA4YWxkTrZu0gW\r\n\
             Content-Disposition: form-data; name=\"format\"\r\n\r\n\
             flighty\r\n\
             ------TestBoundary7MA4YWxkTrZu0gW\r\n\
             Content-Disposition: form-data; name=\"file\"; filename=\"test.csv\"\r\n\
             Content-Type: text/csv\r\n\r\n\
             {FLIGHTY_CSV}\r\n\
             ------TestBoundary7MA4YWxkTrZu0gW--\r\n"
        );

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/import/csv")
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
            location.starts_with("/settings?csv="),
            "expected redirect to /settings?csv=N, got: {location}"
        );
    }

    #[tokio::test]
    async fn import_csv_unknown_format_returns_bad_request() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let app = create_router(test_app_state(pool));

        let bad_csv = "col1,col2,col3\na,b,c\n";

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/import/csv")
                    .header(header::COOKIE, &cookie)
                    .body(Body::from(bad_csv))
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}

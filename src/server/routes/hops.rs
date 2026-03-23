use super::ErrorResponse;
use super::types::{
    MultiFormatResponse, add_multi_format_docs, multi_format_docs, negotiate_format,
};
use crate::{
    db,
    server::{AppState, middleware::AuthUser, session::is_form_request},
};
use aide::transform::TransformOperation;
use axum::{
    Json,
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Redirect, Response},
};
use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// API response type for a single travel hop.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct HopResponse {
    /// Mode of transport for this hop.
    pub travel_type: HopTravelType,
    /// Name of the origin location (airport code, city, or station).
    pub origin_name: String,
    /// Latitude of the origin location.
    pub origin_lat: f64,
    /// Longitude of the origin location.
    pub origin_lng: f64,
    /// Name of the destination location (airport code, city, or station).
    pub dest_name: String,
    /// Latitude of the destination location.
    pub dest_lat: f64,
    /// Longitude of the destination location.
    pub dest_lng: f64,
    /// Departure date in `YYYY-MM-DD` format.
    pub start_date: String,
    /// Arrival date in `YYYY-MM-DD` format.
    pub end_date: String,
}

/// Mode of transport for a travel hop.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum HopTravelType {
    /// Flight segment.
    #[default]
    Air,
    /// Train or rail segment.
    Rail,
    /// Cruise or ferry segment.
    Cruise,
    /// Ground transport (car, bus, taxi, etc.).
    Transport,
}

impl HopTravelType {
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Air => "air",
            Self::Rail => "rail",
            Self::Cruise => "cruise",
            Self::Transport => "transport",
        }
    }

    #[must_use]
    pub const fn emoji(&self) -> &'static str {
        match self {
            Self::Air => "✈️",
            Self::Rail => "🚆",
            Self::Cruise => "🚢",
            Self::Transport => "🚗",
        }
    }
}

impl std::fmt::Display for HopTravelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Air => write!(f, "air"),
            Self::Rail => write!(f, "rail"),
            Self::Cruise => write!(f, "cruise"),
            Self::Transport => write!(f, "transport"),
        }
    }
}

impl From<db::hops::TravelType> for HopTravelType {
    fn from(t: db::hops::TravelType) -> Self {
        match t {
            db::hops::TravelType::Air => Self::Air,
            db::hops::TravelType::Rail => Self::Rail,
            db::hops::TravelType::Cruise => Self::Cruise,
            db::hops::TravelType::Transport => Self::Transport,
        }
    }
}

impl From<db::hops::Row> for HopResponse {
    fn from(hop: db::hops::Row) -> Self {
        Self {
            travel_type: hop.travel_type.into(),
            origin_name: hop.origin_name,
            origin_lat: hop.origin_lat,
            origin_lng: hop.origin_lng,
            dest_name: hop.dest_name,
            dest_lat: hop.dest_lat,
            dest_lng: hop.dest_lng,
            start_date: hop.start_date,
            end_date: hop.end_date,
        }
    }
}

impl MultiFormatResponse for HopResponse {
    const HTML_TITLE: &'static str = "Travel Hops";

    const CSV_HEADERS: &'static [&'static str] = &[
        "travel_type",
        "origin_name",
        "origin_lat",
        "origin_lng",
        "dest_name",
        "dest_lat",
        "dest_lng",
        "start_date",
        "end_date",
    ];

    fn csv_row(&self) -> Vec<String> {
        vec![
            self.travel_type.to_string(),
            self.origin_name.clone(),
            self.origin_lat.to_string(),
            self.origin_lng.to_string(),
            self.dest_name.clone(),
            self.dest_lat.to_string(),
            self.dest_lng.to_string(),
            self.start_date.clone(),
            self.end_date.clone(),
        ]
    }

    fn html_card(&self) -> String {
        let emoji = self.travel_type.emoji();
        let travel_type = self.travel_type.as_str();
        let origin = &self.origin_name;
        let dest = &self.dest_name;
        let date = &self.start_date;

        format!(
            "<div class=\"data-card hop-card\">\
             <div class=\"hop-card-route\">\
             <span class=\"hop-card-place\">{origin}</span>\
             <span class=\"hop-card-arrow\">→</span>\
             <span class=\"hop-card-place\">{dest}</span>\
             </div>\
             <div class=\"hop-card-meta\">\
             <span class=\"hop-card-badge\">{emoji} {travel_type}</span>\
             <span class=\"hop-card-date\">{date}</span>\
             </div>\
             </div>"
        )
    }
}

#[derive(Deserialize, JsonSchema)]
pub struct HopQuery {
    #[serde(rename = "type")]
    travel_type: Option<HopTravelType>,
}

pub async fn hops_handler(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(query): Query<HopQuery>,
    headers: HeaderMap,
) -> Response {
    let format = negotiate_format(&headers);
    let hops = match (db::hops::GetAll {
        user_id: auth.user_id,
        travel_type_filter: query.travel_type.as_ref().map(HopTravelType::as_str),
    })
    .execute(&state.db)
    .await
    {
        Ok(hops) => hops,
        Err(err) => {
            return ErrorResponse::into_format_response(
                format!("failed to fetch hops: {err}"),
                format,
                StatusCode::INTERNAL_SERVER_ERROR,
            );
        }
    };

    let responses: Vec<HopResponse> = hops.into_iter().map(HopResponse::from).collect();
    HopResponse::into_format_response(&responses, format, StatusCode::OK)
}

pub fn hops_handler_docs(op: TransformOperation) -> TransformOperation {
    multi_format_docs!(
        op.description("List travel hops for the authenticated user.")
            .response_with::<200, Json<Vec<HopResponse>>, _>(|mut res| {
                add_multi_format_docs::<HopResponse>(res.inner());
                res
            }),
        401 | 500 => ErrorResponse,
    )
    .tag("hops")
}

const QUERY_ENCODE_SET: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'_')
    .remove(b'.')
    .remove(b'~');

fn encode_query_value(s: &str) -> String {
    utf8_percent_encode(s, QUERY_ENCODE_SET).to_string()
}

/// Request body for manually creating a flight hop.
#[derive(Deserialize, JsonSchema)]
pub struct CreateHopRequest {
    /// IATA airport code for the origin (e.g. "LHR").
    pub origin: String,
    /// IATA airport code for the destination (e.g. "JFK").
    pub destination: String,
    /// Departure date in `YYYY-MM-DD` format.
    pub date: String,
    /// Airline name or IATA code.
    #[serde(default)]
    pub airline: Option<String>,
    /// Flight number (e.g. "BA117").
    #[serde(default)]
    pub flight_number: Option<String>,
    /// Aircraft type (e.g. "Boeing 777-300ER").
    #[serde(default)]
    pub aircraft_type: Option<String>,
    /// Cabin class (e.g. "Economy", "Business").
    #[serde(default)]
    pub cabin_class: Option<String>,
    /// Seat assignment.
    #[serde(default)]
    pub seat: Option<String>,
    /// Passenger Name Record / booking reference.
    #[serde(default)]
    pub pnr: Option<String>,
}

/// Successful response after creating a hop.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreateHopResponse {
    /// Number of hops created.
    pub created: u64,
}

/// Create a flight hop manually.
///
/// Accepts JSON or form-encoded body. Form submissions redirect to the add
/// flight page with a success or error query parameter.
pub async fn create_hop_handler(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> Response {
    let is_form = is_form_request(&headers);

    let parsed: Result<CreateHopRequest, String> = if is_form {
        serde_urlencoded::from_bytes(&body).map_err(|e| e.to_string())
    } else {
        serde_json::from_slice(&body).map_err(|e| e.to_string())
    };

    let req = match parsed {
        Ok(r) => r,
        Err(err) => {
            return if is_form {
                Redirect::to(&format!(
                    "/flights/new?error={}",
                    encode_query_value(&format!("Invalid form data: {err}"))
                ))
                .into_response()
            } else {
                let format = negotiate_format(&headers);
                ErrorResponse::into_format_response(
                    format!("invalid request body: {err}"),
                    format,
                    StatusCode::BAD_REQUEST,
                )
            };
        }
    };

    if req.origin.is_empty() || req.destination.is_empty() || req.date.is_empty() {
        let msg = "origin, destination, and date are required";
        return if is_form {
            Redirect::to(&format!("/flights/new?error={}", encode_query_value(msg))).into_response()
        } else {
            let format = negotiate_format(&headers);
            ErrorResponse::into_format_response(msg.to_string(), format, StatusCode::BAD_REQUEST)
        };
    }

    let detail = db::hops::FlightDetail {
        airline: req.airline.unwrap_or_default(),
        flight_number: req.flight_number.unwrap_or_default(),
        aircraft_type: req.aircraft_type.unwrap_or_default(),
        cabin_class: req.cabin_class.unwrap_or_default(),
        seat: req.seat.unwrap_or_default(),
        pnr: req.pnr.unwrap_or_default(),
    };

    let result = (db::hops::CreateManual {
        user_id: auth.user_id,
        origin: req.origin,
        destination: req.destination,
        date: req.date,
        flight_detail: detail,
    })
    .execute(&state.db)
    .await;

    match result {
        Ok(created) => {
            if is_form {
                Redirect::to("/flights/new?success=1").into_response()
            } else {
                (StatusCode::CREATED, Json(CreateHopResponse { created })).into_response()
            }
        }
        Err(err) => {
            let msg = format!("failed to create hop: {err}");
            if is_form {
                Redirect::to(&format!("/flights/new?error={}", encode_query_value(&msg)))
                    .into_response()
            } else {
                let format = negotiate_format(&headers);
                ErrorResponse::into_format_response(msg, format, StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}

pub fn create_hop_handler_docs(op: TransformOperation) -> TransformOperation {
    op.description("Create a flight hop manually. Accepts JSON or form-encoded body.")
        .input::<Json<CreateHopRequest>>()
        .with(|mut op| {
            if let Some(aide::openapi::ReferenceOr::Item(body)) = &mut op.inner_mut().request_body
                && let Some(json_media) = body.content.get("application/json").cloned()
            {
                body.content
                    .insert("application/x-www-form-urlencoded".to_string(), json_media);
            }
            op
        })
        .response::<201, Json<CreateHopResponse>>()
        .response::<400, Json<ErrorResponse>>()
        .response::<401, Json<ErrorResponse>>()
        .response::<500, Json<ErrorResponse>>()
        .tag("hops")
}

#[cfg(test)]
mod tests {
    use super::{CreateHopResponse, HopResponse, HopTravelType};
    use crate::{
        db::{self, hops::TravelType},
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
        let user = db::users::GetByUsername { username: "alice" }
            .execute(&pool)
            .await
            .expect("lookup failed")
            .expect("missing user");
        db::hops::Create {
            trip_id: "trip-1",
            user_id: user.id,
            hops: &[sample_hop(
                TravelType::Air,
                "LHR",
                "JFK",
                "2024-01-01",
                "2024-01-01",
            )],
        }
        .execute(&pool)
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
        let user = db::users::GetByUsername { username: "alice" }
            .execute(&pool)
            .await
            .expect("lookup failed")
            .expect("missing user");
        db::hops::Create {
            trip_id: "trip-1",
            user_id: user.id,
            hops: &[sample_hop(
                TravelType::Rail,
                "Paris",
                "London",
                "2024-01-01",
                "2024-01-01",
            )],
        }
        .execute(&pool)
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
        let user = db::users::GetByUsername { username: "alice" }
            .execute(&pool)
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
        db::hops::Create {
            trip_id: "trip-1",
            user_id: user.id,
            hops: &hops,
        }
        .execute(&pool)
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
        let parsed: Vec<HopResponse> =
            serde_json::from_slice(&body).expect("body should be valid JSON array");

        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].travel_type, HopTravelType::Rail);
    }

    #[tokio::test]
    async fn get_hops_json_filters_by_type() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let user = db::users::GetByUsername { username: "alice" }
            .execute(&pool)
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
        db::hops::Create {
            trip_id: "trip-1",
            user_id: user.id,
            hops: &hops,
        }
        .execute(&pool)
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
        let parsed: Vec<HopResponse> = serde_json::from_slice(&body).expect("valid json");
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].travel_type, HopTravelType::Rail);
    }

    #[tokio::test]
    async fn get_hops_with_accept_csv_returns_csv() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let user = db::users::GetByUsername { username: "alice" }
            .execute(&pool)
            .await
            .expect("lookup failed")
            .expect("missing user");
        db::hops::Create {
            trip_id: "trip-1",
            user_id: user.id,
            hops: &[sample_hop(
                TravelType::Rail,
                "Paris",
                "London",
                "2024-01-01",
                "2024-01-01",
            )],
        }
        .execute(&pool)
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
        let user = db::users::GetByUsername { username: "alice" }
            .execute(&pool)
            .await
            .expect("lookup failed")
            .expect("missing user");
        db::hops::Create {
            trip_id: "trip-1",
            user_id: user.id,
            hops: &[sample_hop(
                TravelType::Air,
                "LHR",
                "JFK",
                "2024-02-01",
                "2024-02-01",
            )],
        }
        .execute(&pool)
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
        let user = db::users::GetByUsername { username: "alice" }
            .execute(&pool)
            .await
            .expect("lookup failed")
            .expect("missing user");
        db::hops::Create {
            trip_id: "trip-2",
            user_id: user.id,
            hops: &[sample_hop(
                TravelType::Rail,
                "Paris",
                "London",
                "2024-01-01",
                "2024-01-01",
            )],
        }
        .execute(&pool)
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
        assert!(body.contains("data-card"));
        assert!(body.contains("Travel Hops"));
        assert!(body.contains("Paris"));
        assert!(body.contains("London"));
    }

    #[tokio::test]
    async fn post_hops_without_auth_returns_unauthorized() {
        let pool = test_pool().await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/hops")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(
                        r#"{"origin":"DUB","destination":"LHR","date":"2025-06-15"}"#,
                    ))
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn post_hops_json_creates_hop() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let app = create_router(test_app_state(pool.clone()));

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/hops")
                    .header(header::COOKIE, &cookie)
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(
                        r#"{"origin":"DUB","destination":"LHR","date":"2025-06-15","airline":"Aer Lingus","flight_number":"EI154"}"#,
                    ))
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::CREATED);
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("failed to read body");
        let parsed: CreateHopResponse = serde_json::from_slice(&body).expect("valid json response");
        assert_eq!(parsed.created, 1);

        let user = db::users::GetByUsername { username: "alice" }
            .execute(&pool)
            .await
            .expect("lookup failed")
            .expect("missing user");
        let hops = db::hops::GetAll {
            user_id: user.id,
            travel_type_filter: Some("air"),
        }
        .execute(&pool)
        .await
        .expect("fetch failed");
        assert_eq!(hops.len(), 1);
        assert_eq!(hops[0].origin_name, "DUB");
        assert_eq!(hops[0].dest_name, "LHR");
    }

    #[tokio::test]
    async fn post_hops_form_redirects_on_success() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/hops")
                    .header(header::COOKIE, &cookie)
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .body(Body::from("origin=DUB&destination=LHR&date=2025-06-15"))
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
        assert_eq!(location, "/flights/new?success=1");
    }

    #[tokio::test]
    async fn post_hops_json_missing_fields_returns_bad_request() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/hops")
                    .header(header::COOKIE, &cookie)
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(
                        r#"{"origin":"","destination":"LHR","date":"2025-06-15"}"#,
                    ))
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn get_flights_new_page_returns_add_flight_form() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/flights/new")
                    .header(header::COOKIE, cookie)
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_text(response).await;
        assert!(
            body.contains("Add Flight"),
            "page should contain 'Add Flight'"
        );
    }
}

use super::ErrorResponse;
use super::types::{
    MultiFormatResponse, add_multi_format_docs, multi_format_docs, negotiate_format,
    opt_f64_to_string,
};
use crate::{
    db,
    server::{AppState, middleware::AuthUser},
};
use aide::transform::TransformOperation;
use axum::{
    Json,
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::Response,
};
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
    pub origin_lat: Option<f64>,
    /// Longitude of the origin location.
    pub origin_lng: Option<f64>,
    /// Name of the destination location (airport code, city, or station).
    pub dest_name: String,
    /// Latitude of the destination location.
    pub dest_lat: Option<f64>,
    /// Longitude of the destination location.
    pub dest_lng: Option<f64>,
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
            opt_f64_to_string(self.origin_lat),
            opt_f64_to_string(self.origin_lng),
            self.dest_name.clone(),
            opt_f64_to_string(self.dest_lat),
            opt_f64_to_string(self.dest_lng),
            self.start_date.clone(),
            self.end_date.clone(),
        ]
    }

    fn html_cells(&self) -> Vec<String> {
        vec![
            format!("{} {}", self.travel_type.emoji(), self.travel_type),
            self.origin_name.clone(),
            opt_f64_to_string(self.origin_lat),
            opt_f64_to_string(self.origin_lng),
            self.dest_name.clone(),
            opt_f64_to_string(self.dest_lat),
            opt_f64_to_string(self.dest_lng),
            self.start_date.clone(),
            self.end_date.clone(),
        ]
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

#[cfg(test)]
mod tests {
    use super::{HopResponse, HopTravelType};
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
        assert!(body.contains("<table>"));
        assert!(body.contains("Travel Hops"));
        assert!(body.contains("travel_type"));
        assert!(body.contains("origin_name"));
        assert!(body.contains("dest_name"));
        assert!(body.contains("Paris"));
        assert!(body.contains("London"));
    }
}

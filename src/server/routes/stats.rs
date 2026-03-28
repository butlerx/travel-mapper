use super::{ErrorResponse, MultiFormatResponse, multi_format_docs, negotiate_format};
use crate::{
    db,
    server::{
        AppState,
        error::AppError,
        extractors::AuthUser,
        pages::stats::{CountedItem, StatsQuery, compute_detailed_stats},
    },
};
use aide::transform::TransformOperation;
use axum::{
    Json,
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::Response,
};
use schemars::JsonSchema;
use serde::Serialize;

/// A ranked entry in a stats breakdown (airlines, routes, countries, etc.).
#[derive(Debug, Default, Serialize, JsonSchema)]
pub struct RankedItem {
    pub name: String,
    pub count: usize,
}

impl From<CountedItem> for RankedItem {
    fn from(item: CountedItem) -> Self {
        Self {
            name: item.name,
            count: item.count,
        }
    }
}

/// Aggregated travel statistics for the authenticated user.
#[derive(Debug, Default, Serialize, JsonSchema)]
pub struct StatsResponse {
    pub total_journeys: usize,
    pub total_flights: usize,
    pub total_rail: usize,
    pub total_boat: usize,
    pub total_transport: usize,
    pub total_distance_km: u64,
    pub unique_airports: usize,
    pub unique_countries: usize,
    pub top_airlines: Vec<RankedItem>,
    pub top_aircraft: Vec<RankedItem>,
    pub top_routes: Vec<RankedItem>,
    pub cabin_class_breakdown: Vec<RankedItem>,
    pub seat_type_breakdown: Vec<RankedItem>,
    pub flight_reason_breakdown: Vec<RankedItem>,
    pub countries: Vec<RankedItem>,
    pub available_years: Vec<String>,
    pub selected_year: Option<String>,
    pub first_year: Option<String>,
    pub last_year: Option<String>,
}

impl MultiFormatResponse for StatsResponse {
    const HTML_TITLE: &'static str = "Stats";
    const CSV_HEADERS: &'static [&'static str] = &[
        "total_journeys",
        "total_flights",
        "total_rail",
        "total_boat",
        "total_transport",
        "total_distance_km",
        "unique_airports",
        "unique_countries",
    ];

    fn csv_row(&self) -> Vec<String> {
        vec![
            self.total_journeys.to_string(),
            self.total_flights.to_string(),
            self.total_rail.to_string(),
            self.total_boat.to_string(),
            self.total_transport.to_string(),
            self.total_distance_km.to_string(),
            self.unique_airports.to_string(),
            self.unique_countries.to_string(),
        ]
    }
}

fn convert_items(items: Vec<CountedItem>) -> Vec<RankedItem> {
    items.into_iter().map(RankedItem::from).collect()
}

impl From<crate::server::pages::stats::DetailedStats> for StatsResponse {
    fn from(s: crate::server::pages::stats::DetailedStats) -> Self {
        Self {
            total_journeys: s.total_journeys,
            total_flights: s.total_flights,
            total_rail: s.total_rail,
            total_boat: s.total_boat,
            total_transport: s.total_transport,
            total_distance_km: s.total_distance_km,
            unique_airports: s.unique_airports,
            unique_countries: s.unique_countries,
            top_airlines: convert_items(s.top_airlines),
            top_aircraft: convert_items(s.top_aircraft),
            top_routes: convert_items(s.top_routes),
            cabin_class_breakdown: convert_items(s.cabin_class_breakdown),
            seat_type_breakdown: convert_items(s.seat_type_breakdown),
            flight_reason_breakdown: convert_items(s.flight_reason_breakdown),
            countries: convert_items(s.countries),
            available_years: s.available_years,
            selected_year: s.selected_year,
            first_year: s.first_year,
            last_year: s.last_year,
        }
    }
}

pub async fn handler(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(query): Query<StatsQuery>,
    headers: HeaderMap,
) -> Response {
    let format = negotiate_format(&headers);

    let all_rows = match (db::hops::GetAllForStats {
        user_id: auth.user_id,
    })
    .execute(&state.db)
    .await
    {
        Ok(rows) => rows,
        Err(err) => return AppError::from(err).into_format_response(format),
    };

    let detailed = compute_detailed_stats(&all_rows, query.year.as_deref());

    if format == super::ResponseFormat::Html {
        crate::server::pages::stats::render_page(detailed)
    } else {
        let response = StatsResponse::from(detailed);
        StatsResponse::single_format_response(&response, format, StatusCode::OK)
    }
}

/// `OpenAPI` metadata for the stats endpoint.
pub fn handler_docs(op: TransformOperation) -> TransformOperation {
    multi_format_docs!(
        op.description("Aggregated travel statistics for the authenticated user.")
            .response_with::<200, Json<StatsResponse>, _>(|mut res| {
                super::add_multi_format_docs::<StatsResponse>(res.inner());
                res
            }),
        401 | 500 => ErrorResponse,
    )
    .tag("stats")
}

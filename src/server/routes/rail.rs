use super::{ErrorResponse, MultiFormatResponse, multi_format_docs, negotiate_format};
use crate::{
    integrations::transitland::{client::TransitlandClient, feed_discovery},
    server::AppState,
};
use aide::transform::TransformOperation;
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::Response,
};
use schemars::JsonSchema;
use serde::Serialize;

#[derive(Debug, Clone, Default, Serialize, JsonSchema)]
pub struct OperatorResponse {
    pub onestop_id: String,
    pub display_name: String,
    pub country_code: String,
}

impl MultiFormatResponse for OperatorResponse {
    const HTML_TITLE: &'static str = "Rail Operators";
    const CSV_HEADERS: &'static [&'static str] = &["onestop_id", "display_name", "country_code"];

    fn csv_row(&self) -> Vec<String> {
        vec![
            self.onestop_id.clone(),
            self.display_name.clone(),
            self.country_code.clone(),
        ]
    }
}

#[derive(Debug, Clone, Default, Serialize, JsonSchema)]
pub struct FeedResponse {
    pub id: i64,
    pub onestop_id: String,
    pub name: Option<String>,
    pub spec: String,
    pub static_url: Option<String>,
    pub realtime_trip_updates_url: Option<String>,
    pub realtime_vehicle_positions_url: Option<String>,
    pub realtime_alerts_url: Option<String>,
}

impl MultiFormatResponse for FeedResponse {
    const HTML_TITLE: &'static str = "GTFS-RT Feeds";
    const CSV_HEADERS: &'static [&'static str] = &[
        "id",
        "onestop_id",
        "name",
        "spec",
        "static_url",
        "realtime_trip_updates_url",
        "realtime_vehicle_positions_url",
        "realtime_alerts_url",
    ];

    fn csv_row(&self) -> Vec<String> {
        vec![
            self.id.to_string(),
            self.onestop_id.clone(),
            self.name.clone().unwrap_or_default(),
            self.spec.clone(),
            self.static_url.clone().unwrap_or_default(),
            self.realtime_trip_updates_url.clone().unwrap_or_default(),
            self.realtime_vehicle_positions_url
                .clone()
                .unwrap_or_default(),
            self.realtime_alerts_url.clone().unwrap_or_default(),
        ]
    }
}

pub async fn list_operators_handler(headers: HeaderMap) -> Response {
    let format = negotiate_format(&headers);
    let operators: Vec<OperatorResponse> = feed_discovery::supported_operators()
        .into_iter()
        .map(|op| OperatorResponse {
            onestop_id: op.onestop_id().to_owned(),
            display_name: op.display_name().to_owned(),
            country_code: op.country_code().to_owned(),
        })
        .collect();
    OperatorResponse::into_format_response(&operators, format, StatusCode::OK)
}

pub fn list_operators_handler_docs(op: TransformOperation) -> TransformOperation {
    let op = op
        .summary("List supported rail operators")
        .description("Returns all rail operators with known GTFS-RT feed availability.")
        .tag("rail");
    multi_format_docs!(op, 200 => OperatorResponse)
}

pub async fn operator_feeds_handler(
    State(state): State<AppState>,
    Path(onestop_id): Path<String>,
    headers: HeaderMap,
) -> Response {
    let format = negotiate_format(&headers);

    let Some(operator) = feed_discovery::supported_operators()
        .into_iter()
        .find(|op| op.onestop_id() == onestop_id)
    else {
        return ErrorResponse::into_format_response(
            format!("unknown operator: {onestop_id}"),
            format,
            StatusCode::NOT_FOUND,
        );
    };

    let Some(ref api_key) = state.transitland_api_key else {
        return ErrorResponse::into_format_response(
            "Transitland API key not configured",
            format,
            StatusCode::SERVICE_UNAVAILABLE,
        );
    };

    let client = match TransitlandClient::new(api_key.clone()) {
        Ok(c) => c,
        Err(e) => {
            return ErrorResponse::into_format_response(
                format!("failed to create Transitland client: {e}"),
                format,
                StatusCode::INTERNAL_SERVER_ERROR,
            );
        }
    };

    match feed_discovery::discover_feeds_for_operator(&client, operator).await {
        Ok(result) => {
            let feeds: Vec<FeedResponse> = result
                .feeds
                .into_iter()
                .map(|f| FeedResponse {
                    id: f.id,
                    onestop_id: f.onestop_id,
                    name: f.name,
                    spec: f.spec,
                    static_url: f.urls.static_current,
                    realtime_trip_updates_url: f.urls.realtime_trip_updates,
                    realtime_vehicle_positions_url: f.urls.realtime_vehicle_positions,
                    realtime_alerts_url: f.urls.realtime_alerts,
                })
                .collect();
            FeedResponse::into_format_response(&feeds, format, StatusCode::OK)
        }
        Err(e) => ErrorResponse::into_format_response(
            format!("Transitland API error: {e}"),
            format,
            StatusCode::BAD_GATEWAY,
        ),
    }
}

pub fn operator_feeds_handler_docs(op: TransformOperation) -> TransformOperation {
    let op = op
        .summary("Discover GTFS-RT feeds for a rail operator")
        .description("Queries Transitland for available GTFS-RT feeds matching the given operator onestop ID.")
        .tag("rail");
    multi_format_docs!(op,
        200 => FeedResponse,
        404 | 503 | 502 => ErrorResponse,
    )
}

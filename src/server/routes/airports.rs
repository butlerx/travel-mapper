use super::{ErrorResponse, MultiFormatResponse, multi_format_docs, negotiate_format};
use aide::transform::TransformOperation;
use axum::{
    extract::Path,
    http::{HeaderMap, StatusCode},
    response::Response,
};
use schemars::JsonSchema;
use serde::Serialize;

/// Airport lookup response from the embedded IATA database.
#[derive(Debug, Clone, Default, Serialize, JsonSchema)]
pub struct AirportResponse {
    pub iata: String,
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub country_code: String,
    pub timezone: String,
}

impl MultiFormatResponse for AirportResponse {
    const HTML_TITLE: &'static str = "Airport";
    const CSV_HEADERS: &'static [&'static str] = &[
        "iata",
        "name",
        "latitude",
        "longitude",
        "country_code",
        "timezone",
    ];

    fn csv_row(&self) -> Vec<String> {
        vec![
            self.iata.clone(),
            self.name.clone(),
            self.latitude.to_string(),
            self.longitude.to_string(),
            self.country_code.clone(),
            self.timezone.clone(),
        ]
    }
}

/// Look up enriched airport data by IATA code.
pub async fn handler(Path(iata): Path<String>, headers: HeaderMap) -> Response {
    let format = negotiate_format(&headers);

    let Some(airport) = crate::geocode::airports::lookup_enriched(&iata) else {
        return ErrorResponse::into_format_response(
            format!("unknown IATA code: {iata}"),
            format,
            StatusCode::NOT_FOUND,
        );
    };

    let resp = AirportResponse {
        iata: airport.iata,
        name: airport.name,
        latitude: airport.latitude,
        longitude: airport.longitude,
        country_code: airport.country_code,
        timezone: airport.timezone,
    };

    AirportResponse::single_format_response(&resp, format, StatusCode::OK)
}

pub fn handler_docs(op: TransformOperation) -> TransformOperation {
    let op = op
        .summary("Look up airport by IATA code")
        .description("Return airport metadata (name, coordinates, country, timezone) for a three-letter IATA code.")
        .tag("airports");
    multi_format_docs!(op,
        200 => AirportResponse,
        404 => ErrorResponse,
    )
}

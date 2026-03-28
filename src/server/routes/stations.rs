use super::{ErrorResponse, MultiFormatResponse, multi_format_docs, negotiate_format};
use aide::transform::TransformOperation;
use axum::{
    extract::{Path, Query},
    http::{HeaderMap, StatusCode},
    response::Response,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Station lookup response from the embedded UK CRS database.
#[derive(Debug, Clone, Default, Serialize, JsonSchema)]
pub struct StationResponse {
    pub code: String,
    pub name: String,
}

impl MultiFormatResponse for StationResponse {
    const HTML_TITLE: &'static str = "Station";
    const CSV_HEADERS: &'static [&'static str] = &["code", "name"];

    fn csv_row(&self) -> Vec<String> {
        vec![self.code.clone(), self.name.clone()]
    }
}

/// Query parameters for station name lookup.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct StationLookupQuery {
    /// Station name to search for (exact match then fuzzy).
    pub name: String,
}

/// Look up a station by CRS code.
pub async fn get_by_code_handler(Path(crs): Path<String>, headers: HeaderMap) -> Response {
    let format = negotiate_format(&headers);

    let Some(name) = crate::geocode::stations::lookup_crs(&crs) else {
        return ErrorResponse::into_format_response(
            format!("unknown CRS code: {crs}"),
            format,
            StatusCode::NOT_FOUND,
        );
    };

    let resp = StationResponse {
        code: crs.to_ascii_uppercase(),
        name: name.to_owned(),
    };

    StationResponse::single_format_response(&resp, format, StatusCode::OK)
}

pub fn get_by_code_handler_docs(op: TransformOperation) -> TransformOperation {
    let op = op
        .summary("Look up station by CRS code")
        .description(
            "Return the station name for a three-letter UK CRS code. \
             Returns 404 if the code is not in the database.",
        )
        .tag("stations");
    multi_format_docs!(op,
        200 => StationResponse,
        404 => ErrorResponse,
    )
}

/// Search for a station CRS code by name (exact then fuzzy match).
pub async fn lookup_handler(
    Query(params): Query<StationLookupQuery>,
    headers: HeaderMap,
) -> Response {
    let format = negotiate_format(&headers);

    let Some(code) = crate::geocode::stations::crs_from_name(&params.name) else {
        return ErrorResponse::into_format_response(
            format!("no station found matching: {}", params.name),
            format,
            StatusCode::NOT_FOUND,
        );
    };

    let name = crate::geocode::stations::lookup_crs(code).unwrap_or_default();

    let resp = StationResponse {
        code: code.to_owned(),
        name: name.to_owned(),
    };

    StationResponse::single_format_response(&resp, format, StatusCode::OK)
}

pub fn lookup_handler_docs(op: TransformOperation) -> TransformOperation {
    let op = op
        .summary("Look up station CRS code by name")
        .description(
            "Search for a station by name. Tries exact match first (case-insensitive), \
             then falls back to Jaro-Winkler fuzzy matching (threshold 0.85). \
             Returns the matching station code and name, or 404 if no match.",
        )
        .tag("stations");
    multi_format_docs!(op,
        200 => StationResponse,
        404 => ErrorResponse,
    )
}

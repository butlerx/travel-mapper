//! HTTP route handlers and response formatting.

use crate::server::components::{NavBar, Shell};
use aide::{
    axum::{
        ApiRouter,
        routing::{delete_with, get_with, post_with, put_with},
    },
    openapi::{MediaType, SchemaObject},
};
use axum::{
    Json,
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
};
use leptos::prelude::*;
use schemars::JsonSchema;
use serde::Serialize;
use std::fmt::Write;

pub(super) mod api_keys;
/// Generic CSV/delimited import handler (Flighty, myFlightradar24, `OpenFlights`, App in the Air).
pub(super) mod csv_import;
pub(super) mod health;
pub(super) mod hops;
pub(super) mod login;
pub(super) mod logout;
pub(super) mod register;
pub(super) mod static_assets;
pub(super) mod sync;
pub(super) mod tripit_callback;
pub(super) mod tripit_connect;
pub(super) mod tripit_credentials;
pub(super) mod trips;

pub(super) use hops::HopResponse;
pub(super) use login::AuthResponse;

/// Chain `.response_with` calls that attach multi-format (CSV + HTML) docs.
///
/// Status codes that share a type can be collapsed with `|`:
///
/// ```ignore
/// multi_format_docs!(op,
///     200 => HealthResponse,
///     401 | 500 => ErrorResponse,
/// );
/// ```
macro_rules! multi_format_docs {
    ($op:expr, $($($status:literal)|+ => $ty:ty),* $(,)?) => {{
        $op
        $($(
            .response_with::<$status, axum::Json<$ty>, _>(|mut res| {
                $crate::server::routes::add_multi_format_docs::<$ty>(res.inner());
                res
            })
        )+)*
    }};
}

pub(crate) use multi_format_docs;

/// Generic error response returned by all endpoints on failure.
#[derive(Debug, Default, Serialize, JsonSchema)]
pub struct ErrorResponse {
    /// Human-readable error message describing what went wrong.
    pub error: String,
}

/// Generic success response with a status field.
#[derive(Debug, Default, Serialize, JsonSchema)]
pub struct StatusResponse {
    /// Result status, typically `"ok"`.
    pub status: String,
}

/// Supported response formats for content negotiation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResponseFormat {
    Json,
    Csv,
    Html,
}

/// Inspect the `Accept` header and return the best matching [`ResponseFormat`].
///
/// Falls back to JSON when no recognised media type is present.
pub fn negotiate_format(headers: &HeaderMap) -> ResponseFormat {
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

/// Trait for response types that can render to JSON, CSV and HTML.
///
/// Implement this on any `Serialize + Default` struct to gain multi-format
/// responses via [`MultiFormatResponse::into_format_response`].
/// `Default` is used to generate example output for the `OpenAPI` spec.
pub trait MultiFormatResponse: Serialize + Default + Sized {
    /// Page title shown in the HTML response wrapper.
    const HTML_TITLE: &'static str;

    /// CSV column headers in the order returned by [`csv_row`].
    const CSV_HEADERS: &'static [&'static str];

    /// Return the values for a single CSV row, aligned with [`CSV_HEADERS`].
    fn csv_row(&self) -> Vec<String>;

    /// Render a single item as a card HTML fragment.
    ///
    /// Default builds a generic key-value card from headers + cells.
    fn html_card(&self) -> String {
        let fields: String = Self::CSV_HEADERS
            .iter()
            .zip(self.csv_row())
            .filter(|(_, v)| !v.is_empty())
            .fold(String::new(), |mut acc, (h, v)| {
                let _ = write!(
                    acc,
                    "<div class=\"data-card-field\">\
                     <span class=\"data-card-label\">{h}</span>\
                     <span class=\"data-card-value\">{v}</span>\
                     </div>"
                );
                acc
            });
        format!("<div class=\"data-card\">{fields}</div>")
    }

    /// Build the final [`Response`] in the requested format.
    fn into_format_response(items: &[Self], format: ResponseFormat, status: StatusCode) -> Response
    where
        Self: Serialize,
    {
        match format {
            ResponseFormat::Json => (
                status,
                Json(serde_json::to_value(items).unwrap_or_default()),
            )
                .into_response(),
            ResponseFormat::Csv => build_csv::<Self>(items),
            ResponseFormat::Html => build_html::<Self>(items),
        }
    }

    /// Convenience wrapper for rendering a single item.
    fn single_format_response(item: &Self, format: ResponseFormat, status: StatusCode) -> Response
    where
        Self: Serialize,
    {
        match format {
            ResponseFormat::Json => {
                (status, Json(serde_json::to_value(item).unwrap_or_default())).into_response()
            }
            ResponseFormat::Csv => build_csv::<Self>(std::slice::from_ref(item)),
            ResponseFormat::Html => build_html::<Self>(std::slice::from_ref(item)),
        }
    }
}

fn build_csv<T: MultiFormatResponse>(items: &[T]) -> Response {
    let mut writer = csv::Writer::from_writer(Vec::new());
    if let Err(err) = writer.write_record(T::CSV_HEADERS) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("failed to write CSV header: {err}"),
        )
            .into_response();
    }

    for item in items {
        if let Err(err) = writer.write_record(item.csv_row()) {
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
                "attachment; filename=\"export.csv\"",
            ),
        ],
        body,
    )
        .into_response()
}

fn build_html<T: MultiFormatResponse>(items: &[T]) -> Response {
    let title = T::HTML_TITLE;
    let cards: String = items.iter().map(T::html_card).collect();
    let count = items.len();

    let html = view! {
        <Shell title=title.to_owned()>
            <NavBar current="hops" />
            <main class="data-page">
                <div class="data-page-header">
                    <h1>{title}</h1>
                    <span class="data-page-count">{format!("{count} records")}</span>
                </div>
                <div class="data-card-list" inner_html=cards />
            </main>
        </Shell>
    };

    axum::response::Html(html.to_html()).into_response()
}

/// Add `text/csv` and `text/html` media types with derived examples to an
/// `OpenAPI` response object.
pub fn add_multi_format_docs<T: MultiFormatResponse>(response: &mut aide::openapi::Response) {
    let string_schema = SchemaObject {
        json_schema: schemars::Schema::from(serde_json::Map::from_iter([(
            "type".to_owned(),
            serde_json::Value::String("string".to_owned()),
        )])),
        external_docs: None,
        example: None,
    };

    let sample = T::default();
    let row: Vec<String> = sample
        .csv_row()
        .into_iter()
        .map(|v| {
            if v.is_empty() {
                "string".to_string()
            } else {
                v
            }
        })
        .collect();

    let csv_example = {
        let mut lines = T::CSV_HEADERS.join(",");
        lines.push('\n');
        lines.push_str(&row.join(","));
        lines.push('\n');
        lines
    };

    let html_example = {
        let ths: String = T::CSV_HEADERS.iter().fold(String::new(), |mut acc, h| {
            let _ = write!(acc, "<th>{h}</th>");
            acc
        });
        let tds: String = row.iter().fold(String::new(), |mut acc, c| {
            let _ = write!(acc, "<td>{c}</td>");
            acc
        });
        format!("<table><thead><tr>{ths}</tr></thead><tbody><tr>{tds}</tr></tbody></table>")
    };

    response.content.insert(
        "text/csv".to_string(),
        MediaType {
            schema: Some(string_schema.clone()),
            example: Some(serde_json::Value::String(csv_example)),
            ..Default::default()
        },
    );
    response.content.insert(
        "text/html".to_string(),
        MediaType {
            schema: Some(string_schema),
            example: Some(serde_json::Value::String(html_example)),
            ..Default::default()
        },
    );
}

impl MultiFormatResponse for StatusResponse {
    const HTML_TITLE: &'static str = "Status";
    const CSV_HEADERS: &'static [&'static str] = &["status"];

    fn csv_row(&self) -> Vec<String> {
        vec![self.status.clone()]
    }
}

impl MultiFormatResponse for ErrorResponse {
    const HTML_TITLE: &'static str = "Error";
    const CSV_HEADERS: &'static [&'static str] = &["error"];

    fn csv_row(&self) -> Vec<String> {
        vec![self.error.clone()]
    }
}

/// Authentication API routes, nested under `/auth`.
pub(super) fn auth_api_routes() -> ApiRouter<super::AppState> {
    ApiRouter::new()
        .api_route(
            "/register",
            post_with(register::handler, register::handler_docs),
        )
        .api_route("/login", post_with(login::handler, login::handler_docs))
        .api_route("/logout", post_with(logout::handler, logout::handler_docs))
        .api_route(
            "/api-keys",
            post_with(api_keys::handler, api_keys::handler_docs),
        )
}

/// Hop API routes, nested under `/hops`.
pub(super) fn hops_api_routes() -> ApiRouter<super::AppState> {
    ApiRouter::new()
        .api_route(
            "/",
            get_with(hops::handler, hops::handler_docs)
                .post_with(hops::create_handler, hops::create_handler_docs),
        )
        .api_route(
            "/{id}",
            put_with(hops::update_handler, hops::update_handler_docs)
                .post_with(hops::update_handler, hops::update_handler_docs),
        )
}

/// Trip API routes, nested under `/trips`.
pub(super) fn trip_api_routes() -> ApiRouter<super::AppState> {
    ApiRouter::new()
        .api_route(
            "/",
            post_with(trips::create_handler, trips::create_handler_docs),
        )
        .api_route(
            "/{id}",
            put_with(trips::update_handler, trips::update_handler_docs)
                .delete_with(trips::delete_handler, trips::delete_handler_docs)
                .post_with(trips::update_handler, trips::update_handler_docs),
        )
        .api_route(
            "/auto-group",
            post_with(trips::auto_group_handler, trips::auto_group_handler_docs),
        )
        .api_route(
            "/{id}/hops",
            post_with(trips::assign_hop_handler, trips::assign_hop_handler_docs),
        )
        .api_route(
            "/{id}/hops/{hop_id}",
            delete_with(
                trips::unassign_hop_handler,
                trips::unassign_hop_handler_docs,
            )
            .post_with(
                trips::unassign_hop_handler,
                trips::unassign_hop_handler_docs,
            ),
        )
}

/// `TripIt` integration routes, nested under `/auth/tripit`.
pub(super) fn tripit_api_routes() -> ApiRouter<super::AppState> {
    ApiRouter::new()
        .api_route(
            "/",
            put_with(
                tripit_credentials::handler,
                tripit_credentials::handler_docs,
            ),
        )
        .api_route(
            "/connect",
            get_with(tripit_connect::handler, tripit_connect::handler_docs),
        )
        .api_route(
            "/callback",
            get_with(tripit_callback::handler, tripit_callback::handler_docs),
        )
}

/// Import API routes, nested under `/import`.
pub(super) fn import_api_routes() -> ApiRouter<super::AppState> {
    ApiRouter::new().route("/csv", axum::routing::post(csv_import::handler))
}

/// Top-level API routes that don't belong to a nested group.
pub(super) fn toplevel_api_routes() -> ApiRouter<super::AppState> {
    ApiRouter::new()
        .api_route("/health", get_with(health::handler, health::handler_docs))
        .api_route("/sync", post_with(sync::handler, sync::handler_docs))
}

impl ErrorResponse {
    pub fn into_format_response(
        msg: impl Into<String>,
        format: ResponseFormat,
        status: StatusCode,
    ) -> Response {
        let err = Self { error: msg.into() };
        <Self as MultiFormatResponse>::single_format_response(&err, format, status)
    }
}

use super::{ErrorResponse, MultiFormatResponse, multi_format_docs, negotiate_format};
use crate::{
    db,
    server::{AppState, extractors::AuthUser},
};
use aide::transform::TransformOperation;
use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::Response,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct EnrichmentResponse {
    pub id: i64,
    pub hop_id: i64,
    pub provider: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delay_minutes: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dep_gate: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dep_terminal: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arr_gate: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arr_terminal: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dep_platform: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arr_platform: Option<String>,
    pub fetched_at: String,
    pub is_fresh: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_json: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct EnrichmentQuery {
    #[serde(default)]
    pub include_raw: bool,
}

fn non_empty(s: &str) -> Option<String> {
    if s.is_empty() {
        None
    } else {
        Some(s.to_owned())
    }
}

fn compute_freshness(fetched_at: &str, start_date: &str) -> bool {
    let Ok(fetched) = chrono::NaiveDateTime::parse_from_str(fetched_at, "%Y-%m-%d %H:%M:%S") else {
        return false;
    };
    let ttl = crate::worker::departure_aware_ttl(start_date);
    let age = chrono::Utc::now().naive_utc() - fetched;
    age.num_seconds() < ttl
}

impl EnrichmentResponse {
    fn from_row(row: db::status_enrichments::Row, start_date: &str, include_raw: bool) -> Self {
        Self {
            id: row.id,
            hop_id: row.hop_id,
            provider: row.provider,
            status: row.status.clone(),
            delay_minutes: row.delay_minutes,
            dep_gate: non_empty(&row.dep_gate),
            dep_terminal: non_empty(&row.dep_terminal),
            arr_gate: non_empty(&row.arr_gate),
            arr_terminal: non_empty(&row.arr_terminal),
            dep_platform: non_empty(&row.dep_platform),
            arr_platform: non_empty(&row.arr_platform),
            fetched_at: row.fetched_at.clone(),
            is_fresh: compute_freshness(&row.fetched_at, start_date),
            raw_json: if include_raw {
                Some(row.raw_json)
            } else {
                None
            },
        }
    }
}

impl MultiFormatResponse for EnrichmentResponse {
    const HTML_TITLE: &'static str = "Enrichments";
    const CSV_HEADERS: &'static [&'static str] = &[
        "id",
        "hop_id",
        "provider",
        "status",
        "delay_minutes",
        "dep_gate",
        "dep_terminal",
        "arr_gate",
        "arr_terminal",
        "dep_platform",
        "arr_platform",
        "fetched_at",
        "is_fresh",
    ];

    fn csv_row(&self) -> Vec<String> {
        vec![
            self.id.to_string(),
            self.hop_id.to_string(),
            self.provider.clone(),
            self.status.clone(),
            self.delay_minutes
                .map_or_else(String::new, |d| d.to_string()),
            self.dep_gate.clone().unwrap_or_default(),
            self.dep_terminal.clone().unwrap_or_default(),
            self.arr_gate.clone().unwrap_or_default(),
            self.arr_terminal.clone().unwrap_or_default(),
            self.dep_platform.clone().unwrap_or_default(),
            self.arr_platform.clone().unwrap_or_default(),
            self.fetched_at.clone(),
            self.is_fresh.to_string(),
        ]
    }
}

pub async fn handler(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(journey_id): Path<i64>,
    Query(params): Query<EnrichmentQuery>,
    headers: HeaderMap,
) -> Response {
    let format = negotiate_format(&headers);

    let hop = match (db::hops::GetById {
        id: journey_id,
        user_id: auth.user_id,
    })
    .execute(&state.db)
    .await
    {
        Ok(Some(h)) => h,
        Ok(None) => {
            return ErrorResponse::into_format_response(
                "journey not found",
                format,
                StatusCode::NOT_FOUND,
            );
        }
        Err(e) => {
            return ErrorResponse::into_format_response(
                format!("database error: {e}"),
                format,
                StatusCode::INTERNAL_SERVER_ERROR,
            );
        }
    };

    let rows = match (db::status_enrichments::GetAllByHopId { hop_id: journey_id })
        .execute(&state.db)
        .await
    {
        Ok(rows) => rows,
        Err(e) => {
            return ErrorResponse::into_format_response(
                format!("database error: {e}"),
                format,
                StatusCode::INTERNAL_SERVER_ERROR,
            );
        }
    };

    let items: Vec<EnrichmentResponse> = rows
        .into_iter()
        .map(|r| EnrichmentResponse::from_row(r, &hop.start_date, params.include_raw))
        .collect();

    EnrichmentResponse::into_format_response(&items, format, StatusCode::OK)
}

pub fn handler_docs(op: TransformOperation) -> TransformOperation {
    let op = op
        .summary("List enrichments for a journey")
        .description(
            "Return all status enrichment records (one per provider) for a journey. \
             Pass `?include_raw=true` to include the raw provider JSON.",
        )
        .tag("enrichments");
    multi_format_docs!(op,
        200 => EnrichmentResponse,
        404 | 500 => ErrorResponse,
    )
}

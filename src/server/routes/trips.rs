use super::{
    ErrorResponse, MultiFormatResponse, StatusResponse, multi_format_docs, negotiate_format,
};
use crate::{
    db,
    server::{AppState, error::AppError, extractors::AuthUser, session::is_form_request},
};
use aide::transform::TransformOperation;
use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Redirect, Response},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

#[derive(Debug, Default, Serialize, JsonSchema)]
pub struct TripResponse {
    pub id: i64,
    pub name: String,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub hop_count: i64,
}

impl From<db::trips::Row> for TripResponse {
    fn from(row: db::trips::Row) -> Self {
        Self {
            id: row.id,
            name: row.name,
            start_date: row.start_date,
            end_date: row.end_date,
            hop_count: row.hop_count,
        }
    }
}

impl MultiFormatResponse for TripResponse {
    const HTML_TITLE: &'static str = "Trips";
    const CSV_HEADERS: &'static [&'static str] =
        &["id", "name", "start_date", "end_date", "hop_count"];

    fn csv_row(&self) -> Vec<String> {
        vec![
            self.id.to_string(),
            self.name.clone(),
            self.start_date.clone().unwrap_or_default(),
            self.end_date.clone().unwrap_or_default(),
            self.hop_count.to_string(),
        ]
    }

    fn html_card(&self) -> String {
        let date_range = match (&self.start_date, &self.end_date) {
            (Some(start), Some(end)) if start == end => start.clone(),
            (Some(start), Some(end)) => format!("{start} – {end}"),
            (Some(start), None) => start.clone(),
            (None, Some(end)) => end.clone(),
            (None, None) => "No dates yet".to_string(),
        };

        format!(
            "<a href=\"/trip/{}\" class=\"hop-card-link\">\
              <div class=\"data-card hop-card\">\
              <div class=\"hop-card-route\">{}\
              </div>\
              <div class=\"hop-card-meta\">\
              <span class=\"hop-card-date\">{}</span>\
              <span class=\"hop-card-badge\">{} journeys</span>\
              </div>\
              </div>\
             </a>",
            self.id, self.name, date_range, self.hop_count,
        )
    }
}

#[derive(Deserialize, JsonSchema, Default)]
pub struct CreateTripRequest {
    pub name: String,
}

#[derive(Deserialize, JsonSchema, Default)]
pub struct UpdateTripRequest {
    pub name: String,
}

#[derive(Deserialize, JsonSchema, Default)]
pub struct AutoGroupRequest {
    pub gap_days: Option<i64>,
}

#[derive(Deserialize, JsonSchema, Default)]
pub struct AssignHopRequest {
    pub hop_id: i64,
}

#[derive(Debug, Default, Serialize, JsonSchema)]
pub struct AutoGroupResponse {
    pub created: u64,
}

impl MultiFormatResponse for AutoGroupResponse {
    const HTML_TITLE: &'static str = "Auto Group Result";
    const CSV_HEADERS: &'static [&'static str] = &["created"];

    fn csv_row(&self) -> Vec<String> {
        vec![self.created.to_string()]
    }
}

fn redirect_back_or(headers: &HeaderMap, fallback: &str) -> Response {
    if let Some(referer) = headers.get(header::REFERER).and_then(|v| v.to_str().ok()) {
        Redirect::to(referer).into_response()
    } else {
        Redirect::to(fallback).into_response()
    }
}

fn parse_payload<T>(headers: &HeaderMap, body: &[u8]) -> Result<T, AppError>
where
    T: DeserializeOwned + Default,
{
    if body.is_empty() {
        return Ok(T::default());
    }

    if is_form_request(headers) {
        serde_urlencoded::from_bytes(body).map_err(AppError::from)
    } else {
        serde_json::from_slice(body).map_err(AppError::from)
    }
}

pub async fn create_handler(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> Response {
    let format = negotiate_format(&headers);
    let req: CreateTripRequest = match parse_payload(&headers, &body) {
        Ok(r) => r,
        Err(err) => return err.into_format_response(format),
    };

    if req.name.trim().is_empty() {
        return AppError::MissingField("name is required").into_format_response(format);
    }

    let id = match (db::trips::Create {
        user_id: auth.user_id,
        name: req.name.trim(),
    })
    .execute(&state.db)
    .await
    {
        Ok(id) => id,
        Err(err) => return AppError::from(err).into_format_response(format),
    };

    if is_form_request(&headers) {
        redirect_back_or(&headers, "/trips")
    } else {
        let response = TripResponse {
            id,
            name: req.name.trim().to_string(),
            start_date: None,
            end_date: None,
            hop_count: 0,
        };
        TripResponse::single_format_response(&response, format, StatusCode::CREATED)
    }
}

pub fn create_handler_docs(op: TransformOperation) -> TransformOperation {
    op.description("Create a named trip for the authenticated user.")
        .input::<Json<CreateTripRequest>>()
        .with(|mut op| {
            if let Some(aide::openapi::ReferenceOr::Item(body)) = &mut op.inner_mut().request_body
                && let Some(json_media) = body.content.get("application/json").cloned()
            {
                body.content
                    .insert("application/x-www-form-urlencoded".to_string(), json_media);
            }
            op
        })
        .response::<201, Json<TripResponse>>()
        .response::<400, Json<ErrorResponse>>()
        .response::<401, Json<ErrorResponse>>()
        .response::<500, Json<ErrorResponse>>()
        .tag("trips")
}

pub async fn update_handler(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i64>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> Response {
    let format = negotiate_format(&headers);
    let req: UpdateTripRequest = match parse_payload(&headers, &body) {
        Ok(r) => r,
        Err(err) => return err.into_format_response(format),
    };

    if req.name.trim().is_empty() {
        return AppError::MissingField("name is required").into_format_response(format);
    }

    match (db::trips::Update {
        id,
        user_id: auth.user_id,
        name: req.name.trim(),
    })
    .execute(&state.db)
    .await
    {
        Ok(true) => {
            if is_form_request(&headers) {
                redirect_back_or(&headers, &format!("/trip/{id}"))
            } else {
                let response = StatusResponse {
                    status: "ok".to_string(),
                };
                StatusResponse::single_format_response(&response, format, StatusCode::OK)
            }
        }
        Ok(false) => {
            ErrorResponse::into_format_response("trip not found", format, StatusCode::NOT_FOUND)
        }
        Err(err) => AppError::from(err).into_format_response(format),
    }
}

pub fn update_handler_docs(op: TransformOperation) -> TransformOperation {
    op.description("Update a trip name.")
        .input::<Json<UpdateTripRequest>>()
        .with(|mut op| {
            if let Some(aide::openapi::ReferenceOr::Item(body)) = &mut op.inner_mut().request_body
                && let Some(json_media) = body.content.get("application/json").cloned()
            {
                body.content
                    .insert("application/x-www-form-urlencoded".to_string(), json_media);
            }
            op
        })
        .response::<200, Json<StatusResponse>>()
        .response::<400, Json<ErrorResponse>>()
        .response::<401, Json<ErrorResponse>>()
        .response::<404, Json<ErrorResponse>>()
        .response::<500, Json<ErrorResponse>>()
        .tag("trips")
}

pub async fn delete_handler(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i64>,
    headers: HeaderMap,
) -> Response {
    let format = negotiate_format(&headers);
    match (db::trips::Delete {
        id,
        user_id: auth.user_id,
    })
    .execute(&state.db)
    .await
    {
        Ok(true) => {
            if is_form_request(&headers) {
                redirect_back_or(&headers, "/trips")
            } else {
                let response = StatusResponse {
                    status: "ok".to_string(),
                };
                StatusResponse::single_format_response(&response, format, StatusCode::OK)
            }
        }
        Ok(false) => {
            ErrorResponse::into_format_response("trip not found", format, StatusCode::NOT_FOUND)
        }
        Err(err) => AppError::from(err).into_format_response(format),
    }
}

pub fn delete_handler_docs(op: TransformOperation) -> TransformOperation {
    multi_format_docs!(
        op.description("Delete a trip. Assigned journeys are unassigned via FK set null."),
        200 => StatusResponse,
        401 | 404 | 500 => ErrorResponse,
    )
    .tag("trips")
}

pub async fn auto_group_handler(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> Response {
    let format = negotiate_format(&headers);
    let req: AutoGroupRequest = match parse_payload(&headers, &body) {
        Ok(r) => r,
        Err(err) => return err.into_format_response(format),
    };

    let gap_days = req.gap_days.unwrap_or(3);
    let created = match (db::trips::AutoGroup {
        user_id: auth.user_id,
        gap_days,
    })
    .execute(&state.db)
    .await
    {
        Ok(created) => created,
        Err(err) => return AppError::from(err).into_format_response(format),
    };

    if is_form_request(&headers) {
        redirect_back_or(&headers, "/trips")
    } else {
        let response = AutoGroupResponse { created };
        AutoGroupResponse::single_format_response(&response, format, StatusCode::OK)
    }
}

pub fn auto_group_handler_docs(op: TransformOperation) -> TransformOperation {
    op.description("Automatically group unassigned journeys into trips by date proximity.")
        .input::<Json<AutoGroupRequest>>()
        .with(|mut op| {
            if let Some(aide::openapi::ReferenceOr::Item(body)) = &mut op.inner_mut().request_body
                && let Some(json_media) = body.content.get("application/json").cloned()
            {
                body.content
                    .insert("application/x-www-form-urlencoded".to_string(), json_media);
            }
            op
        })
        .response::<200, Json<AutoGroupResponse>>()
        .response::<400, Json<ErrorResponse>>()
        .response::<401, Json<ErrorResponse>>()
        .response::<500, Json<ErrorResponse>>()
        .tag("trips")
}

pub async fn assign_hop_handler(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i64>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> Response {
    let format = negotiate_format(&headers);
    let req: AssignHopRequest = match parse_payload(&headers, &body) {
        Ok(r) => r,
        Err(err) => return err.into_format_response(format),
    };

    if req.hop_id == 0 {
        return AppError::MissingField("hop_id is required").into_format_response(format);
    }

    match (db::trips::AssignHop {
        hop_id: req.hop_id,
        trip_id: id,
        user_id: auth.user_id,
    })
    .execute(&state.db)
    .await
    {
        Ok(true) => {
            if is_form_request(&headers) {
                redirect_back_or(&headers, &format!("/trip/{id}"))
            } else {
                let response = StatusResponse {
                    status: "ok".to_string(),
                };
                StatusResponse::single_format_response(&response, format, StatusCode::OK)
            }
        }
        Ok(false) => ErrorResponse::into_format_response(
            "trip or journey not found",
            format,
            StatusCode::NOT_FOUND,
        ),
        Err(err) => AppError::from(err).into_format_response(format),
    }
}

pub fn assign_hop_handler_docs(op: TransformOperation) -> TransformOperation {
    op.description("Assign a journey to a trip.")
        .input::<Json<AssignHopRequest>>()
        .with(|mut op| {
            if let Some(aide::openapi::ReferenceOr::Item(body)) = &mut op.inner_mut().request_body
                && let Some(json_media) = body.content.get("application/json").cloned()
            {
                body.content
                    .insert("application/x-www-form-urlencoded".to_string(), json_media);
            }
            op
        })
        .response::<200, Json<StatusResponse>>()
        .response::<400, Json<ErrorResponse>>()
        .response::<401, Json<ErrorResponse>>()
        .response::<404, Json<ErrorResponse>>()
        .response::<500, Json<ErrorResponse>>()
        .tag("trips")
}

pub async fn unassign_hop_handler(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((id, hop_id)): Path<(i64, i64)>,
    headers: HeaderMap,
) -> Response {
    let format = negotiate_format(&headers);

    let in_trip = match sqlx::query_scalar!(
        r#"SELECT EXISTS(
               SELECT 1 FROM hops
               WHERE id = ? AND user_id = ? AND user_trip_id = ?
           ) as "exists!: i64""#,
        hop_id,
        auth.user_id,
        id,
    )
    .fetch_one(&state.db)
    .await
    {
        Ok(exists) => exists == 1,
        Err(err) => return AppError::from(err).into_format_response(format),
    };

    if !in_trip {
        return ErrorResponse::into_format_response(
            "journey not found in trip",
            format,
            StatusCode::NOT_FOUND,
        );
    }

    match (db::trips::UnassignHop {
        hop_id,
        user_id: auth.user_id,
    })
    .execute(&state.db)
    .await
    {
        Ok(true) => {
            if is_form_request(&headers) {
                redirect_back_or(&headers, &format!("/trip/{id}"))
            } else {
                let response = StatusResponse {
                    status: "ok".to_string(),
                };
                StatusResponse::single_format_response(&response, format, StatusCode::OK)
            }
        }
        Ok(false) => {
            ErrorResponse::into_format_response("journey not found", format, StatusCode::NOT_FOUND)
        }
        Err(err) => AppError::from(err).into_format_response(format),
    }
}

pub fn unassign_hop_handler_docs(op: TransformOperation) -> TransformOperation {
    multi_format_docs!(
        op.description("Remove a journey from a trip."),
        200 => StatusResponse,
        401 | 404 | 500 => ErrorResponse,
    )
    .tag("trips")
}

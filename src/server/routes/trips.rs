use super::{
    ErrorResponse, MultiFormatResponse, StatusResponse, multi_format_docs, negotiate_format,
};
use crate::{
    db,
    server::{
        AppState,
        error::AppError,
        extractors::{AuthUser, FormOrJson},
        pages::FormFeedback,
        session::is_form_request,
    },
};
use aide::transform::TransformOperation;
use axum::{
    Json,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Redirect, Response},
};
use leptos::prelude::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, JsonSchema)]
pub struct TripResponse {
    pub id: i64,
    pub name: String,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    #[serde(rename = "journey_count")]
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
        &["id", "name", "start_date", "end_date", "journey_count"];

    fn csv_row(&self) -> Vec<String> {
        vec![
            self.id.to_string(),
            self.name.clone(),
            self.start_date.clone().unwrap_or_default(),
            self.end_date.clone().unwrap_or_default(),
            self.hop_count.to_string(),
        ]
    }

    fn html_card(&self) -> AnyView {
        let href = format!("/trips/{}", self.id);
        let name = self.name.clone();
        let date_range = match (&self.start_date, &self.end_date) {
            (Some(start), Some(end)) if start == end => start.clone(),
            (Some(start), Some(end)) => format!("{start} – {end}"),
            (Some(start), None) => start.clone(),
            (None, Some(end)) => end.clone(),
            (None, None) => "No dates yet".to_string(),
        };
        let badge_text = format!("{} journeys", self.hop_count);

        view! {
            <a href=href class="journey-card-link">
                <div class="data-card journey-card">
                    <div class="journey-card-route">{name}</div>
                    <div class="journey-card-meta">
                        <span class="journey-card-date">{date_range}</span>
                        <span class="journey-card-badge">{badge_text}</span>
                    </div>
                </div>
            </a>
        }
        .into_any()
    }
}

pub async fn handler(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(feedback): Query<FormFeedback>,
    headers: HeaderMap,
) -> Response {
    let format = negotiate_format(&headers);
    let trips = match (db::trips::GetAll {
        user_id: auth.user_id,
    })
    .execute(&state.db)
    .await
    {
        Ok(trips) => trips,
        Err(err) => {
            return AppError::from(err).into_format_response(format);
        }
    };

    if format == super::ResponseFormat::Html {
        crate::server::pages::trips::render_page(trips, feedback)
    } else {
        let responses: Vec<TripResponse> = trips.into_iter().map(TripResponse::from).collect();
        TripResponse::into_format_response(&responses, format, StatusCode::OK)
    }
}

pub fn handler_docs(op: TransformOperation) -> TransformOperation {
    multi_format_docs!(
        op.description("List trips for the authenticated user.")
            .response_with::<200, Json<Vec<TripResponse>>, _>(|mut res| {
                super::add_multi_format_docs::<TripResponse>(res.inner());
                res
            }),
        401 | 500 => ErrorResponse,
    )
    .tag("trips")
}

pub async fn get_trip_handler(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i64>,
    Query(feedback): Query<FormFeedback>,
    headers: HeaderMap,
) -> Response {
    let format = negotiate_format(&headers);

    let trip = match (db::trips::GetById {
        id,
        user_id: auth.user_id,
    })
    .execute(&state.db)
    .await
    {
        Ok(Some(trip)) => trip,
        Ok(None) => {
            return if format == super::ResponseFormat::Html {
                crate::server::pages::not_found::page().await
            } else {
                (
                    StatusCode::NOT_FOUND,
                    Json(ErrorResponse {
                        error: "trip not found".to_owned(),
                    }),
                )
                    .into_response()
            };
        }
        Err(err) => {
            return AppError::from(err).into_format_response(format);
        }
    };

    if format == super::ResponseFormat::Html {
        use crate::server::pages::trip_detail::{TripHopRow, UnassignedHopRow};

        let trip_hops = (db::hops::GetForTrip {
            user_id: auth.user_id,
            trip_id: id,
        })
        .execute(&state.db)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(TripHopRow::from)
        .collect();

        let unassigned_hops = (db::hops::GetUnassigned {
            user_id: auth.user_id,
        })
        .execute(&state.db)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(UnassignedHopRow::from)
        .collect();

        crate::server::pages::trip_detail::render_page(trip, trip_hops, unassigned_hops, feedback)
    } else {
        let response = TripResponse::from(trip);
        TripResponse::single_format_response(&response, format, StatusCode::OK)
    }
}

pub fn get_trip_handler_docs(op: TransformOperation) -> TransformOperation {
    multi_format_docs!(
        op.description("Get a single trip by ID.")
            .response_with::<200, Json<TripResponse>, _>(|mut res| {
                super::add_multi_format_docs::<TripResponse>(res.inner());
                res
            }),
        401 | 404 | 500 => ErrorResponse,
    )
    .tag("trips")
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
pub struct AssignJourneyRequest {
    #[serde(alias = "hop_id")]
    pub journey_id: i64,
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

pub async fn create_handler(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> Response {
    let format = negotiate_format(&headers);
    let req: CreateTripRequest = match FormOrJson::<CreateTripRequest>::parse(&headers, &body) {
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
    multi_format_docs!(
        op.description("Create a named trip for the authenticated user.")
            .input::<FormOrJson<CreateTripRequest>>(),
        201 => TripResponse,
        400 | 401 | 500 => ErrorResponse,
    )
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
    let req: UpdateTripRequest = match FormOrJson::<UpdateTripRequest>::parse(&headers, &body) {
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
                redirect_back_or(&headers, &format!("/trips/{id}"))
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
    multi_format_docs!(
        op.description("Update a trip name.")
            .input::<FormOrJson<UpdateTripRequest>>(),
        200 => StatusResponse,
        400 | 401 | 404 | 500 => ErrorResponse,
    )
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
    let req: AutoGroupRequest = match FormOrJson::<AutoGroupRequest>::parse(&headers, &body) {
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
    multi_format_docs!(
        op.description("Automatically group unassigned journeys into trips by date proximity.")
            .input::<FormOrJson<AutoGroupRequest>>(),
        200 => AutoGroupResponse,
        400 | 401 | 500 => ErrorResponse,
    )
    .tag("trips")
}

pub async fn assign_journey_handler(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i64>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> Response {
    let format = negotiate_format(&headers);
    let req: AssignJourneyRequest = match FormOrJson::<AssignJourneyRequest>::parse(&headers, &body)
    {
        Ok(r) => r,
        Err(err) => return err.into_format_response(format),
    };

    if req.journey_id == 0 {
        return AppError::MissingField("journey_id is required").into_format_response(format);
    }

    match (db::trips::AssignHop {
        hop_id: req.journey_id,
        trip_id: id,
        user_id: auth.user_id,
    })
    .execute(&state.db)
    .await
    {
        Ok(true) => {
            if is_form_request(&headers) {
                redirect_back_or(&headers, &format!("/trips/{id}"))
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

pub fn assign_journey_handler_docs(op: TransformOperation) -> TransformOperation {
    multi_format_docs!(
        op.description("Assign a journey to a trip.")
            .input::<FormOrJson<AssignJourneyRequest>>(),
        200 => StatusResponse,
        400 | 401 | 404 | 500 => ErrorResponse,
    )
    .tag("trips")
}

pub async fn unassign_journey_handler(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((id, journey_id)): Path<(i64, i64)>,
    headers: HeaderMap,
) -> Response {
    let format = negotiate_format(&headers);

    let in_trip = match (db::hops::ExistsInTrip {
        hop_id: journey_id,
        user_id: auth.user_id,
        trip_id: id,
    })
    .execute(&state.db)
    .await
    {
        Ok(exists) => exists,
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
        hop_id: journey_id,
        user_id: auth.user_id,
    })
    .execute(&state.db)
    .await
    {
        Ok(true) => {
            if is_form_request(&headers) {
                redirect_back_or(&headers, &format!("/trips/{id}"))
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

pub fn unassign_journey_handler_docs(op: TransformOperation) -> TransformOperation {
    multi_format_docs!(
        op.description("Remove a journey from a trip."),
        200 => StatusResponse,
        401 | 404 | 500 => ErrorResponse,
    )
    .tag("trips")
}

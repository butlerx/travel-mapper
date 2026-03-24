use super::{
    ErrorResponse, MultiFormatResponse, add_multi_format_docs, multi_format_docs, negotiate_format,
};
use crate::{
    db,
    server::{AppState, error::AppError, extractors::AuthUser, session::is_form_request},
};
use aide::transform::TransformOperation;
use axum::{
    Json,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Redirect, Response},
};
use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// API response type for a single travel hop.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct HopResponse {
    /// Database identifier for this hop.
    pub id: i64,
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
    /// Boat (cruise or ferry) segment.
    Boat,
    /// Ground transport (car, bus, taxi, etc.).
    Transport,
}

impl HopTravelType {
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Air => "air",
            Self::Rail => "rail",
            Self::Boat => "boat",
            Self::Transport => "transport",
        }
    }

    #[must_use]
    pub const fn emoji(&self) -> &'static str {
        match self {
            Self::Air => "✈️",
            Self::Rail => "🚆",
            Self::Boat => "🚢",
            Self::Transport => "🚗",
        }
    }
}

impl std::fmt::Display for HopTravelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Air => write!(f, "air"),
            Self::Rail => write!(f, "rail"),
            Self::Boat => write!(f, "boat"),
            Self::Transport => write!(f, "transport"),
        }
    }
}

impl From<db::hops::TravelType> for HopTravelType {
    fn from(t: db::hops::TravelType) -> Self {
        match t {
            db::hops::TravelType::Air => Self::Air,
            db::hops::TravelType::Rail => Self::Rail,
            db::hops::TravelType::Boat => Self::Boat,
            db::hops::TravelType::Transport => Self::Transport,
        }
    }
}

impl From<HopTravelType> for db::hops::TravelType {
    fn from(t: HopTravelType) -> Self {
        match t {
            HopTravelType::Air => Self::Air,
            HopTravelType::Rail => Self::Rail,
            HopTravelType::Boat => Self::Boat,
            HopTravelType::Transport => Self::Transport,
        }
    }
}

impl From<db::hops::Row> for HopResponse {
    fn from(hop: db::hops::Row) -> Self {
        Self {
            id: hop.id,
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
    const HTML_TITLE: &'static str = "Travel Journeys";

    const CSV_HEADERS: &'static [&'static str] = &[
        "id",
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
            self.id.to_string(),
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
        let id = self.id;
        let emoji = self.travel_type.emoji();
        let travel_type = self.travel_type.as_str();
        let origin = &self.origin_name;
        let dest = &self.dest_name;
        let date = &self.start_date;

        format!(
            "<a href=\"/journey/{id}\" class=\"hop-card-link\">\
             <div class=\"data-card hop-card\">\
             <div class=\"hop-card-route\">\
             <span class=\"hop-card-place\">{origin}</span>\
             <span class=\"hop-card-arrow\">→</span>\
             <span class=\"hop-card-place\">{dest}</span>\
             </div>\
             <div class=\"hop-card-meta\">\
             <span class=\"hop-card-badge badge-{travel_type}\">{emoji} {travel_type}</span>\
             <span class=\"hop-card-date\">{date}</span>\
             </div>\
             </div>\
             </a>"
        )
    }
}

#[derive(Deserialize, JsonSchema)]
pub struct HopQuery {
    #[serde(rename = "type")]
    travel_type: Option<HopTravelType>,
    origin: Option<String>,
    dest: Option<String>,
    date_from: Option<String>,
    date_to: Option<String>,
    airline: Option<String>,
    flight_number: Option<String>,
    cabin_class: Option<String>,
    flight_reason: Option<String>,
    q: Option<String>,
}

pub async fn handler(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(query): Query<HopQuery>,
    headers: HeaderMap,
) -> Response {
    let format = negotiate_format(&headers);
    let hops = match (db::hops::Search {
        user_id: auth.user_id,
        travel_type: query.travel_type.as_ref().map(HopTravelType::as_str),
        origin: query.origin.as_deref(),
        dest: query.dest.as_deref(),
        date_from: query.date_from.as_deref(),
        date_to: query.date_to.as_deref(),
        airline: query.airline.as_deref(),
        flight_number: query.flight_number.as_deref(),
        cabin_class: query.cabin_class.as_deref(),
        flight_reason: query.flight_reason.as_deref(),
        q: query.q.as_deref(),
    })
    .execute(&state.db)
    .await
    {
        Ok(hops) => hops,
        Err(err) => {
            return AppError::from(err).into_format_response(format);
        }
    };

    let responses: Vec<HopResponse> = hops.into_iter().map(HopResponse::from).collect();
    HopResponse::into_format_response(&responses, format, StatusCode::OK)
}

pub fn handler_docs(op: TransformOperation) -> TransformOperation {
    multi_format_docs!(
        op.description("List travel journeys for the authenticated user.")
            .response_with::<200, Json<Vec<HopResponse>>, _>(|mut res| {
                add_multi_format_docs::<HopResponse>(res.inner());
                res
            }),
        401 | 500 => ErrorResponse,
    )
    .tag("journeys")
}

const QUERY_ENCODE_SET: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'_')
    .remove(b'.')
    .remove(b'~');

fn encode_query_value(s: &str) -> String {
    utf8_percent_encode(s, QUERY_ENCODE_SET).to_string()
}

/// Request body for manually creating a hop of any travel type.
#[derive(Deserialize, JsonSchema)]
pub struct CreateHopRequest {
    /// Mode of transport (defaults to `air` when omitted).
    #[serde(default)]
    pub travel_type: HopTravelType,
    /// Origin location — IATA code for flights, station/city name otherwise.
    pub origin: String,
    /// Destination location — IATA code for flights, station/city name otherwise.
    pub destination: String,
    /// Departure date in `YYYY-MM-DD` format.
    pub date: String,
    #[serde(default)]
    pub airline: Option<String>,
    #[serde(default)]
    pub flight_number: Option<String>,
    #[serde(default)]
    pub aircraft_type: Option<String>,
    #[serde(default)]
    pub cabin_class: Option<String>,
    #[serde(default)]
    pub seat: Option<String>,
    #[serde(default)]
    pub pnr: Option<String>,
    #[serde(default)]
    pub rail_carrier: Option<String>,
    #[serde(default)]
    pub train_number: Option<String>,
    #[serde(default)]
    pub service_class: Option<String>,
    #[serde(default)]
    pub coach_number: Option<String>,
    #[serde(default)]
    pub rail_seats: Option<String>,
    #[serde(default)]
    pub rail_confirmation: Option<String>,
    #[serde(default)]
    pub rail_booking_site: Option<String>,
    #[serde(default)]
    pub rail_notes: Option<String>,
    #[serde(default)]
    pub ship_name: Option<String>,
    #[serde(default)]
    pub cabin_type: Option<String>,
    #[serde(default)]
    pub cabin_number: Option<String>,
    #[serde(default)]
    pub boat_confirmation: Option<String>,
    #[serde(default)]
    pub boat_booking_site: Option<String>,
    #[serde(default)]
    pub boat_notes: Option<String>,
    #[serde(default)]
    pub transport_carrier: Option<String>,
    #[serde(default)]
    pub vehicle_description: Option<String>,
    #[serde(default)]
    pub transport_confirmation: Option<String>,
    #[serde(default)]
    pub transport_notes: Option<String>,
}

impl CreateHopRequest {
    fn build_manual_detail(&self) -> db::hops::ManualDetail {
        match self.travel_type {
            HopTravelType::Air => db::hops::ManualDetail::Air(db::hops::FlightDetail {
                airline: self.airline.clone().unwrap_or_default(),
                flight_number: self.flight_number.clone().unwrap_or_default(),
                aircraft_type: self.aircraft_type.clone().unwrap_or_default(),
                cabin_class: self.cabin_class.clone().unwrap_or_default(),
                seat: self.seat.clone().unwrap_or_default(),
                pnr: self.pnr.clone().unwrap_or_default(),
            }),
            HopTravelType::Rail => db::hops::ManualDetail::Rail(db::hops::RailDetail {
                carrier: self.rail_carrier.clone().unwrap_or_default(),
                train_number: self.train_number.clone().unwrap_or_default(),
                service_class: self.service_class.clone().unwrap_or_default(),
                coach_number: self.coach_number.clone().unwrap_or_default(),
                seats: self.rail_seats.clone().unwrap_or_default(),
                confirmation_num: self.rail_confirmation.clone().unwrap_or_default(),
                booking_site: self.rail_booking_site.clone().unwrap_or_default(),
                notes: self.rail_notes.clone().unwrap_or_default(),
            }),
            HopTravelType::Boat => db::hops::ManualDetail::Boat(db::hops::BoatDetail {
                ship_name: self.ship_name.clone().unwrap_or_default(),
                cabin_type: self.cabin_type.clone().unwrap_or_default(),
                cabin_number: self.cabin_number.clone().unwrap_or_default(),
                confirmation_num: self.boat_confirmation.clone().unwrap_or_default(),
                booking_site: self.boat_booking_site.clone().unwrap_or_default(),
                notes: self.boat_notes.clone().unwrap_or_default(),
            }),
            HopTravelType::Transport => {
                db::hops::ManualDetail::Transport(db::hops::TransportDetail {
                    carrier_name: self.transport_carrier.clone().unwrap_or_default(),
                    vehicle_description: self.vehicle_description.clone().unwrap_or_default(),
                    confirmation_num: self.transport_confirmation.clone().unwrap_or_default(),
                    notes: self.transport_notes.clone().unwrap_or_default(),
                })
            }
        }
    }
}

/// Successful response after creating a hop.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreateHopResponse {
    /// Number of hops created.
    pub created: u64,
}

/// Create a hop manually for any travel type.
///
/// Accepts JSON or form-encoded body. Form submissions redirect to the add
/// hop page with a success or error query parameter.
pub async fn create_handler(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> Response {
    let is_form = is_form_request(&headers);

    let parsed: Result<CreateHopRequest, AppError> = if is_form {
        serde_urlencoded::from_bytes(&body).map_err(AppError::from)
    } else {
        serde_json::from_slice(&body).map_err(AppError::from)
    };

    let req = match parsed {
        Ok(r) => r,
        Err(err) => {
            return if is_form {
                Redirect::to(&format!(
                    "/journeys/new?error={}",
                    encode_query_value(&format!("Invalid form data: {err}"))
                ))
                .into_response()
            } else {
                let format = negotiate_format(&headers);
                err.into_format_response(format)
            };
        }
    };

    if req.origin.is_empty() || req.destination.is_empty() || req.date.is_empty() {
        let err = AppError::MissingField("origin, destination, and date are required");
        return if is_form {
            Redirect::to(&format!(
                "/journeys/new?error={}",
                encode_query_value(&err.to_string())
            ))
            .into_response()
        } else {
            let format = negotiate_format(&headers);
            err.into_format_response(format)
        };
    }

    let detail = req.build_manual_detail();

    let result = (db::hops::CreateManual {
        user_id: auth.user_id,
        origin: req.origin,
        destination: req.destination,
        date: req.date,
        detail,
    })
    .execute(&state.db)
    .await;

    match result {
        Ok(created) => {
            if is_form {
                Redirect::to("/journeys/new?success=1").into_response()
            } else {
                (StatusCode::CREATED, Json(CreateHopResponse { created })).into_response()
            }
        }
        Err(err) => {
            let err = AppError::from(err);
            if is_form {
                Redirect::to(&format!(
                    "/journeys/new?error={}",
                    encode_query_value(&err.to_string())
                ))
                .into_response()
            } else {
                let format = negotiate_format(&headers);
                err.into_format_response(format)
            }
        }
    }
}

pub fn create_handler_docs(op: TransformOperation) -> TransformOperation {
    op.description(
        "Create a journey manually for any travel type. Accepts JSON or form-encoded body.",
    )
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
    .tag("journeys")
}

#[derive(Deserialize, JsonSchema)]
pub struct UpdateHopRequest {
    pub travel_type: HopTravelType,
    pub origin_name: String,
    pub dest_name: String,
    pub start_date: String,
    pub end_date: String,
    pub origin_lat: f64,
    pub origin_lng: f64,
    #[serde(default)]
    pub origin_country: Option<String>,
    pub dest_lat: f64,
    pub dest_lng: f64,
    #[serde(default)]
    pub dest_country: Option<String>,
    #[serde(default)]
    pub airline: Option<String>,
    #[serde(default)]
    pub flight_number: Option<String>,
    #[serde(default)]
    pub dep_terminal: Option<String>,
    #[serde(default)]
    pub dep_gate: Option<String>,
    #[serde(default)]
    pub arr_terminal: Option<String>,
    #[serde(default)]
    pub arr_gate: Option<String>,
    #[serde(default)]
    pub canceled: Option<bool>,
    #[serde(default)]
    pub diverted_to: Option<String>,
    #[serde(default)]
    pub gate_dep_scheduled: Option<String>,
    #[serde(default)]
    pub gate_dep_actual: Option<String>,
    #[serde(default)]
    pub takeoff_scheduled: Option<String>,
    #[serde(default)]
    pub takeoff_actual: Option<String>,
    #[serde(default)]
    pub landing_scheduled: Option<String>,
    #[serde(default)]
    pub landing_actual: Option<String>,
    #[serde(default)]
    pub gate_arr_scheduled: Option<String>,
    #[serde(default)]
    pub gate_arr_actual: Option<String>,
    #[serde(default)]
    pub aircraft_type: Option<String>,
    #[serde(default)]
    pub tail_number: Option<String>,
    #[serde(default)]
    pub pnr: Option<String>,
    #[serde(default)]
    pub seat: Option<String>,
    #[serde(default)]
    pub seat_type: Option<String>,
    #[serde(default)]
    pub cabin_class: Option<String>,
    #[serde(default)]
    pub flight_reason: Option<String>,
    #[serde(default)]
    pub flight_notes: Option<String>,
    #[serde(default)]
    pub rail_carrier: Option<String>,
    #[serde(default)]
    pub train_number: Option<String>,
    #[serde(default)]
    pub service_class: Option<String>,
    #[serde(default)]
    pub coach_number: Option<String>,
    #[serde(default)]
    pub rail_seats: Option<String>,
    #[serde(default)]
    pub rail_confirmation: Option<String>,
    #[serde(default)]
    pub rail_booking_site: Option<String>,
    #[serde(default)]
    pub rail_notes: Option<String>,
    #[serde(default)]
    pub ship_name: Option<String>,
    #[serde(default)]
    pub cabin_type: Option<String>,
    #[serde(default)]
    pub cabin_number: Option<String>,
    #[serde(default)]
    pub boat_confirmation: Option<String>,
    #[serde(default)]
    pub boat_booking_site: Option<String>,
    #[serde(default)]
    pub boat_notes: Option<String>,
    #[serde(default)]
    pub transport_carrier: Option<String>,
    #[serde(default)]
    pub vehicle_description: Option<String>,
    #[serde(default)]
    pub transport_confirmation: Option<String>,
    #[serde(default)]
    pub transport_notes: Option<String>,
}

impl UpdateHopRequest {
    fn build_flight_detail(&self) -> Option<db::hops::FullFlightDetail> {
        (self.travel_type == HopTravelType::Air).then(|| db::hops::FullFlightDetail {
            airline: self.airline.clone().unwrap_or_default(),
            flight_number: self.flight_number.clone().unwrap_or_default(),
            dep_terminal: self.dep_terminal.clone().unwrap_or_default(),
            dep_gate: self.dep_gate.clone().unwrap_or_default(),
            arr_terminal: self.arr_terminal.clone().unwrap_or_default(),
            arr_gate: self.arr_gate.clone().unwrap_or_default(),
            canceled: self.canceled.unwrap_or(false),
            diverted_to: self.diverted_to.clone().unwrap_or_default(),
            gate_dep_scheduled: self.gate_dep_scheduled.clone().unwrap_or_default(),
            gate_dep_actual: self.gate_dep_actual.clone().unwrap_or_default(),
            takeoff_scheduled: self.takeoff_scheduled.clone().unwrap_or_default(),
            takeoff_actual: self.takeoff_actual.clone().unwrap_or_default(),
            landing_scheduled: self.landing_scheduled.clone().unwrap_or_default(),
            landing_actual: self.landing_actual.clone().unwrap_or_default(),
            gate_arr_scheduled: self.gate_arr_scheduled.clone().unwrap_or_default(),
            gate_arr_actual: self.gate_arr_actual.clone().unwrap_or_default(),
            aircraft_type: self.aircraft_type.clone().unwrap_or_default(),
            tail_number: self.tail_number.clone().unwrap_or_default(),
            pnr: self.pnr.clone().unwrap_or_default(),
            seat: self.seat.clone().unwrap_or_default(),
            seat_type: self.seat_type.clone().unwrap_or_default(),
            cabin_class: self.cabin_class.clone().unwrap_or_default(),
            flight_reason: self.flight_reason.clone().unwrap_or_default(),
            notes: self.flight_notes.clone().unwrap_or_default(),
        })
    }

    fn build_rail_detail(&self) -> Option<db::hops::RailDetail> {
        (self.travel_type == HopTravelType::Rail).then(|| db::hops::RailDetail {
            carrier: self.rail_carrier.clone().unwrap_or_default(),
            train_number: self.train_number.clone().unwrap_or_default(),
            service_class: self.service_class.clone().unwrap_or_default(),
            coach_number: self.coach_number.clone().unwrap_or_default(),
            seats: self.rail_seats.clone().unwrap_or_default(),
            confirmation_num: self.rail_confirmation.clone().unwrap_or_default(),
            booking_site: self.rail_booking_site.clone().unwrap_or_default(),
            notes: self.rail_notes.clone().unwrap_or_default(),
        })
    }

    fn build_boat_detail(&self) -> Option<db::hops::BoatDetail> {
        (self.travel_type == HopTravelType::Boat).then(|| db::hops::BoatDetail {
            ship_name: self.ship_name.clone().unwrap_or_default(),
            cabin_type: self.cabin_type.clone().unwrap_or_default(),
            cabin_number: self.cabin_number.clone().unwrap_or_default(),
            confirmation_num: self.boat_confirmation.clone().unwrap_or_default(),
            booking_site: self.boat_booking_site.clone().unwrap_or_default(),
            notes: self.boat_notes.clone().unwrap_or_default(),
        })
    }

    fn build_transport_detail(&self) -> Option<db::hops::TransportDetail> {
        (self.travel_type == HopTravelType::Transport).then(|| db::hops::TransportDetail {
            carrier_name: self.transport_carrier.clone().unwrap_or_default(),
            vehicle_description: self.vehicle_description.clone().unwrap_or_default(),
            confirmation_num: self.transport_confirmation.clone().unwrap_or_default(),
            notes: self.transport_notes.clone().unwrap_or_default(),
        })
    }
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct UpdateHopResponse {
    pub updated: bool,
}

pub async fn update_handler(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i64>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> Response {
    let is_form = is_form_request(&headers);

    let parsed: Result<UpdateHopRequest, AppError> = if is_form {
        serde_urlencoded::from_bytes(&body).map_err(AppError::from)
    } else {
        serde_json::from_slice(&body).map_err(AppError::from)
    };

    let req = match parsed {
        Ok(r) => r,
        Err(err) => {
            return if is_form {
                Redirect::to(&format!(
                    "/journey/{id}?error={}",
                    encode_query_value(&format!("Invalid form data: {err}"))
                ))
                .into_response()
            } else {
                let format = negotiate_format(&headers);
                err.into_format_response(format)
            };
        }
    };

    if req.origin_name.is_empty() || req.dest_name.is_empty() || req.start_date.is_empty() {
        let err = AppError::MissingField("origin_name, dest_name, and start_date are required");
        return if is_form {
            Redirect::to(&format!(
                "/journey/{id}?error={}",
                encode_query_value(&err.to_string())
            ))
            .into_response()
        } else {
            let format = negotiate_format(&headers);
            err.into_format_response(format)
        };
    }

    let flight_detail = req.build_flight_detail();
    let rail_detail = req.build_rail_detail();
    let boat_detail = req.build_boat_detail();
    let transport_detail = req.build_transport_detail();

    let result = (db::hops::UpdateById {
        id,
        user_id: auth.user_id,
        origin_name: req.origin_name,
        dest_name: req.dest_name,
        start_date: req.start_date,
        end_date: req.end_date,
        origin_lat: req.origin_lat,
        origin_lng: req.origin_lng,
        origin_country: req.origin_country,
        dest_lat: req.dest_lat,
        dest_lng: req.dest_lng,
        dest_country: req.dest_country,
        travel_type: req.travel_type.into(),
        flight_detail,
        rail_detail,
        boat_detail,
        transport_detail,
    })
    .execute(&state.db)
    .await;

    match result {
        Ok(true) => {
            if is_form {
                Redirect::to(&format!("/journey/{id}?success=1")).into_response()
            } else {
                (StatusCode::OK, Json(UpdateHopResponse { updated: true })).into_response()
            }
        }
        Ok(false) => {
            if is_form {
                Redirect::to(&format!(
                    "/journey/{id}?error={}",
                    encode_query_value("Journey not found")
                ))
                .into_response()
            } else {
                (
                    StatusCode::NOT_FOUND,
                    Json(ErrorResponse {
                        error: "journey not found".to_owned(),
                    }),
                )
                    .into_response()
            }
        }
        Err(err) => {
            let err = AppError::from(err);
            if is_form {
                Redirect::to(&format!(
                    "/journey/{id}?error={}",
                    encode_query_value(&err.to_string())
                ))
                .into_response()
            } else {
                let format = negotiate_format(&headers);
                err.into_format_response(format)
            }
        }
    }
}

pub fn update_handler_docs(op: TransformOperation) -> TransformOperation {
    op.description("Update an existing journey by ID. Accepts JSON or form-encoded body.")
        .input::<Json<UpdateHopRequest>>()
        .with(|mut op| {
            if let Some(aide::openapi::ReferenceOr::Item(body)) = &mut op.inner_mut().request_body
                && let Some(json_media) = body.content.get("application/json").cloned()
            {
                body.content
                    .insert("application/x-www-form-urlencoded".to_string(), json_media);
            }
            op
        })
        .response::<200, Json<UpdateHopResponse>>()
        .response::<400, Json<ErrorResponse>>()
        .response::<401, Json<ErrorResponse>>()
        .response::<404, Json<ErrorResponse>>()
        .response::<500, Json<ErrorResponse>>()
        .tag("journeys")
}

#[cfg(test)]
mod tests {
    use super::{CreateHopResponse, HopResponse, HopTravelType};
    use crate::{
        db::{self, hops::TravelType},
        server::create_router,
        server::test_helpers::*,
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
                    .uri("/journeys")
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
                    .uri("/journeys")
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
                    .uri("/journeys")
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
                    .uri("/journeys")
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
                    .uri("/journeys?type=rail")
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
                    .uri("/journeys")
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
                    .uri("/journeys")
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
                    .uri("/journeys")
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
        assert!(body.contains("Travel Journeys"));
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
                    .uri("/journeys")
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
                    .uri("/journeys")
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
                    .uri("/journeys")
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
        assert_eq!(location, "/journeys/new?success=1");
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
                    .uri("/journeys")
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
    async fn get_hops_new_page_returns_add_hop_form() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/journeys/new")
                    .header(header::COOKIE, cookie)
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_text(response).await;
        assert!(
            body.contains("Add Journey"),
            "page should contain 'Add Journey'"
        );
    }

    #[tokio::test]
    async fn post_hops_json_creates_rail_hop() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let app = create_router(test_app_state(pool.clone()));

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/journeys")
                    .header(header::COOKIE, &cookie)
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(
                        r#"{"travel_type":"rail","origin":"Paris Gare du Nord","destination":"London St Pancras","date":"2025-07-01","rail_carrier":"Eurostar","train_number":"9024"}"#,
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
            travel_type_filter: Some("rail"),
        }
        .execute(&pool)
        .await
        .expect("fetch failed");
        assert_eq!(hops.len(), 1);
        assert_eq!(hops[0].origin_name, "Paris Gare du Nord");
        assert_eq!(hops[0].dest_name, "London St Pancras");
    }
}

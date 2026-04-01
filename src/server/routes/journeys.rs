use super::{
    ErrorResponse, MultiFormatResponse, ResponseFormat, add_multi_format_docs, multi_format_docs,
    negotiate_format,
};
use crate::{
    db,
    distance::haversine_miles,
    server::{
        AppState,
        components::{CarrierIcon, NavBar, Shell},
        error::AppError,
        extractors::{AuthUser, FormOrJson},
        session::is_form_request,
    },
};
use aide::transform::TransformOperation;
use axum::{
    Json,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Redirect, Response},
};
use leptos::prelude::*;
use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// API response type for a single travel journey.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct JourneyResponse {
    /// Database identifier for this journey.
    pub id: i64,
    /// Mode of transport for this journey.
    pub travel_type: JourneyTravelType,
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
    /// Carrier name or IATA code (e.g. `"BA"`, `"Irish Rail"`).
    pub carrier: Option<String>,
    /// Live flight status (e.g. `"scheduled"`, `"active"`, `"landed"`, `"cancelled"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// Delay in minutes (positive = late, negative = early).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delay_minutes: Option<i64>,
    /// Departure gate from status enrichment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dep_gate: Option<String>,
    /// Departure terminal from status enrichment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dep_terminal: Option<String>,
    /// Arrival gate from status enrichment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arr_gate: Option<String>,
    /// Arrival terminal from status enrichment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arr_terminal: Option<String>,
    /// Departure platform from rail status enrichment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dep_platform: Option<String>,
    /// Arrival platform from rail status enrichment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arr_platform: Option<String>,
    /// Whether the flight route was verified via ADS-B data from `OpenSky` Network.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub route_verified: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost_amount: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost_currency: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub loyalty_program: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub miles_earned: Option<f64>,
    /// When the most recent enrichment data was fetched (UTC datetime string).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fetched_at: Option<String>,
    /// Whether the enrichment data is still within its freshness TTL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_fresh: Option<bool>,
    /// Name of the enrichment data provider (e.g. `"airlabs"`, `"darwin"`, `"transitland"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
}

/// Mode of transport for a travel journey.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum JourneyTravelType {
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

impl JourneyTravelType {
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

impl std::fmt::Display for JourneyTravelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Air => write!(f, "air"),
            Self::Rail => write!(f, "rail"),
            Self::Boat => write!(f, "boat"),
            Self::Transport => write!(f, "transport"),
        }
    }
}

impl From<db::hops::TravelType> for JourneyTravelType {
    fn from(t: db::hops::TravelType) -> Self {
        match t {
            db::hops::TravelType::Air => Self::Air,
            db::hops::TravelType::Rail => Self::Rail,
            db::hops::TravelType::Boat => Self::Boat,
            db::hops::TravelType::Transport => Self::Transport,
        }
    }
}

impl From<JourneyTravelType> for db::hops::TravelType {
    fn from(t: JourneyTravelType) -> Self {
        match t {
            JourneyTravelType::Air => Self::Air,
            JourneyTravelType::Rail => Self::Rail,
            JourneyTravelType::Boat => Self::Boat,
            JourneyTravelType::Transport => Self::Transport,
        }
    }
}

fn non_empty(s: &str) -> Option<String> {
    if s.is_empty() {
        None
    } else {
        Some(s.to_owned())
    }
}

impl JourneyResponse {
    fn apply_enrichment(&mut self, enrichment: &db::status_enrichments::Row) {
        self.status = non_empty(&enrichment.status);
        self.delay_minutes = enrichment.delay_minutes;
        self.dep_gate = non_empty(&enrichment.dep_gate);
        self.dep_terminal = non_empty(&enrichment.dep_terminal);
        self.arr_gate = non_empty(&enrichment.arr_gate);
        self.arr_terminal = non_empty(&enrichment.arr_terminal);
        self.dep_platform = non_empty(&enrichment.dep_platform);
        self.arr_platform = non_empty(&enrichment.arr_platform);
        self.fetched_at = Some(enrichment.fetched_at.clone());
        self.is_fresh = Some(Self::compute_freshness(
            &enrichment.fetched_at,
            &self.start_date,
        ));
        self.provider = non_empty(&enrichment.provider);
    }

    fn apply_opensky_verification(&mut self, enrichment: &db::status_enrichments::Row) {
        self.route_verified = Some(enrichment.status == "verified");
    }

    fn compute_freshness(fetched_at: &str, start_date: &str) -> bool {
        let Ok(fetched) = chrono::NaiveDateTime::parse_from_str(fetched_at, "%Y-%m-%d %H:%M:%S")
        else {
            return false;
        };
        let ttl = crate::worker::departure_aware_ttl(start_date);
        let age = chrono::Utc::now().naive_utc() - fetched;
        age.num_seconds() < ttl
    }
}

impl From<db::hops::Row> for JourneyResponse {
    fn from(hop: db::hops::Row) -> Self {
        let carrier = hop.carrier().map(str::to_owned);
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
            carrier,
            status: None,
            delay_minutes: None,
            dep_gate: None,
            dep_terminal: None,
            arr_gate: None,
            arr_terminal: None,
            dep_platform: None,
            arr_platform: None,
            route_verified: None,
            cost_amount: hop.cost_amount,
            cost_currency: hop.cost_currency,
            loyalty_program: hop.loyalty_program,
            miles_earned: hop.miles_earned,
            fetched_at: None,
            is_fresh: None,
            provider: None,
        }
    }
}

impl From<db::hops::DetailRow> for JourneyResponse {
    fn from(hop: db::hops::DetailRow) -> Self {
        let carrier = hop
            .flight_detail
            .as_ref()
            .map(|d| &d.airline)
            .or_else(|| hop.rail_detail.as_ref().map(|d| &d.carrier))
            .or_else(|| hop.boat_detail.as_ref().map(|d| &d.ship_name))
            .or_else(|| hop.transport_detail.as_ref().map(|d| &d.carrier_name))
            .filter(|s| !s.is_empty())
            .cloned();
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
            carrier,
            status: None,
            delay_minutes: None,
            dep_gate: None,
            dep_terminal: None,
            arr_gate: None,
            arr_terminal: None,
            dep_platform: None,
            arr_platform: None,
            route_verified: None,
            cost_amount: hop.cost_amount,
            cost_currency: hop.cost_currency,
            loyalty_program: hop.loyalty_program,
            miles_earned: hop.miles_earned,
            fetched_at: None,
            is_fresh: None,
            provider: None,
        }
    }
}

impl MultiFormatResponse for JourneyResponse {
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
        "carrier",
        "status",
        "delay_minutes",
        "dep_gate",
        "dep_terminal",
        "arr_gate",
        "arr_terminal",
        "dep_platform",
        "arr_platform",
        "route_verified",
        "cost_amount",
        "cost_currency",
        "fetched_at",
        "is_fresh",
        "provider",
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
            self.carrier.clone().unwrap_or_default(),
            self.status.clone().unwrap_or_default(),
            self.delay_minutes
                .map_or_else(String::new, |d| d.to_string()),
            self.dep_gate.clone().unwrap_or_default(),
            self.dep_terminal.clone().unwrap_or_default(),
            self.arr_gate.clone().unwrap_or_default(),
            self.arr_terminal.clone().unwrap_or_default(),
            self.dep_platform.clone().unwrap_or_default(),
            self.arr_platform.clone().unwrap_or_default(),
            self.route_verified
                .map_or_else(String::new, |v| v.to_string()),
            self.cost_amount.map_or_else(String::new, |v| v.to_string()),
            self.cost_currency.clone().unwrap_or_default(),
            self.fetched_at.clone().unwrap_or_default(),
            self.is_fresh.map_or_else(String::new, |v| v.to_string()),
            self.provider.clone().unwrap_or_default(),
        ]
    }

    fn html_card(&self) -> AnyView {
        let href = format!("/journeys/{}", self.id);
        let emoji = self.travel_type.emoji();
        let travel_type = self.travel_type.as_str();
        let badge_class = format!("journey-card-badge badge-{travel_type}");
        let badge_text = format!("{emoji} {travel_type}");
        let origin = self.origin_name.clone();
        let dest = self.dest_name.clone();
        let date = self.start_date.clone();
        let carrier = self.carrier.clone().unwrap_or_default();
        let travel_type_str = travel_type.to_owned();

        let status_badge = self.status.as_ref().map(|s| {
            let css_class = format!("status-badge status-{}", s.to_lowercase().replace(' ', "-"));
            let label = match self.delay_minutes {
                Some(mins) if mins > 0 => format!("{s} (+{mins}m)"),
                Some(mins) if mins < 0 => format!("{s} ({mins}m)"),
                _ => s.clone(),
            };
            (css_class, label)
        });
        let verified_badge = self.route_verified.map(|verified| {
            if verified {
                ("status-badge status-connected", "✓ Verified")
            } else {
                ("status-badge status-disconnected", "Unverified")
            }
        });

        view! {
            <a href=href class="journey-card-link">
                <div class="data-card journey-card">
                    <div class="journey-card-route">
                        <CarrierIcon carrier=carrier travel_type=travel_type_str size=20 />
                        <span class="journey-card-place">{origin}</span>
                        <span class="journey-card-arrow">"→"</span>
                        <span class="journey-card-place">{dest}</span>
                    </div>
                    <div class="journey-card-meta">
                        <span class=badge_class>{badge_text}</span>
                        {status_badge.map(|(css_class, label)| view! {
                            <span class=css_class>{label}</span>
                        })}
                        {verified_badge.map(|(css_class, label)| view! {
                            <span class=css_class>{label}</span>
                        })}
                        <span class="journey-card-date">{date}</span>
                    </div>
                </div>
            </a>
        }
        .into_any()
    }
}

/// Available sort options for journey lists.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum JourneySort {
    /// Newest journeys first (default).
    #[default]
    DateDesc,
    /// Oldest journeys first.
    DateAsc,
    /// Origin airport/station A → Z.
    OriginAsc,
    /// Origin airport/station Z → A.
    OriginDesc,
    /// Destination airport/station A → Z.
    DestAsc,
    /// Destination airport/station Z → A.
    DestDesc,
}

impl JourneySort {
    /// Label shown in the sort control dropdown.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::DateDesc => "Date (newest)",
            Self::DateAsc => "Date (oldest)",
            Self::OriginAsc => "Origin (A→Z)",
            Self::OriginDesc => "Origin (Z→A)",
            Self::DestAsc => "Dest (A→Z)",
            Self::DestDesc => "Dest (Z→A)",
        }
    }

    /// Query string value matching serde rename.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::DateDesc => "date_desc",
            Self::DateAsc => "date_asc",
            Self::OriginAsc => "origin_asc",
            Self::OriginDesc => "origin_desc",
            Self::DestAsc => "dest_asc",
            Self::DestDesc => "dest_desc",
        }
    }

    /// Derive the section heading key for a journey under this sort.
    ///
    /// Date sorts group by year; origin/dest sorts group by first letter.
    #[must_use]
    pub fn group_key(self, resp: &JourneyResponse) -> String {
        match self {
            Self::DateDesc | Self::DateAsc => {
                resp.start_date.get(..4).unwrap_or("Unknown").to_owned()
            }
            Self::OriginAsc | Self::OriginDesc => resp
                .origin_name
                .chars()
                .next()
                .map_or_else(|| "?".to_owned(), |c| c.to_uppercase().to_string()),
            Self::DestAsc | Self::DestDesc => resp
                .dest_name
                .chars()
                .next()
                .map_or_else(|| "?".to_owned(), |c| c.to_uppercase().to_string()),
        }
    }

    /// All variants for rendering the dropdown.
    #[must_use]
    pub fn all() -> &'static [Self] {
        &[
            Self::DateDesc,
            Self::DateAsc,
            Self::OriginAsc,
            Self::OriginDesc,
            Self::DestAsc,
            Self::DestDesc,
        ]
    }
}

/// Query parameters for filtering journey lists.
#[derive(Deserialize, JsonSchema)]
pub struct JourneyQuery {
    #[serde(rename = "type")]
    travel_type: Option<JourneyTravelType>,
    origin: Option<String>,
    dest: Option<String>,
    date_from: Option<String>,
    date_to: Option<String>,
    airline: Option<String>,
    flight_number: Option<String>,
    cabin_class: Option<String>,
    flight_reason: Option<String>,
    q: Option<String>,
    sort: Option<JourneySort>,
}

pub async fn handler(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(query): Query<JourneyQuery>,
    headers: HeaderMap,
) -> Response {
    let format = negotiate_format(&headers);
    let hops = match (db::hops::Search {
        user_id: auth.user_id,
        travel_type: query.travel_type.as_ref().map(JourneyTravelType::as_str),
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

    let mut responses: Vec<JourneyResponse> = hops.into_iter().map(JourneyResponse::from).collect();

    let hop_ids: Vec<i64> = responses.iter().map(|r| r.id).collect();
    if let Ok(enrichments) = (db::status_enrichments::GetByHopIds { hop_ids })
        .execute(&state.db)
        .await
    {
        let enrichment_map: std::collections::HashMap<i64, db::status_enrichments::Row> =
            enrichments.into_iter().map(|e| (e.hop_id, e)).collect();
        for response in &mut responses {
            if let Some(enrichment) = enrichment_map.get(&response.id) {
                response.apply_enrichment(enrichment);
            }
        }
    }

    let hop_ids_for_opensky: Vec<i64> = responses
        .iter()
        .filter(|r| r.travel_type == JourneyTravelType::Air)
        .map(|r| r.id)
        .collect();
    if !hop_ids_for_opensky.is_empty()
        && let Ok(opensky_enrichments) = (db::status_enrichments::GetByHopIdsAndProvider {
            hop_ids: hop_ids_for_opensky,
            provider: "opensky",
        })
        .execute(&state.db)
        .await
    {
        let opensky_map: std::collections::HashMap<i64, db::status_enrichments::Row> =
            opensky_enrichments
                .into_iter()
                .map(|e| (e.hop_id, e))
                .collect();
        for response in &mut responses {
            if let Some(enrichment) = opensky_map.get(&response.id) {
                response.apply_opensky_verification(enrichment);
            }
        }
    }

    let sort = query.sort.unwrap_or_default();
    match sort {
        JourneySort::DateDesc => responses.sort_by(|a, b| b.start_date.cmp(&a.start_date)),
        JourneySort::DateAsc => responses.sort_by(|a, b| a.start_date.cmp(&b.start_date)),
        JourneySort::OriginAsc => responses.sort_by(|a, b| a.origin_name.cmp(&b.origin_name)),
        JourneySort::OriginDesc => responses.sort_by(|a, b| b.origin_name.cmp(&a.origin_name)),
        JourneySort::DestAsc => responses.sort_by(|a, b| a.dest_name.cmp(&b.dest_name)),
        JourneySort::DestDesc => responses.sort_by(|a, b| b.dest_name.cmp(&a.dest_name)),
    }

    if format != ResponseFormat::Html {
        return JourneyResponse::into_format_response(&responses, format, StatusCode::OK);
    }

    build_journey_list_html(&responses, sort)
}

fn build_journey_list_html(responses: &[JourneyResponse], sort: JourneySort) -> Response {
    let total = responses.len();

    let sort_options: Vec<AnyView> = JourneySort::all()
        .iter()
        .map(|&variant| {
            let value = variant.as_str();
            let label = variant.label();
            let selected = variant == sort;
            view! { <option value=value selected=selected>{label}</option> }.into_any()
        })
        .collect();

    let mut groups: Vec<(String, Vec<AnyView>)> = Vec::new();
    for resp in responses {
        let key = sort.group_key(resp);
        if groups.last().is_some_and(|(k, _)| *k == key) {
            groups.last_mut().unwrap().1.push(resp.html_card());
        } else {
            groups.push((key, vec![resp.html_card()]));
        }
    }

    let content: Vec<AnyView> = groups
        .into_iter()
        .map(|(heading_key, cards)| {
            let count = cards.len();
            let heading = format!("{heading_key} ({count})");
            view! {
                <section class="journey-year-section">
                    <h2 class="journey-year-heading">{heading}</h2>
                    <div class="data-card-list">{cards}</div>
                </section>
            }
            .into_any()
        })
        .collect();

    let title = JourneyResponse::HTML_TITLE;
    let html = view! {
        <Shell title=title.to_owned()>
            <NavBar current="journeys" />
            <main class="data-page">
                <div class="data-page-header">
                    <h1>{title}</h1>
                    <span class="data-page-count">{format!("{total} records")}</span>
                    <form class="sort-control" method="get" action="/journeys">
                        <label for="sort-select" class="sort-label">"Sort"</label>
                        <select id="sort-select" name="sort" onchange="this.form.submit()">
                            {sort_options}
                        </select>
                    </form>
                </div>
                {content}
            </main>
        </Shell>
    };

    axum::response::Html(html.to_html()).into_response()
}

/// `OpenAPI` metadata for the list journeys endpoint.
pub fn handler_docs(op: TransformOperation) -> TransformOperation {
    multi_format_docs!(
        op.description("List travel journeys for the authenticated user.")
            .response_with::<200, Json<Vec<JourneyResponse>>, _>(|mut res| {
                add_multi_format_docs::<JourneyResponse>(res.inner());
                res
            }),
        401 | 500 => ErrorResponse,
    )
    .tag("journeys")
}

pub async fn get_journey_handler(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i64>,
    Query(feedback): Query<crate::server::pages::journey_detail::JourneyDetailFeedback>,
    headers: HeaderMap,
) -> Response {
    let format = negotiate_format(&headers);

    let detail = match (db::hops::GetById {
        id,
        user_id: auth.user_id,
    })
    .execute(&state.db)
    .await
    {
        Ok(Some(detail)) => detail,
        Ok(None) => {
            return if format == super::ResponseFormat::Html {
                crate::server::pages::not_found::page().await
            } else {
                (
                    StatusCode::NOT_FOUND,
                    Json(ErrorResponse {
                        error: "journey not found".to_owned(),
                    }),
                )
                    .into_response()
            };
        }
        Err(err) => {
            return AppError::from(err).into_format_response(format);
        }
    };

    let enrichment = (db::status_enrichments::GetByHopId { hop_id: detail.id })
        .execute(&state.db)
        .await
        .ok()
        .flatten();

    let opensky_verification = (db::status_enrichments::GetByHopIdAndProvider {
        hop_id: detail.id,
        provider: "opensky",
    })
    .execute(&state.db)
    .await
    .ok()
    .flatten();

    let attachments = (db::attachments::GetByHopId {
        hop_id: detail.id,
        user_id: auth.user_id,
    })
    .execute(&state.db)
    .await
    .unwrap_or_default();

    if format == super::ResponseFormat::Html {
        crate::server::pages::journey_detail::render_page(
            detail,
            feedback,
            enrichment,
            attachments,
            opensky_verification,
        )
    } else {
        let mut response: JourneyResponse = detail.into();
        if let Some(ref e) = enrichment {
            response.apply_enrichment(e);
        }
        if let Some(ref e) = opensky_verification {
            response.apply_opensky_verification(e);
        }
        JourneyResponse::single_format_response(&response, format, StatusCode::OK)
    }
}

/// `OpenAPI` metadata for the get journey by ID endpoint.
pub fn get_journey_handler_docs(op: TransformOperation) -> TransformOperation {
    multi_format_docs!(
        op.description("Get a single journey by ID.")
            .response_with::<200, Json<JourneyResponse>, _>(|mut res| {
                add_multi_format_docs::<JourneyResponse>(res.inner());
                res
            }),
        401 | 404 | 500 => ErrorResponse,
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

fn has_coords(origin_lat: f64, origin_lng: f64, dest_lat: f64, dest_lng: f64) -> bool {
    origin_lat != 0.0 && origin_lng != 0.0 && dest_lat != 0.0 && dest_lng != 0.0
}

fn auto_miles_if_air(
    travel_type: &JourneyTravelType,
    miles_earned: Option<f64>,
    origin_lat: f64,
    origin_lng: f64,
    dest_lat: f64,
    dest_lng: f64,
) -> Option<f64> {
    if *travel_type != JourneyTravelType::Air || miles_earned.is_some() {
        return miles_earned;
    }
    if !has_coords(origin_lat, origin_lng, dest_lat, dest_lng) {
        return None;
    }
    let computed = haversine_miles(origin_lat, origin_lng, dest_lat, dest_lng);
    if computed.is_finite() && computed > 0.0 {
        Some(computed)
    } else {
        None
    }
}

fn update_form_error_redirect(id: i64, message: &str) -> Response {
    Redirect::to(&format!(
        "/journeys/{id}?error={}",
        encode_query_value(message)
    ))
    .into_response()
}

/// Request body for manually creating a journey of any travel type.
#[derive(Default, Deserialize, JsonSchema)]
pub struct CreateJourneyRequest {
    /// Mode of transport (defaults to `air` when omitted).
    #[serde(default)]
    pub travel_type: JourneyTravelType,
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
    #[serde(default)]
    pub cost_amount: Option<f64>,
    #[serde(default)]
    pub cost_currency: Option<String>,
    #[serde(default)]
    pub loyalty_program: Option<String>,
    #[serde(default)]
    pub miles_earned: Option<f64>,
}

impl CreateJourneyRequest {
    fn build_manual_detail(&self) -> db::hops::ManualDetail {
        match self.travel_type {
            JourneyTravelType::Air => db::hops::ManualDetail::Air(db::hops::FlightDetail {
                airline: self.airline.clone().unwrap_or_default(),
                flight_number: self.flight_number.clone().unwrap_or_default(),
                aircraft_type: self.aircraft_type.clone().unwrap_or_default(),
                cabin_class: self.cabin_class.clone().unwrap_or_default(),
                seat: self.seat.clone().unwrap_or_default(),
                pnr: self.pnr.clone().unwrap_or_default(),
            }),
            JourneyTravelType::Rail => db::hops::ManualDetail::Rail(db::hops::RailDetail {
                carrier: self.rail_carrier.clone().unwrap_or_default(),
                train_number: self.train_number.clone().unwrap_or_default(),
                service_class: self.service_class.clone().unwrap_or_default(),
                coach_number: self.coach_number.clone().unwrap_or_default(),
                seats: self.rail_seats.clone().unwrap_or_default(),
                confirmation_num: self.rail_confirmation.clone().unwrap_or_default(),
                booking_site: self.rail_booking_site.clone().unwrap_or_default(),
                notes: self.rail_notes.clone().unwrap_or_default(),
            }),
            JourneyTravelType::Boat => db::hops::ManualDetail::Boat(db::hops::BoatDetail {
                ship_name: self.ship_name.clone().unwrap_or_default(),
                cabin_type: self.cabin_type.clone().unwrap_or_default(),
                cabin_number: self.cabin_number.clone().unwrap_or_default(),
                confirmation_num: self.boat_confirmation.clone().unwrap_or_default(),
                booking_site: self.boat_booking_site.clone().unwrap_or_default(),
                notes: self.boat_notes.clone().unwrap_or_default(),
            }),
            JourneyTravelType::Transport => {
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

/// Successful response after creating a journey.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreateJourneyResponse {
    /// Number of journeys created.
    pub created: u64,
}

/// Create a journey manually for any travel type.
///
/// Accepts JSON or form-encoded body. Form submissions redirect to the add
/// journey page with a success or error query parameter.
pub async fn create_handler(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> Response {
    let is_form = is_form_request(&headers);

    let req = match FormOrJson::<CreateJourneyRequest>::parse(&headers, &body) {
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
    let miles_earned = match &detail {
        db::hops::ManualDetail::Air(_) => {
            let origin = crate::geocode::airports::lookup_enriched(&req.origin);
            let dest = crate::geocode::airports::lookup_enriched(&req.destination);
            auto_miles_if_air(
                &req.travel_type,
                req.miles_earned,
                origin.as_ref().map_or(0.0, |a| a.latitude),
                origin.as_ref().map_or(0.0, |a| a.longitude),
                dest.as_ref().map_or(0.0, |a| a.latitude),
                dest.as_ref().map_or(0.0, |a| a.longitude),
            )
        }
        _ => req.miles_earned,
    };

    let result = (db::hops::CreateManual {
        user_id: auth.user_id,
        origin: req.origin,
        destination: req.destination,
        date: req.date,
        detail,
        cost_amount: req.cost_amount,
        cost_currency: req.cost_currency,
        loyalty_program: req.loyalty_program.as_deref(),
        miles_earned,
    })
    .execute(&state.db)
    .await;

    match result {
        Ok(created) => {
            if is_form {
                Redirect::to("/journeys/new?success=1").into_response()
            } else {
                (StatusCode::CREATED, Json(CreateJourneyResponse { created })).into_response()
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

/// `OpenAPI` metadata for the create journey endpoint.
pub fn create_handler_docs(op: TransformOperation) -> TransformOperation {
    multi_format_docs!(
        op.description(
            "Create a journey manually for any travel type. Accepts JSON or form-encoded body.",
        )
        .input::<FormOrJson<CreateJourneyRequest>>()
        .response::<201, Json<CreateJourneyResponse>>(),
        400 | 401 | 500 => ErrorResponse,
    )
    .tag("journeys")
}

/// Request body for updating an existing journey.
#[derive(Default, Deserialize, JsonSchema)]
pub struct UpdateJourneyRequest {
    pub travel_type: JourneyTravelType,
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
    #[serde(default)]
    pub cost_amount: Option<f64>,
    #[serde(default)]
    pub cost_currency: Option<String>,
    #[serde(default)]
    pub loyalty_program: Option<String>,
    #[serde(default)]
    pub miles_earned: Option<f64>,
}

impl UpdateJourneyRequest {
    fn build_flight_detail(&self) -> Option<db::hops::FullFlightDetail> {
        (self.travel_type == JourneyTravelType::Air).then(|| db::hops::FullFlightDetail {
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
        (self.travel_type == JourneyTravelType::Rail).then(|| db::hops::RailDetail {
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
        (self.travel_type == JourneyTravelType::Boat).then(|| db::hops::BoatDetail {
            ship_name: self.ship_name.clone().unwrap_or_default(),
            cabin_type: self.cabin_type.clone().unwrap_or_default(),
            cabin_number: self.cabin_number.clone().unwrap_or_default(),
            confirmation_num: self.boat_confirmation.clone().unwrap_or_default(),
            booking_site: self.boat_booking_site.clone().unwrap_or_default(),
            notes: self.boat_notes.clone().unwrap_or_default(),
        })
    }

    fn build_transport_detail(&self) -> Option<db::hops::TransportDetail> {
        (self.travel_type == JourneyTravelType::Transport).then(|| db::hops::TransportDetail {
            carrier_name: self.transport_carrier.clone().unwrap_or_default(),
            vehicle_description: self.vehicle_description.clone().unwrap_or_default(),
            confirmation_num: self.transport_confirmation.clone().unwrap_or_default(),
            notes: self.transport_notes.clone().unwrap_or_default(),
        })
    }
}

/// JSON response after updating a journey.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct UpdateJourneyResponse {
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

    let req = match FormOrJson::<UpdateJourneyRequest>::parse(&headers, &body) {
        Ok(r) => r,
        Err(err) => {
            return if is_form {
                update_form_error_redirect(id, &format!("Invalid form data: {err}"))
            } else {
                let format = negotiate_format(&headers);
                err.into_format_response(format)
            };
        }
    };

    if req.origin_name.is_empty() || req.dest_name.is_empty() || req.start_date.is_empty() {
        let err = AppError::MissingField("origin_name, dest_name, and start_date are required");
        return if is_form {
            update_form_error_redirect(id, &err.to_string())
        } else {
            let format = negotiate_format(&headers);
            err.into_format_response(format)
        };
    }

    let flight_detail = req.build_flight_detail();
    let rail_detail = req.build_rail_detail();
    let boat_detail = req.build_boat_detail();
    let transport_detail = req.build_transport_detail();
    let travel_type = req.travel_type.clone();

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
        travel_type: travel_type.clone().into(),
        flight_detail,
        rail_detail,
        boat_detail,
        transport_detail,
        cost_amount: req.cost_amount,
        cost_currency: req.cost_currency,
        loyalty_program: req.loyalty_program.as_deref(),
        miles_earned: auto_miles_if_air(
            &travel_type,
            req.miles_earned,
            req.origin_lat,
            req.origin_lng,
            req.dest_lat,
            req.dest_lng,
        ),
    })
    .execute(&state.db)
    .await;

    match result {
        Ok(true) => {
            if is_form {
                Redirect::to(&format!("/journeys/{id}?success=1")).into_response()
            } else {
                (
                    StatusCode::OK,
                    Json(UpdateJourneyResponse { updated: true }),
                )
                    .into_response()
            }
        }
        Ok(false) => {
            if is_form {
                update_form_error_redirect(id, "Journey not found")
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
                update_form_error_redirect(id, &err.to_string())
            } else {
                let format = negotiate_format(&headers);
                err.into_format_response(format)
            }
        }
    }
}

/// `OpenAPI` metadata for the update journey endpoint.
pub fn update_handler_docs(op: TransformOperation) -> TransformOperation {
    multi_format_docs!(
        op.description("Update an existing journey by ID. Accepts JSON or form-encoded body.")
            .input::<FormOrJson<UpdateJourneyRequest>>()
            .response::<200, Json<UpdateJourneyResponse>>(),
        400 | 401 | 404 | 500 => ErrorResponse,
    )
    .tag("journeys")
}

#[cfg(test)]
mod tests {
    use super::{CreateJourneyResponse, JourneyResponse, JourneyTravelType};
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
        let parsed: Vec<JourneyResponse> =
            serde_json::from_slice(&body).expect("body should be valid JSON array");

        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].travel_type, JourneyTravelType::Air);
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
        let parsed: Vec<JourneyResponse> = serde_json::from_slice(&body).expect("valid json");
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].travel_type, JourneyTravelType::Rail);
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
        let parsed: CreateJourneyResponse =
            serde_json::from_slice(&body).expect("valid json response");
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
        let parsed: CreateJourneyResponse =
            serde_json::from_slice(&body).expect("valid json response");
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

use crate::server::{AppState, components::AddFlightPage, middleware::AuthUser};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use leptos::prelude::*;
use serde::Deserialize;

#[derive(Deserialize, Default)]
pub struct AddFlightFeedback {
    pub error: Option<String>,
    pub success: Option<String>,
}

pub async fn add_flight_page(
    State(_state): State<AppState>,
    _auth: AuthUser,
    Query(feedback): Query<AddFlightFeedback>,
) -> Response {
    let html = view! {
        <AddFlightPage error=feedback.error success=feedback.success />
    };
    (StatusCode::OK, axum::response::Html(html.to_html())).into_response()
}

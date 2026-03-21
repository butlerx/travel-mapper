use crate::{
    db,
    server::{AppState, middleware::AuthUser, session::clear_session_cookie},
};
use aide::transform::TransformOperation;
use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::CookieJar;
use schemars::JsonSchema;
use serde::Serialize;
use serde_json::json;

#[derive(Debug, Serialize, JsonSchema)]
pub struct StatusResponse {
    pub status: String,
}

/// Log out and invalidate the current session.
pub async fn logout_handler(
    State(state): State<AppState>,
    jar: CookieJar,
    headers: HeaderMap,
    auth: AuthUser,
) -> (CookieJar, Response) {
    if let Some(cookie) = jar.get("session_id") {
        let _ = db::delete_session(&state.db, cookie.value()).await;
    }
    let _ = auth;

    let wants_html = headers
        .get(header::ACCEPT)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.contains("text/html"));

    let updated_jar = jar.remove(clear_session_cookie());
    (
        updated_jar,
        if wants_html {
            Redirect::to("/login").into_response()
        } else {
            (StatusCode::OK, Json(json!({ "status": "ok" }))).into_response()
        },
    )
}

pub fn logout_handler_docs(op: TransformOperation) -> TransformOperation {
    op.description("Log out and invalidate the current session.")
        .response::<200, Json<StatusResponse>>()
        .tag("auth")
}

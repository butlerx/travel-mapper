use super::{
    ErrorResponse, MultiFormatResponse, ResponseFormat, StatusResponse, multi_format_docs,
    negotiate_format,
};
use crate::{
    db,
    server::{AppState, middleware::AuthUser, session::clear_session_cookie},
};
use aide::transform::TransformOperation;
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::CookieJar;

/// Log out and invalidate the current session.
pub async fn logout_handler(
    State(state): State<AppState>,
    jar: CookieJar,
    headers: HeaderMap,
    auth: AuthUser,
) -> (CookieJar, Response) {
    if let Some(cookie) = jar.get("session_id") {
        let _ = (db::sessions::Delete {
            token: cookie.value(),
        })
        .execute(&state.db)
        .await;
    }
    let _ = auth;

    let format = negotiate_format(&headers);
    let updated_jar = jar.remove(clear_session_cookie());
    (
        updated_jar,
        if format == ResponseFormat::Html {
            Redirect::to("/login").into_response()
        } else {
            let response = StatusResponse {
                status: "ok".to_string(),
            };
            StatusResponse::single_format_response(&response, format, StatusCode::OK)
        },
    )
}

pub fn logout_handler_docs(op: TransformOperation) -> TransformOperation {
    multi_format_docs!(
        op.description("Log out and invalidate the current session."),
        200 => StatusResponse,
        401 => ErrorResponse,
    )
    .tag("auth")
}

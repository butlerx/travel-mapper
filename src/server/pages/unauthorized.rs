use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

#[must_use]
pub fn page() -> Response {
    (
        StatusCode::UNAUTHORIZED,
        axum::response::Html(super::render_error_page(
            "401",
            "Unauthorized",
            "You need to log in to access this page.",
            "/login",
            "Log In",
        )),
    )
        .into_response()
}

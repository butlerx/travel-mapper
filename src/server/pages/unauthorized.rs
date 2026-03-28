use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

/// Render the 401 Unauthorized error page.
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

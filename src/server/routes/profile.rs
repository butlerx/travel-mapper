use super::{ErrorResponse, MultiFormatResponse, StatusResponse, negotiate_format};
use crate::{
    db,
    server::{
        AppState,
        error::AppError,
        extractors::{AuthUser, FormOrJson},
        session::is_form_request,
    },
};
use aide::transform::TransformOperation;
use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Redirect, Response},
};
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Default, Deserialize, JsonSchema)]
pub struct UpdateProfileRequest {
    pub first_name: String,
    pub last_name: String,
}

pub async fn handler(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let parsed = match FormOrJson::<UpdateProfileRequest>::parse(&headers, &body) {
        Ok(v) => v,
        Err(err) => {
            let format = negotiate_format(&headers);
            return err.into_format_response(format);
        }
    };

    if let Err(err) = (db::users::UpdateProfile {
        user_id: auth.user_id,
        first_name: parsed.first_name.trim(),
        last_name: parsed.last_name.trim(),
    })
    .execute(&state.db)
    .await
    {
        let format = negotiate_format(&headers);
        return AppError::from(err).into_format_response(format);
    }

    if is_form_request(&headers) {
        Redirect::to("/settings?profile=updated").into_response()
    } else {
        let format = negotiate_format(&headers);
        let resp = StatusResponse {
            status: "ok".to_owned(),
        };
        StatusResponse::single_format_response(&resp, format, StatusCode::OK)
    }
}

pub fn handler_docs(op: TransformOperation) -> TransformOperation {
    op.description("Update the authenticated user's profile (first and last name).")
        .input::<FormOrJson<UpdateProfileRequest>>()
        .response::<200, axum::Json<StatusResponse>>()
        .response::<401, axum::Json<ErrorResponse>>()
        .response::<500, axum::Json<ErrorResponse>>()
        .tag("auth")
}

#[cfg(test)]
mod tests {
    use crate::server::create_router;
    use crate::server::test_helpers::*;
    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode, header},
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn update_profile_sets_names() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/auth/profile")
                    .header(header::CONTENT_TYPE, "application/json")
                    .header(header::COOKIE, cookie)
                    .body(Body::from(r#"{"first_name":"Alice","last_name":"Smith"}"#))
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("failed to read body");
        let json: serde_json::Value = serde_json::from_slice(&body).expect("valid json");
        assert_eq!(json["status"], "ok");
    }

    #[tokio::test]
    async fn update_profile_form_redirects() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/auth/profile")
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .header(header::COOKIE, cookie)
                    .body(Body::from("first_name=Alice&last_name=Smith"))
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert!(
            response
                .headers()
                .get(header::LOCATION)
                .and_then(|v| v.to_str().ok())
                .unwrap_or("")
                .contains("/settings")
        );
    }
}

use crate::{
    db,
    server::{AppState, components::HopDetailPage, middleware::AuthUser},
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use leptos::prelude::*;

pub async fn hop_detail_page(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<i64>,
) -> Response {
    match (db::hops::GetById {
        id,
        user_id: auth.user_id,
    })
    .execute(&state.db)
    .await
    {
        Ok(Some(hop)) => {
            let html = view! { <HopDetailPage hop=hop /> };
            (StatusCode::OK, axum::response::Html(html.to_html())).into_response()
        }
        Ok(None) => super::not_found_page().await,
        Err(err) => {
            tracing::error!(hop_id = id, %err, "failed to fetch hop detail");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::response::Html(super::render_error_page(
                    "500",
                    "Server Error",
                    "Something went wrong loading this hop.",
                    "/dashboard",
                    "Back to Dashboard",
                )),
            )
                .into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        db::{
            self,
            hops::{Create, GetAll, TravelType},
        },
        server::{create_router, test_helpers::helpers::*},
    };
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    async fn insert_hop_for_user(pool: &sqlx::SqlitePool, username: &str) -> (String, i64) {
        let cookie = auth_cookie_for_user(pool, username).await;
        let user = db::users::GetByUsername { username }
            .execute(pool)
            .await
            .expect("lookup")
            .expect("user exists");

        let hop = sample_hop(
            TravelType::Air,
            "Dublin",
            "London Heathrow",
            "2024-06-01",
            "2024-06-01",
        );

        Create {
            trip_id: "trip-test",
            user_id: user.id,
            hops: &[hop],
        }
        .execute(pool)
        .await
        .expect("insert hop");

        let rows = GetAll {
            user_id: user.id,
            travel_type_filter: None,
        }
        .execute(pool)
        .await
        .expect("get hops");

        (cookie, rows[0].id)
    }

    #[tokio::test]
    async fn hop_detail_page_renders_flight() {
        let pool = test_pool().await;
        let (cookie, hop_id) = insert_hop_for_user(&pool, "alice").await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/hop/{hop_id}"))
                    .header("cookie", &cookie)
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_text(response).await;
        assert!(body.contains("Dublin"), "should contain origin name");
        assert!(body.contains("London Heathrow"), "should contain dest name");
        assert!(body.contains("hop-map"), "should contain map div");
    }

    #[tokio::test]
    async fn hop_detail_page_returns_404_for_missing_hop() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/hop/99999")
                    .header("cookie", &cookie)
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn hop_detail_page_redirects_without_auth() {
        let pool = test_pool().await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/hop/1")
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert!(
            response.status() == StatusCode::UNAUTHORIZED
                || response.status() == StatusCode::SEE_OTHER,
            "expected 401 or redirect, got {}",
            response.status()
        );
    }

    #[tokio::test]
    async fn hop_detail_page_returns_404_for_other_users_hop() {
        let pool = test_pool().await;
        let (_alice_cookie, hop_id) = insert_hop_for_user(&pool, "alice").await;
        let bob_cookie = auth_cookie_for_user(&pool, "bob").await;

        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/hop/{hop_id}"))
                    .header("cookie", &bob_cookie)
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        // 404, not 403 — don't leak existence
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}

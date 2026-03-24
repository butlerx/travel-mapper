//! Application state and Axum router construction.

use crate::{
    integrations::tripit::TripItApi,
    server::{pages, routes},
};
use aide::{
    axum::IntoApiResponse,
    openapi::{ApiKeyLocation, OpenApi, SecurityScheme, Tag},
    swagger::Swagger,
    transform::TransformOpenApi,
};
use axum::{
    Extension, Json, Router,
    extract::{FromRef, MatchedPath},
    routing::get,
};
use indexmap::IndexMap;
use leptos::prelude::LeptosOptions;
use sqlx::SqlitePool;
use std::{sync::Arc, time::Duration};
use tower_http::trace::TraceLayer;

/// Shared application state passed to every Axum handler.
#[derive(Clone)]
pub struct AppState {
    pub leptos_options: LeptosOptions,
    pub db: SqlitePool,
    pub encryption_key: [u8; 32],
    pub tripit_consumer_key: String,
    pub tripit_consumer_secret: String,
    pub tripit_override: Option<Arc<dyn TripItApi>>,
}

impl FromRef<AppState> for LeptosOptions {
    fn from_ref(state: &AppState) -> Self {
        state.leptos_options.clone()
    }
}

async fn serve_api(Extension(api): Extension<Arc<OpenApi>>) -> impl IntoApiResponse {
    Json(api)
}

fn api_docs(api: TransformOpenApi) -> TransformOpenApi {
    api.title(env!("CARGO_PKG_NAME"))
        .summary(env!("CARGO_PKG_DESCRIPTION"))
        .version(env!("CARGO_PKG_VERSION"))
        .tag(Tag {
            name: "health".into(),
            description: Some("Health check".into()),
            ..Default::default()
        })
        .tag(Tag {
            name: "hops".into(),
            description: Some("Travel hop queries".into()),
            ..Default::default()
        })
        .tag(Tag {
            name: "sync".into(),
            description: Some("TripIt sync".into()),
            ..Default::default()
        })
        .tag(Tag {
            name: "trips".into(),
            description: Some("User trip grouping and assignments".into()),
            ..Default::default()
        })
        .tag(Tag {
            name: "auth".into(),
            description: Some("Authentication and API keys".into()),
            ..Default::default()
        })
        .tag(Tag {
            name: "tripit".into(),
            description: Some("TripIt OAuth integration".into()),
            ..Default::default()
        })
        .security_scheme(
            "bearer",
            SecurityScheme::Http {
                scheme: "bearer".into(),
                bearer_format: None,
                description: Some("API key passed as Bearer token".into()),
                extensions: IndexMap::default(),
            },
        )
        .security_scheme(
            "cookie",
            SecurityScheme::ApiKey {
                location: ApiKeyLocation::Cookie,
                name: "session_id".into(),
                description: Some("Session cookie from login".into()),
                extensions: IndexMap::default(),
            },
        )
}

/// Build the full Axum router with API routes, pages, `OpenAPI` docs, and middleware.
pub fn create_router(state: AppState) -> Router {
    let mut api = OpenApi::default();

    pages::page_routes()
        .merge(routes::toplevel_api_routes())
        .nest("/static", routes::static_assets::routes())
        .route("/manifest.json", get(routes::static_assets::serve_manifest))
        .route("/sw.js", get(routes::static_assets::serve_sw))
        .nest("/auth", routes::auth_api_routes())
        .nest("/auth/tripit", routes::tripit_api_routes())
        .nest("/hops", routes::hops_api_routes())
        .nest("/trips", routes::trip_api_routes())
        .nest("/import", routes::import_api_routes())
        .route(
            "/docs",
            get(Swagger::new("/openapi.json")
                .with_title(super::APP_NAME)
                .axum_handler()),
        )
        .route("/openapi.json", get(serve_api))
        .finish_api_with(&mut api, api_docs)
        .layer(Extension(Arc::new(api)))
        .fallback(pages::not_found::page)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &axum::http::Request<_>| {
                    let path = request.extensions().get::<MatchedPath>().map_or_else(
                        || request.uri().path().to_owned(),
                        |m| m.as_str().to_owned(),
                    );
                    tracing::info_span!(
                        "http",
                        method = %request.method(),
                        path,
                    )
                })
                .on_response(
                    |response: &axum::http::Response<_>,
                     latency: Duration,
                     _span: &tracing::Span| {
                        tracing::info!(
                            status = response.status().as_u16(),
                            latency_ms = u64::try_from(latency.as_millis()).unwrap_or(u64::MAX),
                            "response",
                        );
                    },
                ),
        )
        .with_state(state)
}

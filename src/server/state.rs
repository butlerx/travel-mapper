//! Application state and Axum router construction.

use crate::{
    integrations::tripit::TripItApi,
    server::{pages, routes},
};
use aide::{
    axum::{ApiRouter, IntoApiResponse, routing::get_with},
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

/// Register page and static-asset routes that do not need `OpenAPI` metadata.
fn page_routes(router: ApiRouter<AppState>) -> ApiRouter<AppState> {
    let static_assets = routes::static_assets::routes();

    router
        .route("/", get(pages::landing::page))
        .route("/register", get(pages::register::page))
        .route("/login", get(pages::login::page))
        .route("/dashboard", get(pages::dashboard::page))
        .route("/settings", get(pages::settings::page))
        .route("/stats", get(pages::stats::page))
        .route("/flights/new", get(pages::add_flight::page))
        .route("/hop/{id}", get(pages::hop_detail::page))
        .nest("/static", static_assets.into())
}

/// Register API routes that carry `OpenAPI` documentation.
fn api_routes(router: ApiRouter<AppState>) -> ApiRouter<AppState> {
    router
        .api_route(
            "/health",
            get_with(routes::health::handler, routes::health::handler_docs),
        )
        .api_route(
            "/sync",
            aide::axum::routing::post_with(routes::sync::handler, routes::sync::handler_docs),
        )
        .api_route(
            "/hops",
            get_with(routes::hops::handler, routes::hops::handler_docs).post_with(
                routes::hops::create_handler,
                routes::hops::create_handler_docs,
            ),
        )
        .api_route(
            "/hops/{id}",
            aide::axum::routing::put_with(
                routes::hops::update_handler,
                routes::hops::update_handler_docs,
            )
            .post_with(
                routes::hops::update_handler,
                routes::hops::update_handler_docs,
            ),
        )
        .route(
            "/import/flighty",
            axum::routing::post(routes::flighty::handler),
        )
        .api_route(
            "/auth/register",
            aide::axum::routing::post_with(
                routes::register::handler,
                routes::register::handler_docs,
            ),
        )
        .api_route(
            "/auth/login",
            aide::axum::routing::post_with(routes::login::handler, routes::login::handler_docs),
        )
        .api_route(
            "/auth/logout",
            aide::axum::routing::post_with(routes::logout::handler, routes::logout::handler_docs),
        )
        .api_route(
            "/auth/api-keys",
            aide::axum::routing::post_with(
                routes::api_keys::handler,
                routes::api_keys::handler_docs,
            ),
        )
        .api_route(
            "/auth/tripit",
            aide::axum::routing::put_with(
                routes::tripit_credentials::handler,
                routes::tripit_credentials::handler_docs,
            ),
        )
        .api_route(
            "/auth/tripit/connect",
            get_with(
                routes::tripit_connect::handler,
                routes::tripit_connect::handler_docs,
            ),
        )
        .api_route(
            "/auth/tripit/callback",
            get_with(
                routes::tripit_callback::handler,
                routes::tripit_callback::handler_docs,
            ),
        )
}

/// Build the full Axum router with API routes, pages, `OpenAPI` docs, and middleware.
pub fn create_router(state: AppState) -> Router {
    let mut api = OpenApi::default();

    let router = ApiRouter::new();
    let router = page_routes(router);
    let router = api_routes(router);

    router
        .route(
            "/docs",
            get(Swagger::new("/openapi.json")
                .with_title(env!("CARGO_PKG_NAME"))
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

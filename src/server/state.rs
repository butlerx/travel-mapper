use crate::{
    integrations::tripit::TripItApi,
    server::pages::{
        add_flight_page, dashboard_page, landing_page, login_page, not_found_page, register_page,
        settings_page, stats_page,
    },
    server::routes::{
        create_api_key_handler, create_api_key_handler_docs, create_hop_handler,
        create_hop_handler_docs, health_handler, health_handler_docs, hops_handler,
        hops_handler_docs, import_flighty_handler, login_handler, login_handler_docs,
        logout_handler, logout_handler_docs, register_handler, register_handler_docs, serve_css,
        serve_js, store_tripit_credentials_handler, store_tripit_credentials_handler_docs,
        sync_handler, sync_handler_docs, tripit_callback_handler, tripit_callback_handler_docs,
        tripit_connect_handler, tripit_connect_handler_docs,
    },
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

pub fn create_router(state: AppState) -> Router {
    let mut api = OpenApi::default();

    ApiRouter::new()
        .route("/", get(landing_page))
        .route("/register", get(register_page))
        .route("/login", get(login_page))
        .route("/dashboard", get(dashboard_page))
        .route("/settings", get(settings_page))
        .route("/stats", get(stats_page))
        .route("/flights/new", get(add_flight_page))
        .route("/static/style.css", get(serve_css))
        .route("/static/map.js", get(serve_js))
        .api_route("/health", get_with(health_handler, health_handler_docs))
        .api_route(
            "/sync",
            aide::axum::routing::post_with(sync_handler, sync_handler_docs),
        )
        .api_route(
            "/hops",
            get_with(hops_handler, hops_handler_docs)
                .post_with(create_hop_handler, create_hop_handler_docs),
        )
        .route(
            "/import/flighty",
            axum::routing::post(import_flighty_handler),
        )
        .api_route(
            "/auth/register",
            aide::axum::routing::post_with(register_handler, register_handler_docs),
        )
        .api_route(
            "/auth/login",
            aide::axum::routing::post_with(login_handler, login_handler_docs),
        )
        .api_route(
            "/auth/logout",
            aide::axum::routing::post_with(logout_handler, logout_handler_docs),
        )
        .api_route(
            "/auth/api-keys",
            aide::axum::routing::post_with(create_api_key_handler, create_api_key_handler_docs),
        )
        .api_route(
            "/auth/tripit",
            aide::axum::routing::put_with(
                store_tripit_credentials_handler,
                store_tripit_credentials_handler_docs,
            ),
        )
        .api_route(
            "/auth/tripit/connect",
            get_with(tripit_connect_handler, tripit_connect_handler_docs),
        )
        .api_route(
            "/auth/tripit/callback",
            get_with(tripit_callback_handler, tripit_callback_handler_docs),
        )
        .route(
            "/docs",
            get(Swagger::new("/openapi.json")
                .with_title(env!("CARGO_PKG_NAME"))
                .axum_handler()),
        )
        .route("/openapi.json", get(serve_api))
        .finish_api_with(&mut api, api_docs)
        .layer(Extension(Arc::new(api)))
        .fallback(not_found_page)
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

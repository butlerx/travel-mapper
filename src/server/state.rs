//! Application state and Axum router construction.

use crate::{
    integrations::tripit::TripItApi,
    server::{middleware, pages, routes},
};
use aide::{
    axum::IntoApiResponse,
    openapi::{ApiKeyLocation, OpenApi, SecurityScheme, Tag},
    swagger::Swagger,
    transform::TransformOpenApi,
};
use axum::{Extension, Json, Router, extract::FromRef, routing::get};
use indexmap::IndexMap;
use leptos::prelude::LeptosOptions;
use sqlx::SqlitePool;
use std::{path::PathBuf, sync::Arc};
use tower_http::trace::TraceLayer;

/// SMTP connection details for sending transactional email (verification, etc.).
///
/// All fields are required; when absent, email sending is silently skipped.
#[derive(Clone, Debug)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub from: String,
}

/// Shared application state passed to every Axum handler.
#[derive(Clone)]
pub struct AppState {
    pub leptos_options: LeptosOptions,
    pub db: SqlitePool,
    pub encryption_key: [u8; 32],
    pub tripit_consumer_key: String,
    pub tripit_consumer_secret: String,
    pub tripit_override: Option<Arc<dyn TripItApi>>,
    /// Whether new user registration is allowed (`REGISTRATION_ENABLED` env var).
    pub registration_enabled: bool,
    /// Optional `AirLabs` API key for flight status enrichment.
    pub airlabs_api_key: Option<String>,
    /// Optional `OpenSky` Network `OAuth2` client ID for route verification.
    pub opensky_client_id: Option<String>,
    /// Optional `OpenSky` Network `OAuth2` client secret for route verification.
    pub opensky_client_secret: Option<String>,
    /// Optional National Rail Darwin API token for UK rail status.
    pub darwin_api_token: Option<String>,
    /// Optional DB RIS API key for German rail status.
    pub db_ris_api_key: Option<String>,
    /// Optional DB RIS client ID for German rail status.
    pub db_ris_client_id: Option<String>,
    /// Optional Transitland API key for multi-region rail status.
    pub transitland_api_key: Option<String>,
    /// Filesystem directory for attachment storage. `None` disables uploads.
    pub storage_path: Option<PathBuf>,
    /// Optional SMTP config for sending verification emails.
    pub smtp_config: Option<SmtpConfig>,
    /// PEM-encoded VAPID private key for Web Push. `None` disables push notifications.
    pub vapid_private_key: Option<Vec<u8>>,
    /// Base64url-encoded VAPID public key served to browsers for subscription.
    pub vapid_public_key: Option<String>,
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
    api.title(super::APP_NAME)
        .summary(env!("CARGO_PKG_DESCRIPTION"))
        .version(env!("CARGO_PKG_VERSION"))
        .tag(Tag {
            name: "health".into(),
            description: Some("Health check".into()),
            ..Default::default()
        })
        .tag(Tag {
            name: "journeys".into(),
            description: Some("Travel journey queries".into()),
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
        .tag(Tag {
            name: "stats".into(),
            description: Some("Aggregated travel statistics".into()),
            ..Default::default()
        })
        .tag(Tag {
            name: "settings".into(),
            description: Some("Account settings".into()),
            ..Default::default()
        })
        .tag(Tag {
            name: "feed".into(),
            description: Some("Calendar ICS feed".into()),
            ..Default::default()
        })
        .tag(Tag {
            name: "share".into(),
            description: Some("Public shareable stats pages".into()),
            ..Default::default()
        })
        .tag(Tag {
            name: "attachments".into(),
            description: Some("Photo and document attachments for journeys".into()),
            ..Default::default()
        })
        .tag(Tag {
            name: "enrichments".into(),
            description: Some("Status enrichment records for journeys".into()),
            ..Default::default()
        })
        .tag(Tag {
            name: "airports".into(),
            description: Some("Airport IATA code reference lookups".into()),
            ..Default::default()
        })
        .tag(Tag {
            name: "stations".into(),
            description: Some("UK CRS station code reference lookups".into()),
            ..Default::default()
        })
        .tag(Tag {
            name: "rail".into(),
            description: Some("Rail operator and GTFS-RT feed discovery".into()),
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
        .nest("/journeys", routes::journeys_api_routes())
        .nest(
            "/journeys/{id}/attachments",
            routes::attachments_api_routes(),
        )
        .nest(
            "/journeys/{id}/enrichments",
            routes::enrichments_api_routes(),
        )
        .nest("/trips", routes::trip_api_routes())
        .nest("/airports", routes::airports_api_routes())
        .nest("/stations", routes::stations_api_routes())
        .nest("/rail", routes::rail_api_routes())
        .nest("/import", routes::import_api_routes())
        .route("/feed/{token}", get(routes::feed::handler))
        .route("/share/{token}", get(routes::share::handler))
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
                .make_span_with(middleware::request_span)
                .on_response(middleware::on_response),
        )
        .with_state(state)
}

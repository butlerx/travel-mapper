use crate::{
    db,
    server::{
        AppState,
        components::{CarrierIcon, NavBar, Shell},
        extractors::AuthUser,
        routes::JourneyResponse,
    },
};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use leptos::prelude::*;
use std::collections::HashMap;

pub async fn page(State(state): State<AppState>, auth: AuthUser) -> Response {
    let all_hops = db::hops::GetAll {
        user_id: auth.user_id,
        travel_type_filter: None,
    }
    .execute(&state.db)
    .await
    .unwrap_or_default();

    let today = chrono::Utc::now()
        .date_naive()
        .format("%Y-%m-%d")
        .to_string();

    // Journeys departing today or later, soonest first.
    let mut journeys: Vec<JourneyResponse> = all_hops
        .into_iter()
        .filter(|hop| hop.start_date.as_str() >= today.as_str())
        .map(JourneyResponse::from)
        .collect();
    journeys.sort_by(|a, b| a.start_date.cmp(&b.start_date).then(a.id.cmp(&b.id)));

    apply_enrichments(&state, &mut journeys).await;

    let html = view! { <UpcomingPage today=today journeys=journeys /> };
    (StatusCode::OK, axum::response::Html(html.to_html())).into_response()
}

/// Attach the latest status enrichment (status, delay, gate/terminal/platform)
/// to each journey in a single batch query.
async fn apply_enrichments(state: &AppState, journeys: &mut [JourneyResponse]) {
    let hop_ids: Vec<i64> = journeys.iter().map(|j| j.id).collect();
    let Ok(enrichments) = (db::status_enrichments::GetByHopIds { hop_ids })
        .execute(&state.db)
        .await
    else {
        return;
    };
    let by_hop: HashMap<i64, db::status_enrichments::Row> =
        enrichments.into_iter().map(|e| (e.hop_id, e)).collect();
    for journey in journeys.iter_mut() {
        if let Some(enrichment) = by_hop.get(&journey.id) {
            journey.apply_enrichment(enrichment);
        }
    }
}

/// Relative countdown label for an upcoming departure date.
fn countdown(today: &str, start_date: &str) -> String {
    let parse =
        |s: &str| chrono::NaiveDate::parse_from_str(s.get(..10).unwrap_or(s), "%Y-%m-%d").ok();
    match (parse(today), parse(start_date)) {
        (Some(t), Some(d)) => match (d - t).num_days() {
            n if n <= 0 => "Today".to_owned(),
            1 => "Tomorrow".to_owned(),
            n => format!("In {n} days"),
        },
        _ => String::new(),
    }
}

fn status_badge(journey: &JourneyResponse) -> Option<impl IntoView + use<>> {
    let status = journey.status.clone().filter(|s| !s.is_empty())?;
    let css = format!(
        "status-badge status-{}",
        status.to_lowercase().replace(' ', "-")
    );
    let label = match journey.delay_minutes {
        Some(mins) if mins > 0 => format!("{status} (+{mins}m)"),
        Some(mins) if mins < 0 => format!("{status} ({mins}m)"),
        _ => status,
    };
    Some(view! { <span class=css>{label}</span> })
}

fn journey_card(today: &str, journey: &JourneyResponse) -> impl IntoView + use<> {
    let cd = countdown(today, &journey.start_date);
    let is_today = cd == "Today";
    let id = journey.id;

    let chips: Vec<String> = [
        journey.dep_gate.as_ref().map(|g| format!("Gate {g}")),
        journey
            .dep_terminal
            .as_ref()
            .map(|t| format!("Terminal {t}")),
        journey
            .dep_platform
            .as_ref()
            .map(|p| format!("Platform {p}")),
    ]
    .into_iter()
    .flatten()
    .collect();
    let chip_views: Vec<_> = chips
        .into_iter()
        .map(|c| view! { <span class="upcoming-chip">{c}</span> })
        .collect();

    view! {
        <a
            href=format!("/journeys/{id}")
            class="upcoming-card"
            class:upcoming-card--today=is_today
            data-journey-id=id.to_string()
            data-live=is_today.then_some("1")
        >
            <div class="upcoming-countdown">{cd}</div>
            <div class="upcoming-body">
                <div class="upcoming-route">
                    <CarrierIcon
                        carrier=journey.carrier.clone().unwrap_or_default()
                        travel_type=journey.travel_type.as_str().to_owned()
                        size=22
                    />
                    <span class="upcoming-origin">{journey.origin_name.clone()}</span>
                    <span class="upcoming-arrow">"\u{2192}"</span>
                    <span class="upcoming-dest">{journey.dest_name.clone()}</span>
                </div>
                <div class="upcoming-meta">
                    <span class="upcoming-type">
                        {journey.travel_type.emoji()}" "{journey.travel_type.to_string()}
                    </span>
                    <span class="upcoming-date">{journey.start_date.clone()}</span>
                    <span class="upcoming-live-status" data-live-status>{status_badge(journey)}</span>
                    {chip_views}
                </div>
            </div>
        </a>
    }
}

#[component]
fn UpcomingPage(today: String, journeys: Vec<JourneyResponse>) -> impl IntoView {
    let has_journeys = !journeys.is_empty();
    let cards: Vec<_> = journeys
        .iter()
        .map(|journey| journey_card(&today, journey))
        .collect();

    view! {
        <Shell title="Upcoming".to_owned()>
            <NavBar current="upcoming" />
            <main class="container">
                <h1 class="upcoming-title">"Upcoming Journeys"</h1>
                {if has_journeys {
                    view! { <div class="upcoming-list">{cards}</div> }.into_any()
                } else {
                    view! {
                        <section class="card">
                            <div class="empty-state">
                                <div class="empty-state-icon">{"\u{1F9F3}"}</div>
                                <p>"No upcoming journeys. Add one or sync TripIt to see what\u{2019}s next."</p>
                            </div>
                        </section>
                    }.into_any()
                }}
                <script type="module" src="/static/upcoming-poll.js"></script>
            </main>
        </Shell>
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        db,
        db::hops::TravelType,
        server::{create_router, test_helpers::*},
    };
    use axum::{
        body::Body,
        http::{Request, StatusCode, header},
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn upcoming_requires_auth() {
        let pool = test_pool().await;
        let app = create_router(test_app_state(pool));
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/upcoming")
                    .header(header::ACCEPT, "text/html")
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn upcoming_lists_future_excludes_past() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "alice").await;
        let user = db::users::GetByUsername { username: "alice" }
            .execute(&pool)
            .await
            .expect("lookup failed")
            .expect("missing user");

        db::hops::Create {
            trip_id: "past",
            user_id: user.id,
            hops: &[sample_hop(
                TravelType::Air,
                "LHR",
                "JFK",
                "2020-01-01",
                "2020-01-01",
            )],
        }
        .execute(&pool)
        .await
        .expect("insert past");

        db::hops::Create {
            trip_id: "future",
            user_id: user.id,
            hops: &[sample_hop(
                TravelType::Air,
                "SFO",
                "NRT",
                "2099-06-01",
                "2099-06-02",
            )],
        }
        .execute(&pool)
        .await
        .expect("insert future");

        let app = create_router(test_app_state(pool));
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/upcoming")
                    .header(header::COOKIE, cookie)
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_text(response).await;
        assert!(body.contains("Upcoming Journeys"));
        assert!(body.contains("NRT"), "future journey should be listed");
        assert!(!body.contains("JFK"), "past journey should be excluded");
        assert!(body.contains("/static/upcoming-poll.js"));
    }

    #[tokio::test]
    async fn upcoming_empty_state_renders() {
        let pool = test_pool().await;
        let cookie = auth_cookie_for_user(&pool, "bob").await;
        let app = create_router(test_app_state(pool));
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/upcoming")
                    .header(header::COOKIE, cookie)
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_text(response).await;
        assert!(body.contains("No upcoming journeys"));
    }
}

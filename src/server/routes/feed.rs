use crate::{db, server::AppState};
use axum::{
    extract::{Path, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use icalendar::{Calendar, Component, Event, EventLike};

pub async fn handler(State(state): State<AppState>, Path(token_hash): Path<String>) -> Response {
    let token_hash = token_hash.strip_suffix(".ics").unwrap_or(&token_hash);

    let user_id = match (db::feed_tokens::GetUserIdByHash { token_hash })
        .execute(&state.db)
        .await
    {
        Ok(Some(id)) => id,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(err) => {
            tracing::error!(error = %err, "feed token lookup failed");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let hops = match (db::hops::GetAll {
        user_id,
        travel_type_filter: None,
    })
    .execute(&state.db)
    .await
    {
        Ok(rows) => rows,
        Err(err) => {
            tracing::error!(error = %err, "failed to fetch hops for feed");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let mut calendar = Calendar::new();
    calendar.name("Travel Calendar");
    calendar.append_property(icalendar::Property::new("PRODID", "-//TravelMapper//EN"));

    for hop in &hops {
        let summary = build_summary(hop);
        let location = format!("{} → {}", hop.origin_name, hop.dest_name);

        let mut event = Event::new();
        event.uid(&format!("hop-{}@travel-mapper", hop.id));
        event.summary(&summary);
        event.location(&location);

        if let Some(start) = parse_datetime(&hop.start_date) {
            event.starts(start);
        }
        if let Some(end) = parse_datetime(&hop.end_date) {
            event.ends(end);
        }

        let mut description_parts = Vec::new();
        description_parts.push(format!("Type: {}", hop.travel_type));
        if let Some(carrier) = hop.carrier() {
            description_parts.push(format!("Carrier: {carrier}"));
        }
        if !description_parts.is_empty() {
            event.description(&description_parts.join("\n"));
        }

        calendar.push(event.done());
    }

    let body = calendar.to_string();
    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "text/calendar; charset=utf-8"),
            (header::CACHE_CONTROL, "public, max-age=300"),
        ],
        body,
    )
        .into_response()
}

fn build_summary(hop: &db::hops::Row) -> String {
    let emoji = hop.travel_type.emoji();
    let route = format!("{} → {}", hop.origin_name, hop.dest_name);
    match hop.carrier() {
        Some(carrier) => format!("{emoji} {carrier}: {route}"),
        None => format!("{emoji} {route}"),
    }
}

fn parse_datetime(s: &str) -> Option<icalendar::DatePerhapsTime> {
    let trimmed = s.trim();
    if trimmed.len() >= 19 {
        let naive =
            chrono::NaiveDateTime::parse_from_str(&trimmed[..19], "%Y-%m-%d %H:%M:%S").ok()?;
        Some(icalendar::DatePerhapsTime::DateTime(
            icalendar::CalendarDateTime::Floating(naive),
        ))
    } else if trimmed.len() >= 10 {
        let date = chrono::NaiveDate::parse_from_str(&trimmed[..10], "%Y-%m-%d").ok()?;
        Some(icalendar::DatePerhapsTime::Date(date))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        db::{
            self,
            hops::{Create, TravelType},
        },
        server::{create_router, test_helpers::*},
    };
    use axum::{
        body::Body,
        http::{Request, StatusCode, header},
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn feed_returns_ics_for_valid_token() {
        let pool = test_pool().await;
        let user_id = db::tests::test_user(&pool, "alice").await;

        Create {
            trip_id: "trip-1",
            user_id,
            hops: &[sample_hop(
                TravelType::Air,
                "Dublin",
                "London",
                "2024-06-01 08:00:00",
                "2024-06-01 09:30:00",
            )],
        }
        .execute(&pool)
        .await
        .expect("insert hops failed");

        let token_hash = "feed_hash_abc123";
        db::feed_tokens::Create {
            user_id,
            token_hash,
            label: "test",
        }
        .execute(&pool)
        .await
        .expect("create feed token failed");

        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/feed/{token_hash}.ics"))
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);
        let content_type = response
            .headers()
            .get(header::CONTENT_TYPE)
            .expect("missing content-type")
            .to_str()
            .expect("invalid content-type");
        assert!(content_type.contains("text/calendar"));

        let cache_control = response
            .headers()
            .get(header::CACHE_CONTROL)
            .expect("missing cache-control")
            .to_str()
            .expect("invalid cache-control");
        assert!(cache_control.contains("max-age="));

        let body = body_text(response).await;
        assert!(body.contains("BEGIN:VCALENDAR"));
        assert!(body.contains("BEGIN:VEVENT"));
        assert!(body.contains("Dublin"));
        assert!(body.contains("London"));
        assert!(body.contains("END:VCALENDAR"));
    }

    #[tokio::test]
    async fn feed_returns_404_for_invalid_token() {
        let pool = test_pool().await;
        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/feed/nonexistent-token.ics")
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn feed_returns_empty_calendar_for_user_with_no_hops() {
        let pool = test_pool().await;
        let user_id = db::tests::test_user(&pool, "bob").await;

        let token_hash = "bob-feed-hash";
        db::feed_tokens::Create {
            user_id,
            token_hash,
            label: "empty",
        }
        .execute(&pool)
        .await
        .expect("create feed token failed");

        let app = create_router(test_app_state(pool));

        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/feed/{token_hash}.ics"))
                    .body(Body::empty())
                    .expect("failed to build request"),
            )
            .await
            .expect("router request failed");

        assert_eq!(response.status(), StatusCode::OK);
        let body = body_text(response).await;
        assert!(body.contains("BEGIN:VCALENDAR"));
        assert!(body.contains("END:VCALENDAR"));
        assert!(!body.contains("BEGIN:VEVENT"));
    }
}

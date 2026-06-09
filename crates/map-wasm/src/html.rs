use wasm_bindgen::prelude::*;

use crate::geo::escape_html;
use crate::lookup::carrier_icon_url;

#[wasm_bindgen]
pub fn countdown_text(date_str: &str) -> String {
    let now = js_sys::Date::new_0();
    let today_ms = js_sys::Date::new_with_year_month_day(
        now.get_full_year(),
        now.get_month() as i32,
        now.get_date() as i32,
    )
    .get_time();
    let target = js_sys::Date::new(&JsValue::from_str(&format!("{date_str}T00:00:00")));
    let diff_days = ((target.get_time() - today_ms) / 86_400_000.0).ceil() as i64;
    match diff_days {
        0 => "Today".to_owned(),
        1 => "Tomorrow".to_owned(),
        d => format!("In {d} days"),
    }
}

#[wasm_bindgen]
pub fn status_badge_html(status: &str, delay_minutes: JsValue) -> String {
    if status.is_empty() {
        return String::new();
    }
    let css_class = format!(
        "status-badge status-{}",
        escape_html(status).to_lowercase().replace(' ', "-")
    );
    let delay: Option<i32> = delay_minutes.as_f64().map(|d| d as i32);
    let label = match delay {
        Some(d) if d > 0 => format!("{} (+{d}m)", escape_html(status)),
        Some(d) if d < 0 => format!("{} ({d}m)", escape_html(status)),
        _ => escape_html(status),
    };
    format!(r#"<span class="{css_class}">{label}</span>"#)
}

#[wasm_bindgen]
pub fn verification_badge_html(route_verified: JsValue) -> String {
    if route_verified.is_null() || route_verified.is_undefined() {
        return String::new();
    }
    if route_verified.as_bool().unwrap_or(false) {
        r#"<span class="status-badge status-connected">✓ Verified</span>"#.to_owned()
    } else {
        r#"<span class="status-badge status-disconnected">Unverified</span>"#.to_owned()
    }
}

#[wasm_bindgen]
pub fn platform_badge_html(travel_type: &str, dep_platform: &str, arr_platform: &str) -> String {
    if travel_type != "rail" {
        return String::new();
    }
    let mut parts = Vec::new();
    if !dep_platform.is_empty() {
        parts.push(format!("Pl. {}", escape_html(dep_platform)));
    }
    if !arr_platform.is_empty() {
        parts.push(format!("→ Pl. {}", escape_html(arr_platform)));
    }
    if parts.is_empty() {
        return String::new();
    }
    format!(r#"<span class="platform-badge">{}</span>"#, parts.join(" "))
}

fn emoji_for(travel_type: &str) -> &'static str {
    match travel_type {
        "air" => "\u{2708}\u{FE0F}",
        "rail" => "\u{1F686}",
        "boat" => "\u{1F6A2}",
        "transport" => "\u{1F697}",
        _ => "",
    }
}

#[wasm_bindgen]
pub fn journey_card_html(journey_json: &str) -> String {
    let j: serde_json::Value = match serde_json::from_str(journey_json) {
        Ok(v) => v,
        Err(_) => return String::new(),
    };
    let id = j.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
    let tt = j.get("travel_type").and_then(|v| v.as_str()).unwrap_or("");
    let emoji = emoji_for(tt);
    let type_label = if tt.is_empty() {
        String::new()
    } else {
        let mut c = tt.chars();
        c.next().map_or(String::new(), |first| {
            first.to_uppercase().to_string() + c.as_str()
        })
    };
    let origin = escape_html(j.get("origin_name").and_then(|v| v.as_str()).unwrap_or(""));
    let dest = escape_html(j.get("dest_name").and_then(|v| v.as_str()).unwrap_or(""));
    let start_date = escape_html(j.get("start_date").and_then(|v| v.as_str()).unwrap_or(""));
    let carrier = j.get("carrier").and_then(|v| v.as_str()).unwrap_or("");
    let icon = carrier_icon_url(carrier, tt, 20);

    let dist = match (
        j.get("origin_lat").and_then(|v| v.as_f64()),
        j.get("origin_lng").and_then(|v| v.as_f64()),
        j.get("dest_lat").and_then(|v| v.as_f64()),
        j.get("dest_lng").and_then(|v| v.as_f64()),
    ) {
        (Some(lat1), Some(lng1), Some(lat2), Some(lng2)) => {
            let km = crate::geo::haversine_km(lat1, lng1, lat2, lng2);
            if km < 1.0 {
                "<1 km".to_owned()
            } else {
                format!("{} km", km.round() as u64)
            }
        }
        _ => String::new(),
    };

    let status = j.get("status").and_then(|v| v.as_str()).unwrap_or("");
    let delay = j.get("delay_minutes").and_then(|v| v.as_f64());
    let status_html = if status.is_empty() {
        String::new()
    } else {
        let delay_js = delay.map_or(JsValue::NULL, JsValue::from_f64);
        status_badge_html(status, delay_js)
    };

    let verified = j.get("route_verified");
    let verified_html = match verified {
        Some(v) if v.is_boolean() => {
            let jv = if v.as_bool().unwrap_or(false) {
                JsValue::TRUE
            } else {
                JsValue::FALSE
            };
            verification_badge_html(jv)
        }
        _ => String::new(),
    };

    let dep_plat = j.get("dep_platform").and_then(|v| v.as_str()).unwrap_or("");
    let arr_plat = j.get("arr_platform").and_then(|v| v.as_str()).unwrap_or("");
    let plat_html = platform_badge_html(tt, dep_plat, arr_plat);

    let dist_html = if dist.is_empty() {
        String::new()
    } else {
        format!(r#"<span class="journey-distance">{dist}</span>"#)
    };

    format!(
        r#"<a href="/journeys/{id}" class="journey-card-link"><div class="journey-card"><div class="journey-route">{icon} <span class="journey-origin">{origin}</span><span class="journey-arrow">→</span><span class="journey-dest">{dest}</span></div><div class="journey-meta"><span class="journey-badge badge-{tt}">{emoji} {type_label}</span>{status_html}{verified_html}{plat_html}<span class="journey-date">{start_date}</span>{dist_html}</div></div></a>"#
    )
}

#[wasm_bindgen]
pub fn journey_sidebar_html(journeys_json: &str) -> String {
    let journeys: Vec<serde_json::Value> = match serde_json::from_str(journeys_json) {
        Ok(v) => v,
        Err(_) => return String::new(),
    };

    if journeys.is_empty() {
        return r#"<h3 class="journey-sidebar-heading">Journeys</h3><div class="journey-empty">No journeys match the current filters.</div>"#.to_owned();
    }

    let now = js_sys::Date::new_0();
    let today = format!(
        "{:04}-{:02}-{:02}",
        now.get_full_year(),
        now.get_month() + 1,
        now.get_date()
    );

    let mut upcoming: Vec<&serde_json::Value> = Vec::new();
    let mut past: Vec<&serde_json::Value> = Vec::new();
    for j in &journeys {
        let d = j.get("start_date").and_then(|v| v.as_str()).unwrap_or("");
        if d >= today.as_str() {
            upcoming.push(j);
        } else {
            past.push(j);
        }
    }
    upcoming.sort_by(|a, b| {
        let ad = a.get("start_date").and_then(|v| v.as_str()).unwrap_or("");
        let bd = b.get("start_date").and_then(|v| v.as_str()).unwrap_or("");
        ad.cmp(bd)
    });
    past.sort_by(|a, b| {
        let ad = a.get("start_date").and_then(|v| v.as_str()).unwrap_or("");
        let bd = b.get("start_date").and_then(|v| v.as_str()).unwrap_or("");
        bd.cmp(ad)
    });

    let mut html = String::new();
    if !upcoming.is_empty() {
        html.push_str(&format!(
            r#"<h3 class="journey-sidebar-heading journey-sidebar-heading--upcoming">Upcoming ({})</h3>"#,
            upcoming.len()
        ));
        for j in &upcoming {
            let j_str = serde_json::to_string(j).unwrap_or_default();
            let card = journey_card_html(&j_str);
            let date = j.get("start_date").and_then(|v| v.as_str()).unwrap_or("");
            let countdown = countdown_text(date);
            let card = card.replace(
                r#"class="journey-card">"#,
                &format!(r#"class="journey-card journey-card--upcoming"><span class="journey-countdown">{countdown}</span>"#),
            );
            html.push_str(&card);
        }
    }
    if !past.is_empty() {
        html.push_str(&format!(
            r#"<h3 class="journey-sidebar-heading">Past Journeys ({})</h3>"#,
            past.len()
        ));
        for j in &past {
            let j_str = serde_json::to_string(j).unwrap_or_default();
            html.push_str(&journey_card_html(&j_str));
        }
    }
    if upcoming.is_empty() && past.is_empty() {
        html.push_str(r#"<div class="journey-empty">No journeys match the current filters.</div>"#);
    }
    html
}

#[wasm_bindgen]
pub fn route_popup_html(route_key: &str, route_index_json: &str) -> String {
    let index: std::collections::HashMap<String, serde_json::Value> =
        match serde_json::from_str(route_index_json) {
            Ok(v) => v,
            Err(_) => return String::new(),
        };
    let r = match index.get(route_key) {
        Some(v) => v,
        None => return String::new(),
    };

    let origin = escape_html(r.get("origin_name").and_then(|v| v.as_str()).unwrap_or(""));
    let dest = escape_html(r.get("dest_name").and_then(|v| v.as_str()).unwrap_or(""));
    let freq = r.get("freq").and_then(|v| v.as_u64()).unwrap_or(0);
    let from = r.get("from").and_then(|v| v.as_array());
    let to = r.get("to").and_then(|v| v.as_array());
    let dist = match (from, to) {
        (Some(f), Some(t)) if f.len() >= 2 && t.len() >= 2 => {
            let km = crate::geo::haversine_km(
                f[1].as_f64().unwrap_or(0.0),
                f[0].as_f64().unwrap_or(0.0),
                t[1].as_f64().unwrap_or(0.0),
                t[0].as_f64().unwrap_or(0.0),
            );
            if km < 1.0 {
                "<1".to_owned()
            } else {
                format!("{}", km.round() as u64)
            }
        }
        _ => "?".to_owned(),
    };

    let hops = r.get("hops").and_then(|v| v.as_array());
    let mut sorted: Vec<&serde_json::Value> = hops.map_or(Vec::new(), |h| h.iter().collect());
    sorted.sort_by(|a, b| {
        let ad = a.get("start_date").and_then(|v| v.as_str()).unwrap_or("");
        let bd = b.get("start_date").and_then(|v| v.as_str()).unwrap_or("");
        bd.cmp(ad)
    });

    let shown = &sorted[..sorted.len().min(8)];
    let plural = if freq != 1 { "s" } else { "" };
    let mut html = format!(
        r#"<div class="journey-popup"><div class="journey-popup-header"><strong>{origin} ↔ {dest}</strong></div><div class="journey-popup-summary"><span>{freq} journey{plural}</span><span>📏 {dist} km</span></div><div class="journey-popup-list">"#
    );

    for j in shown {
        let tt = j.get("travel_type").and_then(|v| v.as_str()).unwrap_or("");
        let emoji = emoji_for(tt);
        let sd = escape_html(j.get("start_date").and_then(|v| v.as_str()).unwrap_or(""));
        let ed = escape_html(j.get("end_date").and_then(|v| v.as_str()).unwrap_or(""));
        let date_str = if sd == ed {
            sd.clone()
        } else {
            format!("{sd} → {ed}")
        };
        let j_origin = escape_html(j.get("origin_name").and_then(|v| v.as_str()).unwrap_or(""));
        let direction = if j_origin == origin {
            format!("{origin} → {dest}")
        } else {
            format!("{dest} → {origin}")
        };
        let id = j.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
        html.push_str(&format!(
            r#"<a href="/journeys/{id}" class="journey-popup-item"><span class="journey-popup-item-emoji">{emoji}</span><span class="journey-popup-item-direction">{direction}</span><span class="journey-popup-item-date">{date_str}</span></a>"#
        ));
    }
    if sorted.len() > 8 {
        html.push_str(&format!(
            r#"<div class="journey-popup-more">+{} more</div>"#,
            sorted.len() - 8
        ));
    }
    html.push_str("</div></div>");
    html
}

#[wasm_bindgen]
pub fn city_popup_html(name: &str, count: u32, routes_json: &str) -> String {
    let routes: std::collections::HashMap<String, u32> =
        serde_json::from_str(routes_json).unwrap_or_default();
    let mut sorted: Vec<(&String, &u32)> = routes.iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(a.1));

    let route_list: String = sorted
        .iter()
        .take(5)
        .map(|(dest, cnt)| {
            format!(
                r#"<div class="airport-popup-route"><span class="airport-popup-dest">{}</span><span class="airport-popup-freq">{cnt}×</span></div>"#,
                escape_html(dest)
            )
        })
        .collect();

    let more = if sorted.len() > 5 {
        format!(
            r#"<div class="airport-popup-more">+{} more destinations</div>"#,
            sorted.len() - 5
        )
    } else {
        String::new()
    };

    let plural = if count != 1 { "s" } else { "" };
    let conn_plural = if sorted.len() != 1 { "s" } else { "" };
    let escaped_name = escape_html(name);
    format!(
        r#"<div class="airport-popup"><div class="airport-popup-header"><strong>{escaped_name}</strong></div><div class="airport-popup-stats"><span class="airport-popup-visits">{count} visit{plural}</span><span class="airport-popup-connections">{} connection{conn_plural}</span></div><div class="airport-popup-routes">{route_list}{more}</div></div>"#,
        sorted.len()
    )
}

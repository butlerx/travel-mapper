use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

#[derive(Deserialize, Serialize)]
struct Journey {
    id: i64,
    travel_type: Option<String>,
    origin_name: Option<String>,
    origin_lat: Option<f64>,
    origin_lng: Option<f64>,
    dest_name: Option<String>,
    dest_lat: Option<f64>,
    dest_lng: Option<f64>,
    start_date: Option<String>,
    end_date: Option<String>,
    carrier: Option<String>,
    status: Option<String>,
    delay_minutes: Option<i32>,
    dep_platform: Option<String>,
    arr_platform: Option<String>,
    route_verified: Option<bool>,
}

#[derive(Serialize)]
struct ArcFeature {
    r#type: &'static str,
    properties: ArcProperties,
    geometry: LineGeometry,
}

#[derive(Serialize)]
struct ArcProperties {
    key: String,
    color: String,
    width: f64,
    opacity: f64,
}

#[derive(Serialize)]
struct LineGeometry {
    r#type: &'static str,
    coordinates: Vec<[f64; 2]>,
}

#[derive(Serialize)]
struct CityFeature {
    r#type: &'static str,
    geometry: PointGeometry,
    properties: CityProperties,
}

#[derive(Serialize)]
struct PointGeometry {
    r#type: &'static str,
    coordinates: [f64; 2],
}

#[derive(Serialize)]
struct CityProperties {
    name: String,
    count: u32,
    routes: String,
}

#[derive(Serialize)]
struct RouteInfo {
    origin_name: String,
    dest_name: String,
    from: [f64; 2],
    to: [f64; 2],
    freq: usize,
    hops: Vec<serde_json::Value>,
}

#[derive(Serialize)]
struct DashboardData {
    arc_features: serde_json::Value,
    city_features: serde_json::Value,
    route_index: HashMap<String, RouteInfo>,
    journey_count: usize,
}

struct CityAccum {
    name: String,
    lat: f64,
    lng: f64,
    count: u32,
    routes: HashMap<String, u32>,
}

fn arc_coords(from: [f64; 2], to: [f64; 2], n: u32) -> Vec<[f64; 2]> {
    let lat1 = from[1].to_radians();
    let lng1 = from[0].to_radians();
    let lat2 = to[1].to_radians();
    let mut lng2 = to[0].to_radians();

    let d_lng = lng2 - lng1;
    if d_lng > std::f64::consts::PI {
        lng2 -= 2.0 * std::f64::consts::PI;
    } else if d_lng < -std::f64::consts::PI {
        lng2 += 2.0 * std::f64::consts::PI;
    }

    let d = 2.0
        * (((lat1 - lat2) / 2.0).sin().powi(2)
            + lat1.cos() * lat2.cos() * ((lng1 - lng2) / 2.0).sin().powi(2))
        .sqrt()
        .asin();

    if d < 1e-10 {
        return vec![from, to];
    }

    let mut pts = Vec::with_capacity((n + 1) as usize);
    let mut prev = from[0];
    for i in 0..=n {
        let f = f64::from(i) / f64::from(n);
        let a = ((1.0 - f) * d).sin() / d.sin();
        let b = (f * d).sin() / d.sin();
        let x = a * lat1.cos() * lng1.cos() + b * lat2.cos() * lng2.cos();
        let y = a * lat1.cos() * lng1.sin() + b * lat2.cos() * lng2.sin();
        let z = a * lat1.sin() + b * lat2.sin();
        let lat = z.atan2((x * x + y * y).sqrt()).to_degrees();
        let mut lng = y.atan2(x).to_degrees();
        while lng - prev > 180.0 {
            lng -= 360.0;
        }
        while lng - prev < -180.0 {
            lng += 360.0;
        }
        prev = lng;
        pts.push([lng, lat]);
    }
    pts
}

const TYPE_COLORS: &[(&str, &str)] = &[
    ("air", "#56b4e9"),
    ("rail", "#e69f00"),
    ("boat", "#d55e00"),
    ("transport", "#009e73"),
];

fn type_color(tt: &str) -> &'static str {
    TYPE_COLORS
        .iter()
        .find(|(k, _)| *k == tt)
        .map_or("#6b7280", |(_, v)| v)
}

type RouteAccum = (String, [f64; 2], [f64; 2], Vec<serde_json::Value>);

#[wasm_bindgen]
pub fn compute_routes(journeys_json: &str) -> String {
    let journeys: Vec<Journey> = match serde_json::from_str(journeys_json) {
        Ok(v) => v,
        Err(_) => return "{}".to_owned(),
    };

    let mut routes: HashMap<String, RouteAccum> = HashMap::new();
    let mut cities: HashMap<String, CityAccum> = HashMap::new();
    let mut count = 0_usize;

    for j in &journeys {
        let (Some(o_lat), Some(o_lng), Some(d_lat), Some(d_lng)) =
            (j.origin_lat, j.origin_lng, j.dest_lat, j.dest_lng)
        else {
            continue;
        };
        if o_lat == 0.0 && o_lng == 0.0 {
            continue;
        }
        if d_lat == 0.0 && d_lng == 0.0 {
            continue;
        }
        count += 1;

        let o_name = j.origin_name.as_deref().unwrap_or("");
        let d_name = j.dest_name.as_deref().unwrap_or("");

        let key1 = format!("{o_name}|{o_lat}|{o_lng}\u{2192}{d_name}|{d_lat}|{d_lng}");
        let key2 = format!("{d_name}|{d_lat}|{d_lng}\u{2192}{o_name}|{o_lat}|{o_lng}");
        let key = if key1 < key2 { key1 } else { key2 };

        let raw = serde_json::to_value(j).unwrap_or_default();

        routes
            .entry(key)
            .and_modify(|r| r.3.push(raw.clone()))
            .or_insert_with(|| {
                (
                    format!("{o_name}\u{2194}{d_name}"),
                    [o_lng, o_lat],
                    [d_lng, d_lat],
                    vec![raw],
                )
            });

        let add_city = |cities: &mut HashMap<String, CityAccum>,
                        name: &str,
                        lat: f64,
                        lng: f64,
                        peer: &str| {
            let ck = format!("{name}|{lat}|{lng}");
            let c = cities.entry(ck).or_insert_with(|| CityAccum {
                name: name.to_owned(),
                lat,
                lng,
                count: 0,
                routes: HashMap::new(),
            });
            c.count += 1;
            *c.routes.entry(peer.to_owned()).or_insert(0) += 1;
        };

        add_city(&mut cities, o_name, o_lat, o_lng, d_name);
        add_city(&mut cities, d_name, d_lat, d_lng, o_name);
    }

    let arc_features: Vec<ArcFeature> = routes
        .iter()
        .map(|(key, (_, from, to, hops))| {
            let freq = hops.len();
            let mut type_counts: HashMap<&str, usize> = HashMap::new();
            for h in hops {
                let tt = h
                    .get("travel_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("air");
                *type_counts.entry(tt).or_insert(0) += 1;
            }
            let dominant = type_counts
                .into_iter()
                .max_by_key(|(_, c)| *c)
                .map_or("air", |(t, _)| t);
            let color = type_color(dominant).to_owned();
            let width = if freq >= 10 {
                3.0
            } else if freq >= 5 {
                2.5
            } else if freq >= 2 {
                2.0
            } else {
                1.5
            };
            let opacity = (0.5 + freq as f64 * 0.05).min(0.9);

            ArcFeature {
                r#type: "Feature",
                properties: ArcProperties {
                    key: key.clone(),
                    color,
                    width,
                    opacity,
                },
                geometry: LineGeometry {
                    r#type: "LineString",
                    coordinates: arc_coords(*from, *to, 60),
                },
            }
        })
        .collect();

    let city_features: Vec<CityFeature> = cities
        .values()
        .map(|c| CityFeature {
            r#type: "Feature",
            geometry: PointGeometry {
                r#type: "Point",
                coordinates: [c.lng, c.lat],
            },
            properties: CityProperties {
                name: c.name.clone(),
                count: c.count,
                routes: serde_json::to_string(&c.routes).unwrap_or_default(),
            },
        })
        .collect();

    let route_index: HashMap<String, RouteInfo> = routes
        .into_iter()
        .map(|(key, (label, from, to, hops))| {
            let parts: Vec<&str> = label.splitn(2, '\u{2194}').collect();
            let origin_name = parts.first().unwrap_or(&"").to_string();
            let dest_name = parts.get(1).unwrap_or(&"").to_string();
            (
                key,
                RouteInfo {
                    origin_name,
                    dest_name,
                    from,
                    to,
                    freq: hops.len(),
                    hops,
                },
            )
        })
        .collect();

    let data = DashboardData {
        arc_features: serde_json::json!({
            "type": "FeatureCollection",
            "features": arc_features,
        }),
        city_features: serde_json::json!({
            "type": "FeatureCollection",
            "features": city_features,
        }),
        route_index,
        journey_count: count,
    };

    serde_json::to_string(&data).unwrap_or_else(|_| "{}".to_owned())
}

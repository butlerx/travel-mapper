use crate::auth::{CryptoError, decrypt_token, parse_encryption_key};
use crate::db;
use crate::distance::haversine_km;
use crate::geocode::Geocoder;
use crate::integrations::tripit::{FetchError, TripItApi, TripItAuth, TripItClient};
use clap::Args as ClapArgs;
use serde_json::Value;

const DISTANCE_THRESHOLD_KM: f64 = 50.0;

fn ensure_list(val: &Value) -> Vec<&Value> {
    match val {
        Value::Array(arr) => arr.iter().collect(),
        Value::Null => vec![],
        other => vec![other],
    }
}

fn extract_coords(addr: &Value) -> (f64, f64) {
    if addr.is_null() || !addr.is_object() {
        return (0.0, 0.0);
    }
    let lat = coerce_float(&addr["latitude"]).unwrap_or(0.0);
    let lng = coerce_float(&addr["longitude"]).unwrap_or(0.0);
    (lat, lng)
}

fn coerce_float(val: &Value) -> Option<f64> {
    match val {
        Value::Number(n) => n.as_f64(),
        Value::String(s) if !s.is_empty() => s.parse().ok(),
        _ => None,
    }
}

#[derive(ClapArgs)]
pub struct Args {
    #[arg(long, env = "TRIPIT_CONSUMER_KEY")]
    consumer_key: String,

    #[arg(long, env = "TRIPIT_CONSUMER_SECRET")]
    consumer_secret: String,

    #[arg(long, env = "ENCRYPTION_KEY")]
    encryption_key: String,

    #[arg(long, env = "DATABASE_URL", default_value = "sqlite:travel.db")]
    database_url: String,

    #[arg(long)]
    apply: bool,

    #[arg(long)]
    username: String,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Crypto(#[from] CryptoError),

    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("TripIt API error: {0}")]
    Fetch(#[from] FetchError),

    #[error("user {0:?} not found")]
    UserNotFound(String),

    #[error("no TripIt credentials stored for user {0:?}")]
    NoCredentials(String),
}

struct AddressFix {
    trip_name: String,
    object_type: String,
    object_id: String,
    segment_index: usize,
    station_name: String,
    old_lat: f64,
    old_lng: f64,
    new_lat: f64,
    new_lng: f64,
    distance_km: f64,
    full_object: Value,
}

/// # Errors
///
/// Returns an error if database access, credential decryption, or `TripIt` API calls fail.
pub async fn run(args: Args) -> Result<(), Error> {
    let encryption_key = parse_encryption_key(&args.encryption_key)?;
    let pool = crate::db::create_pool(&args.database_url).await?;

    let user = (db::users::GetByUsername {
        username: &args.username,
    })
    .execute(&pool)
    .await?
    .ok_or_else(|| Error::UserNotFound(args.username.clone()))?;

    let creds = (db::credentials::Get { user_id: user.id })
        .execute(&pool)
        .await?
        .ok_or_else(|| Error::NoCredentials(args.username.clone()))?;

    let access_token = decrypt_token(&creds.access_token_enc, &creds.nonce_token, &encryption_key)?;
    let access_token_secret = decrypt_token(
        &creds.access_token_secret_enc,
        &creds.nonce_secret,
        &encryption_key,
    )?;

    let auth = TripItAuth::new(
        args.consumer_key,
        args.consumer_secret,
        access_token,
        access_token_secret,
    );
    let client = TripItClient::new(auth);
    let geocoder = Geocoder::new(pool);

    let fixes = find_incorrect_addresses(&client, &geocoder).await?;

    if fixes.is_empty() {
        println!("No address issues found.");
        return Ok(());
    }

    println!("Found {} address issues:\n", fixes.len());
    for fix in &fixes {
        println!(
            "  [{trip}] {obj_type} (id={obj_id}) segment {seg}: \"{station}\"",
            trip = fix.trip_name,
            obj_type = fix.object_type,
            obj_id = fix.object_id,
            seg = fix.segment_index,
            station = fix.station_name,
        );
        println!("    TripIt:   ({:.4}, {:.4})", fix.old_lat, fix.old_lng);
        println!(
            "    Geocoded: ({:.4}, {:.4})  [{:.1} km off]",
            fix.new_lat, fix.new_lng, fix.distance_km,
        );
        println!();
    }

    if !args.apply {
        println!("Run with --apply to push corrections to TripIt.");
        return Ok(());
    }

    println!("Applying fixes...");
    for fix in &fixes {
        match client
            .replace_object(&fix.object_type, &fix.object_id, &fix.full_object)
            .await
        {
            Ok(_) => println!(
                "  ✓ Fixed {obj_type} {obj_id} segment {seg} ({station})",
                obj_type = fix.object_type,
                obj_id = fix.object_id,
                seg = fix.segment_index,
                station = fix.station_name,
            ),
            Err(e) => println!(
                "  ✗ Failed {obj_type} {obj_id}: {e}",
                obj_type = fix.object_type,
                obj_id = fix.object_id,
            ),
        }
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }

    println!("\nDone. Applied {} fixes.", fixes.len());
    Ok(())
}

async fn find_incorrect_addresses(
    api: &dyn TripItApi,
    geocoder: &Geocoder,
) -> Result<Vec<AddressFix>, FetchError> {
    let trips = fetch_all_trips(api).await?;
    let mut fixes = Vec::new();

    for (trip_id, trip_name) in &trips {
        let data = api.get_trip_objects(trip_id).await?;

        for (obj_type, addr_key) in [
            ("RailObject", "StationAddress"),
            ("TransportObject", "LocationAddress"),
            ("CruiseObject", "LocationAddress"),
        ] {
            for obj in ensure_list(&data[obj_type]) {
                let object_id = obj
                    .get("id")
                    .and_then(|v| {
                        v.as_str()
                            .filter(|s| !s.is_empty())
                            .map(ToString::to_string)
                            .or_else(|| v.as_u64().map(|n| n.to_string()))
                    })
                    .unwrap_or_default();

                if object_id.is_empty() {
                    continue;
                }

                let segments = if obj.get("Segment").is_some() {
                    ensure_list(&obj["Segment"])
                } else {
                    vec![obj]
                };

                let mut modified_object = obj.clone();

                for (seg_idx, seg) in segments.iter().enumerate() {
                    let segment_fixes = check_segment(
                        geocoder, seg, seg_idx, trip_name, &object_id, obj_type, addr_key,
                    )
                    .await;

                    for (station, endpoint, new_lat, new_lng, old_lat, old_lng, dist) in
                        &segment_fixes
                    {
                        apply_coord_fix(
                            &mut modified_object,
                            seg_idx,
                            endpoint,
                            addr_key,
                            *new_lat,
                            *new_lng,
                        );

                        fixes.push(AddressFix {
                            trip_name: trip_name.clone(),
                            object_type: obj_type.trim_end_matches("Object").to_ascii_lowercase(),
                            object_id: object_id.clone(),
                            segment_index: seg_idx,
                            station_name: station.clone(),
                            old_lat: *old_lat,
                            old_lng: *old_lng,
                            new_lat: *new_lat,
                            new_lng: *new_lng,
                            distance_km: *dist,
                            full_object: modified_object.clone(),
                        });
                    }
                }
            }
        }
    }

    Ok(fixes)
}

#[allow(clippy::type_complexity)]
async fn check_segment(
    geocoder: &Geocoder,
    seg: &Value,
    seg_idx: usize,
    trip_name: &str,
    object_id: &str,
    obj_type: &str,
    addr_key: &str,
) -> Vec<(String, String, f64, f64, f64, f64, f64)> {
    let _ = (seg_idx, trip_name, object_id, obj_type);
    let mut results = Vec::new();

    for (endpoint, name_key) in [("Start", "start_station_name"), ("End", "end_station_name")] {
        let full_addr_key = format!("{endpoint}{addr_key}");
        let addr = &seg[&full_addr_key];

        let station_name = seg
            .get(name_key)
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .or_else(|| {
                let alt_key = name_key.replace("station", "location");
                seg.get(&alt_key)
                    .and_then(|v| v.as_str())
                    .filter(|s| !s.is_empty())
            })
            .unwrap_or_default();

        if station_name.is_empty() {
            continue;
        }

        let (existing_lat, existing_lng) = extract_coords(addr);

        let country = addr
            .get("country")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty());
        let city = addr
            .get("city")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty());

        if existing_lat == 0.0 && existing_lng == 0.0 {
            let qualified_query = build_qualified_query(station_name, city, country);
            let resolved =
                resolve_station(geocoder, station_name, qualified_query.as_deref()).await;
            if let Some((geo_lat, geo_lng)) = resolved {
                results.push((
                    station_name.to_string(),
                    endpoint.to_string(),
                    geo_lat,
                    geo_lng,
                    existing_lat,
                    existing_lng,
                    0.0,
                ));
            }
            continue;
        }

        if let Some(co) = country {
            let country_center = geocoder.geocode_with_fallbacks(co, None).await;
            if let Some((cc_lat, cc_lng)) = country_center {
                let dist_to_country = haversine_km(existing_lat, existing_lng, cc_lat, cc_lng);
                if dist_to_country < 1000.0 {
                    continue;
                }
            }
        }

        let qualified_query = build_qualified_query(station_name, city, country);
        let resolved = resolve_station(geocoder, station_name, qualified_query.as_deref()).await;
        let Some((geo_lat, geo_lng)) = resolved else {
            continue;
        };

        let distance = haversine_km(existing_lat, existing_lng, geo_lat, geo_lng);
        if distance > DISTANCE_THRESHOLD_KM {
            results.push((
                station_name.to_string(),
                endpoint.to_string(),
                geo_lat,
                geo_lng,
                existing_lat,
                existing_lng,
                distance,
            ));
        }
    }

    results
}

async fn resolve_station(
    geocoder: &Geocoder,
    station_name: &str,
    qualified_query: Option<&str>,
) -> Option<(f64, f64)> {
    if let Some(q) = qualified_query {
        let r = geocoder.geocode_with_fallbacks(q, None).await;
        if r.is_some() {
            return r;
        }
    }
    geocoder
        .geocode_with_fallbacks(station_name, qualified_query)
        .await
}

fn build_qualified_query(
    station_name: &str,
    city: Option<&str>,
    country: Option<&str>,
) -> Option<String> {
    match (city, country) {
        (Some(c), Some(co)) => Some(format!("{station_name}, {c}, {co}")),
        (None, Some(co)) => Some(format!("{station_name}, {co}")),
        (Some(c), None) => Some(format!("{station_name}, {c}")),
        (None, None) => None,
    }
}

fn apply_coord_fix(
    obj: &mut Value,
    seg_idx: usize,
    endpoint: &str,
    addr_key: &str,
    lat: f64,
    lng: f64,
) {
    let full_key = format!("{endpoint}{addr_key}");

    let target = if let Some(segments) = obj.get_mut("Segment") {
        match segments {
            Value::Array(arr) => arr.get_mut(seg_idx),
            other => Some(other),
        }
    } else {
        Some(obj)
    };

    let Some(seg) = target else { return };

    let addr = seg.as_object_mut().and_then(|m| {
        m.entry(&full_key)
            .or_insert_with(|| Value::Object(serde_json::Map::new()))
            .as_object_mut()
    });

    if let Some(addr_obj) = addr {
        addr_obj.insert("latitude".to_string(), Value::String(format!("{lat:.6}")));
        addr_obj.insert("longitude".to_string(), Value::String(format!("{lng:.6}")));
    }
}

async fn fetch_all_trips(api: &dyn TripItApi) -> Result<Vec<(String, String)>, FetchError> {
    let mut all_trips = Vec::new();

    for past in [true, false] {
        let mut page = 1u64;
        loop {
            let data = api.list_trips(past, page, 25).await?;
            let batch = ensure_list(&data["Trip"]);
            for trip in &batch {
                let id = trip
                    .get("id")
                    .and_then(|v| {
                        v.as_str()
                            .map(std::string::ToString::to_string)
                            .or_else(|| v.as_u64().map(|n| n.to_string()))
                    })
                    .unwrap_or_default();
                let name = trip
                    .get("display_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown")
                    .to_string();
                if !id.is_empty() {
                    all_trips.push((id, name));
                }
            }

            let max_page = data
                .get("max_page")
                .and_then(|v| v.as_str().or_else(|| v.as_u64().map(|_| "")))
                .and_then(|s| {
                    if s.is_empty() {
                        data.get("max_page").and_then(serde_json::Value::as_u64)
                    } else {
                        s.parse().ok()
                    }
                })
                .unwrap_or(1);

            if page >= max_page {
                break;
            }
            page += 1;
        }
    }

    all_trips.dedup_by(|a, b| a.0 == b.0);
    Ok(all_trips)
}

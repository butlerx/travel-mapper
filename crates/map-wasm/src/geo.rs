use js_sys::Date;
use wasm_bindgen::prelude::*;

#[must_use]
#[wasm_bindgen]
pub fn haversine_km(lat1: f64, lng1: f64, lat2: f64, lng2: f64) -> f64 {
    let r = 6_371.0_f64;
    let d_lat = (lat2 - lat1).to_radians();
    let d_lng = (lng2 - lng1).to_radians();
    let a = (d_lat / 2.0).sin().powi(2)
        + lat1.to_radians().cos() * lat2.to_radians().cos() * (d_lng / 2.0).sin().powi(2);
    r * 2.0 * a.sqrt().atan2((1.0 - a).sqrt())
}

#[wasm_bindgen]
pub fn arc_points(
    from_lng: f64,
    from_lat: f64,
    to_lng: f64,
    to_lat: f64,
    num_points: u32,
) -> JsValue {
    let lat1 = from_lat.to_radians();
    let lng1 = from_lng.to_radians();
    let lat2 = to_lat.to_radians();
    let mut lng2 = to_lng.to_radians();

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
        let arr = js_sys::Array::new();
        let p1 = js_sys::Array::of2(&JsValue::from_f64(from_lng), &JsValue::from_f64(from_lat));
        let p2 = js_sys::Array::of2(&JsValue::from_f64(to_lng), &JsValue::from_f64(to_lat));
        arr.push(&p1);
        arr.push(&p2);
        return arr.into();
    }

    let arr = js_sys::Array::new_with_length(num_points + 1);
    let mut prev_lng = from_lng;

    for i in 0..=num_points {
        let f = f64::from(i) / f64::from(num_points);
        let a_coeff = ((1.0 - f) * d).sin() / d.sin();
        let b_coeff = (f * d).sin() / d.sin();
        let x = a_coeff * lat1.cos() * lng1.cos() + b_coeff * lat2.cos() * lng2.cos();
        let y = a_coeff * lat1.cos() * lng1.sin() + b_coeff * lat2.cos() * lng2.sin();
        let z = a_coeff * lat1.sin() + b_coeff * lat2.sin();
        let lat = z.atan2((x * x + y * y).sqrt()).to_degrees();
        let mut lng = y.atan2(x).to_degrees();

        while lng - prev_lng > 180.0 {
            lng -= 360.0;
        }
        while lng - prev_lng < -180.0 {
            lng += 360.0;
        }
        prev_lng = lng;

        let pt = js_sys::Array::of2(&JsValue::from_f64(lng), &JsValue::from_f64(lat));
        arr.set(i, pt.into());
    }

    arr.into()
}

#[wasm_bindgen]
pub fn night_polygon() -> String {
    let now = Date::new_0();
    let year_start = Date::new_with_year_month(now.get_utc_full_year(), 0);
    let day_of_year = ((now.get_time() - year_start.get_time()) / 86_400_000.0).floor();

    let decl_rad = (-23.44_f64).to_radians()
        * ((2.0 * std::f64::consts::PI * (day_of_year + 10.0)) / 365.0).cos();

    let utc_h = f64::from(now.get_utc_hours())
        + f64::from(now.get_utc_minutes()) / 60.0
        + f64::from(now.get_utc_seconds()) / 3600.0;
    let solar_lng = -((utc_h - 12.0) * 15.0);

    let mut ring = Vec::with_capacity(724);
    let mut lng = -180.0_f64;
    while lng <= 180.0 {
        let ha = (lng - solar_lng).to_radians();
        let lat = (-ha.cos() / decl_rad.tan()).atan().to_degrees();
        ring.push(format!("[{lng},{lat}]"));
        lng += 0.5;
    }

    let night_pole = if decl_rad >= 0.0 { -90 } else { 90 };
    ring.push(format!("[180,{night_pole}]"));
    ring.push(format!("[-180,{night_pole}]"));

    let coords = ring.join(",");
    format!(
        r#"{{"type":"Feature","geometry":{{"type":"Polygon","coordinates":[[{coords}]]}},"properties":{{}}}}"#
    )
}

#[must_use]
#[wasm_bindgen]
pub fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

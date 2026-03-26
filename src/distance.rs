#[must_use]
pub fn haversine_km(lat1: f64, lng1: f64, lat2: f64, lng2: f64) -> f64 {
    let r = 6_371.0_f64;
    let d_lat = (lat2 - lat1).to_radians();
    let d_lng = (lng2 - lng1).to_radians();
    let a = (d_lat / 2.0).sin().powi(2)
        + lat1.to_radians().cos() * lat2.to_radians().cos() * (d_lng / 2.0).sin().powi(2);
    r * 2.0 * a.sqrt().atan2((1.0 - a).sqrt())
}

#[must_use]
pub fn haversine_miles(lat1: f64, lng1: f64, lat2: f64, lng2: f64) -> f64 {
    haversine_km(lat1, lng1, lat2, lng2) * 0.621_371
}

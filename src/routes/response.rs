use crate::models::TravelHop;
use axum::{
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResponseFormat {
    Json,
    Csv,
    Html,
}

pub fn negotiate_format(headers: &HeaderMap) -> ResponseFormat {
    let accept = headers
        .get(header::ACCEPT)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    for part in accept.split(',') {
        let media = part.split(';').next().unwrap_or("").trim();
        match media {
            "text/html" => return ResponseFormat::Html,
            "text/csv" => return ResponseFormat::Csv,
            "application/json" => return ResponseFormat::Json,
            _ => {}
        }
    }

    ResponseFormat::Json
}

#[must_use]
pub fn build_csv_response(hops: &[TravelHop]) -> Response {
    let mut writer = csv::Writer::from_writer(Vec::new());
    if let Err(err) = writer.write_record([
        "travel_type",
        "origin_name",
        "origin_lat",
        "origin_lng",
        "dest_name",
        "dest_lat",
        "dest_lng",
        "start_date",
        "end_date",
    ]) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("failed to write CSV header: {err}"),
        )
            .into_response();
    }

    for hop in hops {
        if let Err(err) = writer.write_record([
            hop.travel_type.to_string(),
            hop.origin_name.clone(),
            opt_f64_to_string(hop.origin_lat),
            opt_f64_to_string(hop.origin_lng),
            hop.dest_name.clone(),
            opt_f64_to_string(hop.dest_lat),
            opt_f64_to_string(hop.dest_lng),
            hop.start_date.clone(),
            hop.end_date.clone(),
        ]) {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed to write CSV record: {err}"),
            )
                .into_response();
        }
    }

    if let Err(err) = writer.flush() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("failed to flush CSV writer: {err}"),
        )
            .into_response();
    }

    let body = match writer.into_inner() {
        Ok(bytes) => bytes,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed to build CSV response body: {}", err.into_error()),
            )
                .into_response();
        }
    };

    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "text/csv; charset=utf-8"),
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"hops.csv\"",
            ),
        ],
        body,
    )
        .into_response()
}

#[must_use]
pub fn build_html_response(hops: &[TravelHop]) -> Response {
    use leptos::prelude::*;

    let rows = hops
        .iter()
        .map(|hop| {
            let travel_type = format!("{} {}", hop.travel_type.emoji(), hop.travel_type);
            let origin = hop.origin_name.clone();
            let origin_lat = opt_f64_to_string(hop.origin_lat);
            let origin_lng = opt_f64_to_string(hop.origin_lng);
            let dest = hop.dest_name.clone();
            let dest_lat = opt_f64_to_string(hop.dest_lat);
            let dest_lng = opt_f64_to_string(hop.dest_lng);
            let start = hop.start_date.clone();
            let end = hop.end_date.clone();
            view! {
                <tr>
                    <td>{travel_type}</td>
                    <td>{origin}</td>
                    <td>{origin_lat}</td>
                    <td>{origin_lng}</td>
                    <td>{dest}</td>
                    <td>{dest_lat}</td>
                    <td>{dest_lng}</td>
                    <td>{start}</td>
                    <td>{end}</td>
                </tr>
            }
        })
        .collect::<Vec<_>>();

    let html = view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <title>"Travel Hops"</title>
                <link rel="stylesheet" href="/static/style.css" />
            </head>
            <body>
                <h1>"Travel Hops"</h1>
                <table>
                    <thead>
                        <tr>
                            <th>"Type"</th>
                            <th>"Origin"</th>
                            <th>"Origin Lat"</th>
                            <th>"Origin Lng"</th>
                            <th>"Destination"</th>
                            <th>"Dest Lat"</th>
                            <th>"Dest Lng"</th>
                            <th>"Start Date"</th>
                            <th>"End Date"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {rows}
                    </tbody>
                </table>
            </body>
        </html>
    };

    axum::response::Html(html.to_html()).into_response()
}

fn opt_f64_to_string(val: Option<f64>) -> String {
    val.map_or_else(String::new, |v| v.to_string())
}

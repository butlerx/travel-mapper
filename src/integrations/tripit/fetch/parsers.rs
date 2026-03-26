//! Parsers for `TripIt` travel-object types (air, rail, cruise, transport).

use super::{
    BoatDetail, FlightDetail, Hop, RailDetail, TransportDetail, TravelType, build_address_query,
    coerce_float, ensure_list, extract_coords, extract_country, get_date, get_str, get_tz,
    strip_station_suffix,
};
use crate::geocode::airports;
use serde_json::Value;

pub(super) fn parse_air(obj: &Value) -> Vec<Hop> {
    ensure_list(&obj["Segment"])
        .into_iter()
        .filter_map(|seg| {
            let origin_code = get_str(seg, "start_airport_code");
            let dest_code = get_str(seg, "end_airport_code");
            if origin_code.is_empty() && dest_code.is_empty() {
                return None;
            }

            let (origin_name, origin_fallback, origin_country) = if origin_code.is_empty() {
                (get_str(seg, "start_city_name"), (0.0, 0.0), None)
            } else {
                let enriched = airports::lookup_enriched(&origin_code);
                let coords = enriched
                    .as_ref()
                    .map_or((0.0, 0.0), |a| (a.latitude, a.longitude));
                let country = enriched.map(|a| a.country_code);
                (origin_code, coords, country)
            };

            let (dest_name, dest_fallback, dest_country) = if dest_code.is_empty() {
                (get_str(seg, "end_city_name"), (0.0, 0.0), None)
            } else {
                let enriched = airports::lookup_enriched(&dest_code);
                let coords = enriched
                    .as_ref()
                    .map_or((0.0, 0.0), |a| (a.latitude, a.longitude));
                let country = enriched.map(|a| a.country_code);
                (dest_code, coords, country)
            };

            Some(Hop {
                id: 0,
                travel_type: TravelType::Air,
                origin_name,
                origin_lat: coerce_float(&seg["start_airport_latitude"])
                    .unwrap_or(origin_fallback.0),
                origin_lng: coerce_float(&seg["start_airport_longitude"])
                    .unwrap_or(origin_fallback.1),
                origin_country,
                dest_name,
                dest_lat: coerce_float(&seg["end_airport_latitude"]).unwrap_or(dest_fallback.0),
                dest_lng: coerce_float(&seg["end_airport_longitude"]).unwrap_or(dest_fallback.1),
                dest_country,
                start_date: get_date(seg, "StartDateTime"),
                end_date: get_date(seg, "EndDateTime"),
                raw_json: serde_json::to_string(seg).ok(),
                origin_address_query: None,
                dest_address_query: None,
                origin_tz: get_tz(seg, "StartDateTime"),
                dest_tz: get_tz(seg, "EndDateTime"),
                flight_detail: Some(FlightDetail {
                    airline: get_str(seg, "marketing_airline"),
                    flight_number: get_str(seg, "marketing_flight_number"),
                    aircraft_type: get_str(seg, "aircraft_display_name"),
                    cabin_class: get_str(seg, "service_class"),
                    seat: get_str(seg, "seats"),
                    pnr: get_str(obj, "confirmation_num"),
                }),
                rail_detail: None,
                boat_detail: None,
                transport_detail: None,
                cached_carrier: None,
                cost_amount: None,
                cost_currency: None,
                loyalty_program: None,
                miles_earned: None,
            })
        })
        .collect()
}

pub(super) fn parse_rail(obj: &Value) -> Vec<Hop> {
    ensure_list(&obj["Segment"])
        .into_iter()
        .filter_map(|seg| {
            let start_addr = &seg["StartStationAddress"];
            let end_addr = &seg["EndStationAddress"];
            let (slat, slng) = extract_coords(start_addr);
            let (dlat, dlng) = extract_coords(end_addr);

            let origin_name = {
                let name = get_str(seg, "start_station_name");
                if name.is_empty() {
                    get_str(start_addr, "city")
                } else {
                    strip_station_suffix(&name)
                }
            };
            let dest_name = {
                let name = get_str(seg, "end_station_name");
                if name.is_empty() {
                    get_str(end_addr, "city")
                } else {
                    strip_station_suffix(&name)
                }
            };

            if origin_name.is_empty() && dest_name.is_empty() {
                return None;
            }

            Some(Hop {
                id: 0,
                travel_type: TravelType::Rail,
                origin_name,
                origin_lat: slat,
                origin_lng: slng,
                origin_country: extract_country(start_addr),
                dest_name,
                dest_lat: dlat,
                dest_lng: dlng,
                dest_country: extract_country(end_addr),
                start_date: get_date(seg, "StartDateTime"),
                end_date: get_date(seg, "EndDateTime"),
                raw_json: serde_json::to_string(seg).ok(),
                origin_address_query: build_address_query(start_addr),
                dest_address_query: build_address_query(end_addr),
                origin_tz: get_tz(seg, "StartDateTime"),
                dest_tz: get_tz(seg, "EndDateTime"),
                flight_detail: None,
                rail_detail: Some(RailDetail {
                    carrier: get_str(seg, "carrier_name"),
                    train_number: get_str(seg, "train_number"),
                    service_class: get_str(seg, "service_class"),
                    coach_number: get_str(seg, "coach_number"),
                    seats: get_str(seg, "seats"),
                    confirmation_num: get_str(obj, "confirmation_num"),
                    booking_site: get_str(seg, "booking_site_name"),
                    notes: get_str(seg, "notes"),
                }),
                boat_detail: None,
                transport_detail: None,
                cached_carrier: None,
                cost_amount: None,
                cost_currency: None,
                loyalty_program: None,
                miles_earned: None,
            })
        })
        .collect()
}

pub(super) fn parse_cruise(obj: &Value) -> Vec<Hop> {
    let start_addr = &obj["StartLocationAddress"];
    let end_addr = &obj["EndLocationAddress"];
    let (slat, slng) = extract_coords(start_addr);
    let (dlat, dlng) = extract_coords(end_addr);

    let origin_name = {
        let name = get_str(obj, "start_location_name");
        if name.is_empty() {
            get_str(start_addr, "city")
        } else {
            name
        }
    };
    let dest_name = {
        let name = get_str(obj, "end_location_name");
        if name.is_empty() {
            get_str(end_addr, "city")
        } else {
            name
        }
    };

    vec![Hop {
        id: 0,
        travel_type: TravelType::Boat,
        origin_name,
        origin_lat: slat,
        origin_lng: slng,
        origin_country: extract_country(start_addr),
        dest_name,
        dest_lat: dlat,
        dest_lng: dlng,
        dest_country: extract_country(end_addr),
        start_date: get_date(obj, "StartDateTime"),
        end_date: get_date(obj, "EndDateTime"),
        raw_json: serde_json::to_string(obj).ok(),
        origin_address_query: build_address_query(start_addr),
        dest_address_query: build_address_query(end_addr),
        origin_tz: get_tz(obj, "StartDateTime"),
        dest_tz: get_tz(obj, "EndDateTime"),
        flight_detail: None,
        rail_detail: None,
        boat_detail: Some(BoatDetail {
            ship_name: get_str(obj, "ship_name"),
            cabin_type: get_str(obj, "cabin_type"),
            cabin_number: get_str(obj, "cabin_number"),
            confirmation_num: get_str(obj, "confirmation_num"),
            booking_site: get_str(obj, "booking_site_name"),
            notes: get_str(obj, "notes"),
        }),
        transport_detail: None,
        cached_carrier: None,
        cost_amount: None,
        cost_currency: None,
        loyalty_program: None,
        miles_earned: None,
    }]
}

fn resolve_transport_addr<'a>(seg: &'a Value, location_key: &str, fallback_key: &str) -> &'a Value {
    let addr = &seg[location_key];
    if addr.is_null() {
        &seg[fallback_key]
    } else {
        addr
    }
}

pub(super) fn parse_transport(obj: &Value) -> Vec<Hop> {
    let nested = &obj["Segment"];
    let items = if nested.is_null() {
        vec![obj]
    } else {
        ensure_list(nested)
    };

    items
        .into_iter()
        .map(|seg| {
            let start_addr = resolve_transport_addr(seg, "StartLocationAddress", "StartAddress");
            let end_addr = resolve_transport_addr(seg, "EndLocationAddress", "EndAddress");
            let (slat, slng) = extract_coords(start_addr);
            let (dlat, dlng) = extract_coords(end_addr);

            let origin_name = {
                let name = get_str(seg, "start_location_name");
                if name.is_empty() {
                    get_str(start_addr, "city")
                } else {
                    name
                }
            };
            let dest_name = {
                let name = get_str(seg, "end_location_name");
                if name.is_empty() {
                    get_str(end_addr, "city")
                } else {
                    name
                }
            };

            let carrier_name = get_str(seg, "carrier_name");
            let vehicle_description = get_str(seg, "vehicle_description");
            let is_ferry = {
                let c = carrier_name.to_ascii_lowercase();
                let v = vehicle_description.to_ascii_lowercase();
                c.contains("ferry")
                    || c.contains("ferries")
                    || v.contains("ferry")
                    || v.contains("ferries")
            };

            let (travel_type, boat_detail, transport_detail) = if is_ferry {
                (
                    TravelType::Boat,
                    Some(BoatDetail {
                        ship_name: carrier_name,
                        confirmation_num: get_str(obj, "confirmation_num"),
                        notes: get_str(seg, "notes"),
                        ..BoatDetail::default()
                    }),
                    None,
                )
            } else {
                (
                    TravelType::Transport,
                    None,
                    Some(TransportDetail {
                        carrier_name,
                        vehicle_description,
                        confirmation_num: get_str(obj, "confirmation_num"),
                        notes: get_str(seg, "notes"),
                    }),
                )
            };

            Hop {
                id: 0,
                travel_type,
                origin_name,
                origin_lat: slat,
                origin_lng: slng,
                origin_country: extract_country(start_addr),
                dest_name,
                dest_lat: dlat,
                dest_lng: dlng,
                dest_country: extract_country(end_addr),
                start_date: get_date(seg, "StartDateTime"),
                end_date: get_date(seg, "EndDateTime"),
                raw_json: serde_json::to_string(seg).ok(),
                origin_address_query: build_address_query(start_addr),
                dest_address_query: build_address_query(end_addr),
                origin_tz: get_tz(seg, "StartDateTime"),
                dest_tz: get_tz(seg, "EndDateTime"),
                flight_detail: None,
                rail_detail: None,
                boat_detail,
                transport_detail,
                cached_carrier: None,
                cost_amount: None,
                cost_currency: None,
                loyalty_program: None,
                miles_earned: None,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parse_air_normal_case_with_all_fields() {
        let obj = json!({
            "confirmation_num": "PNR123",
            "Segment": [{
                "start_airport_code": "LHR",
                "start_airport_latitude": "51.4700",
                "start_airport_longitude": -0.4543,
                "end_airport_code": "JFK",
                "end_airport_latitude": 40.6413,
                "end_airport_longitude": "-73.7781",
                "marketing_airline": "BA",
                "marketing_flight_number": "117",
                "aircraft_display_name": "Boeing 777",
                "service_class": "Business",
                "seats": "2A",
                "StartDateTime": {"date": "2024-03-01"},
                "EndDateTime": {"date": "2024-03-01"}
            }]
        });

        let hops = parse_air(&obj);
        assert_eq!(hops.len(), 1);
        let hop = &hops[0];
        assert!(matches!(hop.travel_type, TravelType::Air));
        assert_eq!(hop.origin_name, "LHR");
        assert_eq!(hop.dest_name, "JFK");
        assert!((hop.origin_lat - 51.47_f64).abs() < f64::EPSILON);
        assert!((hop.origin_lng - -0.4543_f64).abs() < f64::EPSILON);
        assert!((hop.dest_lat - 40.6413_f64).abs() < f64::EPSILON);
        assert!((hop.dest_lng - -73.7781_f64).abs() < f64::EPSILON);
        assert_eq!(hop.start_date, "2024-03-01");
        assert_eq!(hop.end_date, "2024-03-01");
        let detail = hop.flight_detail.as_ref().expect("flight detail missing");
        assert_eq!(detail.airline, "BA");
        assert_eq!(detail.flight_number, "117");
        assert_eq!(detail.seat, "2A");
        assert_eq!(detail.pnr, "PNR123");
    }

    #[test]
    fn parse_air_handles_single_object_missing_coords_and_optional_fields() {
        let obj = json!({
            "Segment": {
                "start_airport_code": "ZZZ",
                "start_airport_latitude": null,
                "end_airport_code": null,
                "end_city_name": "New York",
                "end_airport_longitude": "",
                "StartDateTime": {"date": ""}
            }
        });

        let hops = parse_air(&obj);
        assert_eq!(hops.len(), 1);
        let hop = &hops[0];
        assert!(matches!(hop.travel_type, TravelType::Air));
        assert_eq!(hop.origin_name, "ZZZ");
        assert_eq!(hop.dest_name, "New York");
        assert!((hop.origin_lat).abs() < f64::EPSILON);
        assert!((hop.origin_lng).abs() < f64::EPSILON);
        assert!((hop.dest_lat).abs() < f64::EPSILON);
        assert!((hop.dest_lng).abs() < f64::EPSILON);
        assert_eq!(hop.start_date, "");
        assert_eq!(hop.end_date, "");
    }

    #[test]
    fn parse_air_falls_back_to_airport_lookup_when_coords_missing() {
        use crate::geocode::airports;

        let obj = json!({
            "Segment": [{
                "start_airport_code": "DUB",
                "end_airport_code": "JFK",
                "StartDateTime": {"date": "2024-06-15"},
                "EndDateTime": {"date": "2024-06-15"}
            }]
        });

        let hops = parse_air(&obj);
        assert_eq!(hops.len(), 1);
        let hop = &hops[0];
        let dub = airports::lookup_enriched("DUB").unwrap();
        let jfk = airports::lookup_enriched("JFK").unwrap();
        assert!((hop.origin_lat - dub.latitude).abs() < f64::EPSILON);
        assert!((hop.origin_lng - dub.longitude).abs() < f64::EPSILON);
        assert!((hop.dest_lat - jfk.latitude).abs() < f64::EPSILON);
        assert!((hop.dest_lng - jfk.longitude).abs() < f64::EPSILON);
    }

    #[test]
    fn parse_air_handles_empty_and_null_segments() {
        let empty = json!({"Segment": []});
        let null = json!({"Segment": null});

        assert!(parse_air(&empty).is_empty());
        assert!(parse_air(&null).is_empty());
    }

    #[test]
    fn parse_air_skips_segments_with_both_codes_empty() {
        let obj = json!({
            "Segment": [
                {
                    "start_airport_code": "",
                    "end_airport_code": "",
                    "StartDateTime": {"date": "2024-01-01"},
                    "EndDateTime": {"date": "2024-01-01"}
                },
                {
                    "start_airport_code": "DUB",
                    "end_airport_code": "LHR",
                    "StartDateTime": {"date": "2024-01-02"},
                    "EndDateTime": {"date": "2024-01-02"}
                }
            ]
        });
        let hops = parse_air(&obj);
        assert_eq!(hops.len(), 1);
        assert_eq!(hops[0].origin_name, "DUB");
    }

    #[test]
    fn parse_air_sets_country_from_airport_data() {
        let obj = json!({
            "Segment": [{
                "start_airport_code": "DUB",
                "end_airport_code": "JFK",
                "StartDateTime": {"date": "2024-06-15"},
                "EndDateTime": {"date": "2024-06-15"}
            }]
        });

        let hops = parse_air(&obj);
        assert_eq!(hops[0].origin_country.as_deref(), Some("ie"));
        assert_eq!(hops[0].dest_country.as_deref(), Some("us"));
    }

    #[test]
    fn parse_air_falls_back_to_city_name_when_code_missing() {
        let obj = json!({
            "Segment": [{
                "start_airport_code": "DUB",
                "end_airport_code": "",
                "end_city_name": "Milan",
                "StartDateTime": {"date": "2024-06-15"},
                "EndDateTime": {"date": "2024-06-15"}
            }]
        });

        let hops = parse_air(&obj);
        assert_eq!(hops.len(), 1);
        assert_eq!(hops[0].origin_name, "DUB");
        assert_eq!(hops[0].dest_name, "Milan");
    }

    #[test]
    fn parse_air_uses_empty_string_when_code_and_city_both_missing() {
        let obj = json!({
            "Segment": [{
                "start_airport_code": "",
                "start_city_name": "Dublin",
                "end_airport_code": "",
                "StartDateTime": {"date": "2024-06-15"},
                "EndDateTime": {"date": "2024-06-15"}
            }]
        });

        let hops = parse_air(&obj);
        assert!(hops.is_empty());
    }

    #[test]
    fn parse_rail_normal_case_extracts_station_names_and_coords() {
        let obj = json!({
            "confirmation_num": "RAIL-CNF-1",
            "Segment": [{
                "start_station_name": "Gare du Nord",
                "end_station_name": "St Pancras",
                "carrier_name": "Eurostar",
                "train_number": "ES9012",
                "service_class": "First",
                "coach_number": "C",
                "seats": "12A",
                "booking_site_name": "TripIt",
                "notes": "Platform info in app",
                "StartStationAddress": {"city": "Paris", "latitude": 48.8809, "longitude": 2.3553},
                "EndStationAddress": {"city": "London", "latitude": "51.5319", "longitude": "-0.1263"},
                "StartDateTime": {"date": "2024-05-10"},
                "EndDateTime": {"date": "2024-05-10"}
            }]
        });

        let hops = parse_rail(&obj);
        assert_eq!(hops.len(), 1);
        let hop = &hops[0];
        assert!(matches!(hop.travel_type, TravelType::Rail));
        assert_eq!(hop.origin_name, "Gare du Nord");
        assert_eq!(hop.dest_name, "St Pancras");
        assert!((hop.origin_lat - 48.8809_f64).abs() < f64::EPSILON);
        assert!((hop.origin_lng - 2.3553_f64).abs() < f64::EPSILON);
        assert!((hop.dest_lat - 51.5319_f64).abs() < f64::EPSILON);
        assert!((hop.dest_lng - -0.1263_f64).abs() < f64::EPSILON);
        let detail = hop.rail_detail.as_ref().expect("rail detail missing");
        assert_eq!(detail.carrier, "Eurostar");
        assert_eq!(detail.train_number, "ES9012");
        assert_eq!(detail.confirmation_num, "RAIL-CNF-1");
    }

    #[test]
    fn parse_rail_handles_single_segment_fallback_names_and_missing_coords() {
        let obj = json!({
            "Segment": {
                "start_station_name": "",
                "end_station_name": "",
                "StartStationAddress": {"city": "Berlin", "latitude": null, "longitude": ""},
                "EndStationAddress": {"city": "Munich"},
                "StartDateTime": {"date": "2024-06-01"},
                "EndDateTime": {"date": "2024-06-01"}
            }
        });

        let hops = parse_rail(&obj);
        assert_eq!(hops.len(), 1);
        let hop = &hops[0];
        assert!(matches!(hop.travel_type, TravelType::Rail));
        assert_eq!(hop.origin_name, "Berlin");
        assert_eq!(hop.dest_name, "Munich");
        assert!((hop.origin_lat).abs() < f64::EPSILON);
        assert!((hop.origin_lng).abs() < f64::EPSILON);
        assert!((hop.dest_lat).abs() < f64::EPSILON);
        assert!((hop.dest_lng).abs() < f64::EPSILON);
    }

    #[test]
    fn parse_rail_handles_empty_and_null_segments() {
        let empty = json!({"Segment": []});
        let null = json!({"Segment": null});

        assert!(parse_rail(&empty).is_empty());
        assert!(parse_rail(&null).is_empty());
    }

    #[test]
    fn parse_rail_skips_segments_with_both_names_empty() {
        let obj = json!({
            "Segment": [
                {
                    "start_station_name": "",
                    "end_station_name": "",
                    "StartStationAddress": {},
                    "EndStationAddress": {},
                    "StartDateTime": {"date": "2024-01-01"},
                    "EndDateTime": {"date": "2024-01-01"}
                },
                {
                    "start_station_name": "Belfast 1",
                    "end_station_name": "Dublin Connolly 1",
                    "StartStationAddress": {"latitude": "54.5", "longitude": "-5.9"},
                    "EndStationAddress": {"latitude": "53.3", "longitude": "-6.2"},
                    "StartDateTime": {"date": "2024-01-02"},
                    "EndDateTime": {"date": "2024-01-02"}
                }
            ]
        });
        let hops = parse_rail(&obj);
        assert_eq!(hops.len(), 1);
        assert_eq!(hops[0].origin_name, "Belfast");
        assert_eq!(hops[0].dest_name, "Dublin Connolly");
    }

    #[test]
    fn parse_rail_extracts_country_from_station_address() {
        let obj = json!({
            "Segment": [{
                "start_station_name": "Marrakech",
                "end_station_name": "Casablanca",
                "StartStationAddress": {"country": "MA", "latitude": "31.6", "longitude": "-8.0"},
                "EndStationAddress": {"country": "MA", "latitude": "33.5", "longitude": "-7.6"},
                "StartDateTime": {"date": "2024-08-01"},
                "EndDateTime": {"date": "2024-08-01"}
            }]
        });

        let hops = parse_rail(&obj);
        assert_eq!(hops[0].origin_country.as_deref(), Some("ma"));
        assert_eq!(hops[0].dest_country.as_deref(), Some("ma"));
    }

    #[test]
    fn parse_cruise_normal_case_extracts_location_names_and_coords() {
        let obj = json!({
            "start_location_name": "Port of Miami",
            "end_location_name": "Nassau Port",
            "ship_name": "Ocean Dream",
            "cabin_type": "Balcony",
            "cabin_number": "B123",
            "confirmation_num": "CRUISE-55",
            "booking_site_name": "Cruise Co",
            "notes": "Boarding 2pm",
            "StartLocationAddress": {"city": "Miami", "latitude": 25.7781, "longitude": -80.1794},
            "EndLocationAddress": {"city": "Nassau", "latitude": "25.0780", "longitude": "-77.3431"},
            "StartDateTime": {"date": "2024-07-01"},
            "EndDateTime": {"date": "2024-07-05"}
        });

        let hops = parse_cruise(&obj);
        assert_eq!(hops.len(), 1);
        let hop = &hops[0];
        assert!(matches!(hop.travel_type, TravelType::Boat));
        assert_eq!(hop.origin_name, "Port of Miami");
        assert_eq!(hop.dest_name, "Nassau Port");
        assert!((hop.origin_lat - 25.7781_f64).abs() < f64::EPSILON);
        assert!((hop.origin_lng - -80.1794_f64).abs() < f64::EPSILON);
        assert!((hop.dest_lat - 25.078_f64).abs() < f64::EPSILON);
        assert!((hop.dest_lng - -77.3431_f64).abs() < f64::EPSILON);
        assert_eq!(hop.start_date, "2024-07-01");
        assert_eq!(hop.end_date, "2024-07-05");
        let detail = hop.boat_detail.as_ref().expect("boat detail missing");
        assert_eq!(detail.ship_name, "Ocean Dream");
        assert_eq!(detail.cabin_number, "B123");
        assert_eq!(detail.confirmation_num, "CRUISE-55");
    }

    #[test]
    fn parse_cruise_handles_missing_optional_fields_and_coord_fallbacks() {
        let obj = json!({
            "start_location_name": "",
            "end_location_name": null,
            "StartLocationAddress": {"city": "Venice", "latitude": null, "longitude": null},
            "EndLocationAddress": {"city": "Dubrovnik"},
            "StartDateTime": {"date": ""},
            "EndDateTime": null
        });

        let hops = parse_cruise(&obj);
        assert_eq!(hops.len(), 1);
        let hop = &hops[0];
        assert!(matches!(hop.travel_type, TravelType::Boat));
        assert_eq!(hop.origin_name, "Venice");
        assert_eq!(hop.dest_name, "Dubrovnik");
        assert!((hop.origin_lat).abs() < f64::EPSILON);
        assert!((hop.origin_lng).abs() < f64::EPSILON);
        assert!((hop.dest_lat).abs() < f64::EPSILON);
        assert!((hop.dest_lng).abs() < f64::EPSILON);
        assert_eq!(hop.start_date, "");
        assert_eq!(hop.end_date, "");
    }

    #[test]
    fn parse_transport_handles_top_level_object_without_segment_key() {
        let obj = json!({
            "confirmation_num": "TX-7788",
            "start_location_name": "Hotel Lobby",
            "end_location_name": "Airport",
            "carrier_name": "City Taxi",
            "vehicle_description": "Sedan",
            "notes": "Meet at entrance",
            "StartAddress": {"city": "Rome", "latitude": "41.9028", "longitude": "12.4964"},
            "EndAddress": {"city": "Rome", "latitude": 41.8003, "longitude": 12.2389},
            "StartDateTime": {"date": "2024-08-01"},
            "EndDateTime": {"date": "2024-08-01"}
        });

        let hops = parse_transport(&obj);
        assert_eq!(hops.len(), 1);
        let hop = &hops[0];
        assert!(matches!(hop.travel_type, TravelType::Transport));
        assert_eq!(hop.origin_name, "Hotel Lobby");
        assert_eq!(hop.dest_name, "Airport");
        assert!((hop.origin_lat - 41.9028_f64).abs() < f64::EPSILON);
        assert!((hop.origin_lng - 12.4964_f64).abs() < f64::EPSILON);
        assert!((hop.dest_lat - 41.8003_f64).abs() < f64::EPSILON);
        assert!((hop.dest_lng - 12.2389_f64).abs() < f64::EPSILON);
        let detail = hop
            .transport_detail
            .as_ref()
            .expect("transport detail missing");
        assert_eq!(detail.carrier_name, "City Taxi");
        assert_eq!(detail.vehicle_description, "Sedan");
        assert_eq!(detail.confirmation_num, "TX-7788");
    }

    #[test]
    fn parse_transport_detects_ferry_as_boat() {
        let obj = json!({
            "confirmation_num": "FRY-001",
            "start_location_name": "Dover",
            "end_location_name": "Calais",
            "carrier_name": "P&O Ferries",
            "vehicle_description": "Car deck",
            "notes": "Deck 3",
            "StartAddress": {"city": "Dover", "latitude": "51.1279", "longitude": "1.3134"},
            "EndAddress": {"city": "Calais", "latitude": "50.9513", "longitude": "1.8587"},
            "StartDateTime": {"date": "2024-08-15"},
            "EndDateTime": {"date": "2024-08-15"}
        });

        let hops = parse_transport(&obj);
        assert_eq!(hops.len(), 1);
        let hop = &hops[0];
        assert!(matches!(hop.travel_type, TravelType::Boat));
        let detail = hop.boat_detail.as_ref().expect("boat detail missing");
        assert_eq!(detail.ship_name, "P&O Ferries");
        assert_eq!(detail.confirmation_num, "FRY-001");
        assert_eq!(detail.notes, "Deck 3");
        assert!(hop.transport_detail.is_none());
    }

    #[test]
    fn parse_transport_detects_ferry_in_vehicle_description() {
        let obj = json!({
            "start_location_name": "Piraeus",
            "end_location_name": "Santorini",
            "carrier_name": "Blue Star Lines",
            "vehicle_description": "High-speed ferry",
            "StartAddress": {"city": "Piraeus", "latitude": "37.9475", "longitude": "23.6370"},
            "EndAddress": {"city": "Santorini", "latitude": "36.3932", "longitude": "25.4615"},
            "StartDateTime": {"date": "2024-07-20"},
            "EndDateTime": {"date": "2024-07-20"}
        });

        let hops = parse_transport(&obj);
        assert_eq!(hops.len(), 1);
        let hop = &hops[0];
        assert!(matches!(hop.travel_type, TravelType::Boat));
        assert!(hop.boat_detail.is_some());
        assert!(hop.transport_detail.is_none());
    }

    #[test]
    fn parse_transport_handles_nested_segment_array_and_single_object() {
        let array_obj = json!({
            "Segment": [
                {
                    "start_location_name": "",
                    "end_location_name": "",
                    "StartAddress": {"city": "Lisbon", "latitude": null, "longitude": ""},
                    "EndAddress": {"city": "Porto"},
                    "StartDateTime": {"date": "2024-09-01"},
                    "EndDateTime": {"date": "2024-09-01"}
                },
                {
                    "start_location_name": "Porto",
                    "end_location_name": "Braga",
                    "StartAddress": {"city": "Porto", "latitude": 41.1496, "longitude": -8.6109},
                    "EndAddress": {"city": "Braga", "latitude": 41.5454, "longitude": -8.4265},
                    "StartDateTime": {"date": "2024-09-02"},
                    "EndDateTime": {"date": "2024-09-02"}
                }
            ]
        });

        let single_obj = json!({
            "Segment": {
                "start_location_name": "",
                "end_location_name": "",
                "StartAddress": {"city": "Madrid"},
                "EndAddress": {"city": "Toledo"},
                "StartDateTime": {"date": "2024-10-01"},
                "EndDateTime": {"date": "2024-10-01"}
            }
        });

        let array_hops = parse_transport(&array_obj);
        assert_eq!(array_hops.len(), 2);
        assert!(matches!(array_hops[0].travel_type, TravelType::Transport));
        assert_eq!(array_hops[0].origin_name, "Lisbon");
        assert_eq!(array_hops[0].dest_name, "Porto");
        assert!((array_hops[0].origin_lat).abs() < f64::EPSILON);
        assert!((array_hops[0].origin_lng).abs() < f64::EPSILON);
        assert!(matches!(array_hops[1].travel_type, TravelType::Transport));

        let single_hops = parse_transport(&single_obj);
        assert_eq!(single_hops.len(), 1);
        assert_eq!(single_hops[0].origin_name, "Madrid");
        assert_eq!(single_hops[0].dest_name, "Toledo");
    }

    #[test]
    fn parse_transport_handles_empty_segment_array() {
        let obj = json!({"Segment": []});
        assert!(parse_transport(&obj).is_empty());
    }

    #[test]
    fn parse_transport_prefers_start_location_address_key() {
        let obj = json!({
            "start_location_name": "Hotel",
            "end_location_name": "Airport",
            "StartLocationAddress": {"city": "Dublin", "latitude": "53.35", "longitude": "-6.26"},
            "EndLocationAddress": {"city": "Dublin", "latitude": "53.42", "longitude": "-6.27"},
            "StartDateTime": {"date": "2024-08-01"},
            "EndDateTime": {"date": "2024-08-01"}
        });

        let hops = parse_transport(&obj);
        assert_eq!(hops.len(), 1);
        assert!((hops[0].origin_lat - 53.35_f64).abs() < f64::EPSILON);
        assert!((hops[0].origin_lng - (-6.26_f64)).abs() < f64::EPSILON);
        assert!((hops[0].dest_lat - 53.42_f64).abs() < f64::EPSILON);
        assert!((hops[0].dest_lng - (-6.27_f64)).abs() < f64::EPSILON);
    }

    #[test]
    fn parse_transport_falls_back_to_start_address_key() {
        let obj = json!({
            "start_location_name": "Hotel",
            "end_location_name": "Airport",
            "StartAddress": {"city": "Rome", "latitude": "41.90", "longitude": "12.50"},
            "EndAddress": {"city": "Rome", "latitude": "41.80", "longitude": "12.24"},
            "StartDateTime": {"date": "2024-08-01"},
            "EndDateTime": {"date": "2024-08-01"}
        });

        let hops = parse_transport(&obj);
        assert_eq!(hops.len(), 1);
        assert!((hops[0].origin_lat - 41.90_f64).abs() < f64::EPSILON);
        assert!((hops[0].origin_lng - 12.50_f64).abs() < f64::EPSILON);
        assert!((hops[0].dest_lat - 41.80_f64).abs() < f64::EPSILON);
        assert!((hops[0].dest_lng - 12.24_f64).abs() < f64::EPSILON);
    }

    #[test]
    fn parse_transport_location_address_wins_over_start_address() {
        let obj = json!({
            "start_location_name": "Hotel",
            "end_location_name": "Airport",
            "StartLocationAddress": {"city": "Dublin", "latitude": "53.35", "longitude": "-6.26"},
            "StartAddress": {"city": "Wrong", "latitude": "0.0", "longitude": "0.0"},
            "EndLocationAddress": {"city": "Dublin", "latitude": "53.42", "longitude": "-6.27"},
            "EndAddress": {"city": "Wrong", "latitude": "0.0", "longitude": "0.0"},
            "StartDateTime": {"date": "2024-08-01"},
            "EndDateTime": {"date": "2024-08-01"}
        });

        let hops = parse_transport(&obj);
        assert_eq!(hops.len(), 1);
        assert!((hops[0].origin_lat - 53.35_f64).abs() < f64::EPSILON);
        assert!((hops[0].dest_lat - 53.42_f64).abs() < f64::EPSILON);
    }

    #[test]
    fn parse_transport_extracts_country_from_address() {
        let obj = json!({
            "start_location_name": "Hotel",
            "end_location_name": "Airport",
            "StartAddress": {"city": "Marrakech", "country": "MA", "latitude": "0", "longitude": "0"},
            "EndAddress": {"city": "Marrakech", "country": "MA", "latitude": "0", "longitude": "0"},
            "StartDateTime": {"date": "2024-08-01"},
            "EndDateTime": {"date": "2024-08-01"}
        });

        let hops = parse_transport(&obj);
        assert_eq!(hops[0].origin_country.as_deref(), Some("ma"));
        assert_eq!(hops[0].dest_country.as_deref(), Some("ma"));
    }
}

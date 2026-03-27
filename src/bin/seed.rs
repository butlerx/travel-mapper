use clap::Parser;
use sqlx::SqlitePool;
use travel_mapper::db;

#[derive(Parser)]
#[command(about = "Seed the database with test users for local development")]
struct Cli {
    #[arg(long, env = "DATABASE_URL", default_value = "sqlite:travel.db")]
    database_url: String,

    /// Hex-encoded 32-byte AES-256-GCM key (64 hex chars). Required to seed
    /// TripIt credentials; ignored when absent.
    #[arg(long, env = "ENCRYPTION_KEY")]
    encryption_key: Option<String>,

    /// TripIt OAuth access token. Both token fields plus the encryption key
    /// must be set to seed credentials.
    #[arg(long, env = "TRIPIT_ACCESS_TOKEN")]
    tripit_access_token: Option<String>,

    /// TripIt OAuth access token secret.
    #[arg(long, env = "TRIPIT_ACCESS_TOKEN_SECRET")]
    tripit_access_token_secret: Option<String>,
}

#[derive(Debug, thiserror::Error)]
enum SeedError {
    #[error("invalid ENCRYPTION_KEY: expected exactly 64 hex characters (32 bytes)")]
    InvalidEncryptionKey,

    #[error("{0}")]
    Database(#[from] sqlx::Error),

    #[error("failed to encrypt token: {0}")]
    Encrypt(#[from] travel_mapper::auth::CryptoError),

    #[error("failed to hash password: {0}")]
    HashPassword(argon2::password_hash::Error),

    #[error("TRIPIT_ACCESS_TOKEN and TRIPIT_ACCESS_TOKEN_SECRET must both be set or both be unset")]
    IncompleteCredentials,

    #[error("user {0:?} not found after unique violation")]
    UserNotFound(String),
}

impl From<argon2::password_hash::Error> for SeedError {
    fn from(err: argon2::password_hash::Error) -> Self {
        Self::HashPassword(err)
    }
}

fn parse_encryption_key(hex: &str) -> Result<[u8; 32], SeedError> {
    if hex.len() != 64 {
        return Err(SeedError::InvalidEncryptionKey);
    }
    let mut out = [0_u8; 32];
    for (idx, chunk) in hex.as_bytes().chunks(2).enumerate() {
        let pair = std::str::from_utf8(chunk).map_err(|_| SeedError::InvalidEncryptionKey)?;
        out[idx] = u8::from_str_radix(pair, 16).map_err(|_| SeedError::InvalidEncryptionKey)?;
    }
    Ok(out)
}

async fn seed_tripit_credentials(
    pool: &SqlitePool,
    user_id: i64,
    username: &str,
    access_token: &str,
    access_token_secret: &str,
    encryption_key: &[u8; 32],
) -> Result<(), SeedError> {
    let (token_enc, nonce_token) =
        travel_mapper::auth::encrypt_token(access_token, encryption_key)?;
    let (secret_enc, nonce_secret) =
        travel_mapper::auth::encrypt_token(access_token_secret, encryption_key)?;

    (db::credentials::Upsert {
        user_id,
        access_token_enc: &token_enc,
        access_token_secret_enc: &secret_enc,
        nonce_token: &nonce_token,
        nonce_secret: &nonce_secret,
    })
    .execute(pool)
    .await?;

    tracing::info!(username, "stored TripIt credentials");
    Ok(())
}

async fn seed_trips(pool: &SqlitePool, user_id: i64) -> Result<Vec<i64>, SeedError> {
    let trip_names = [
        "Europe Winter 2024",
        "Japan Spring 2024",
        "Caribbean Summer 2024",
    ];
    let mut trip_ids = Vec::new();

    for name in trip_names {
        let id = (db::trips::Create { user_id, name }).execute(pool).await?;
        tracing::info!(user_id, trip_id = id, name, "created trip");
        trip_ids.push(id);
    }

    Ok(trip_ids)
}

fn build_seed_hops() -> Vec<db::hops::Row> {
    vec![
        db::hops::Row {
            id: 0,
            travel_type: db::hops::TravelType::Air,
            origin_name: "Dublin Airport (DUB)".into(),
            origin_lat: 53.4264,
            origin_lng: -6.2499,
            origin_country: Some("IE".into()),
            dest_name: "London Heathrow (LHR)".into(),
            dest_lat: 51.4700,
            dest_lng: -0.4543,
            dest_country: Some("GB".into()),
            start_date: "2024-01-15".into(),
            end_date: "2024-01-15".into(),
            raw_json: None,
            origin_address_query: None,
            dest_address_query: None,
            origin_tz: None,
            dest_tz: None,
            flight_detail: Some(db::hops::FlightDetail {
                airline: "Aer Lingus".into(),
                flight_number: "EI154".into(),
                aircraft_type: "Airbus A320".into(),
                cabin_class: "Economy".into(),
                seat: "12A".into(),
                pnr: "ABC123".into(),
            }),
            rail_detail: None,
            boat_detail: None,
            transport_detail: None,
            cached_carrier: None,
            cost_amount: Some(89.99),
            cost_currency: Some("EUR".into()),
            loyalty_program: Some("AerClub".into()),
            miles_earned: Some(450.0),
        },
        db::hops::Row {
            id: 0,
            travel_type: db::hops::TravelType::Rail,
            origin_name: "London St Pancras".into(),
            origin_lat: 51.5317,
            origin_lng: -0.1262,
            origin_country: Some("GB".into()),
            dest_name: "Paris Gare du Nord".into(),
            dest_lat: 48.8809,
            dest_lng: 2.3553,
            dest_country: Some("FR".into()),
            start_date: "2024-01-16".into(),
            end_date: "2024-01-16".into(),
            raw_json: None,
            origin_address_query: None,
            dest_address_query: None,
            origin_tz: None,
            dest_tz: None,
            flight_detail: None,
            rail_detail: Some(db::hops::RailDetail {
                carrier: "Eurostar".into(),
                train_number: "ES9024".into(),
                service_class: "Standard Premier".into(),
                coach_number: "7".into(),
                seats: "42".into(),
                confirmation_num: "EUR-778899".into(),
                booking_site: "eurostar.com".into(),
                notes: "Booked early for cheaper fare".into(),
            }),
            boat_detail: None,
            transport_detail: None,
            cached_carrier: None,
            cost_amount: Some(150.00),
            cost_currency: Some("GBP".into()),
            loyalty_program: None,
            miles_earned: None,
        },
        db::hops::Row {
            id: 0,
            travel_type: db::hops::TravelType::Transport,
            origin_name: "Paris Gare du Nord".into(),
            origin_lat: 48.8809,
            origin_lng: 2.3553,
            origin_country: Some("FR".into()),
            dest_name: "Le Marais, Paris".into(),
            dest_lat: 48.8566,
            dest_lng: 2.3522,
            dest_country: Some("FR".into()),
            start_date: "2024-01-16".into(),
            end_date: "2024-01-16".into(),
            raw_json: None,
            origin_address_query: None,
            dest_address_query: None,
            origin_tz: None,
            dest_tz: None,
            flight_detail: None,
            rail_detail: None,
            boat_detail: None,
            transport_detail: Some(db::hops::TransportDetail {
                carrier_name: "G7 Taxis".into(),
                vehicle_description: "Taxi sedan".into(),
                confirmation_num: "G7-2024-0116".into(),
                notes: "Flat rate from Gare du Nord".into(),
            }),
            cached_carrier: None,
            cost_amount: Some(25.00),
            cost_currency: Some("EUR".into()),
            loyalty_program: None,
            miles_earned: None,
        },
        db::hops::Row {
            id: 0,
            travel_type: db::hops::TravelType::Air,
            origin_name: "Paris Charles de Gaulle (CDG)".into(),
            origin_lat: 49.0097,
            origin_lng: 2.5479,
            origin_country: Some("FR".into()),
            dest_name: "Dublin Airport (DUB)".into(),
            dest_lat: 53.4264,
            dest_lng: -6.2499,
            dest_country: Some("IE".into()),
            start_date: "2024-01-20".into(),
            end_date: "2024-01-20".into(),
            raw_json: None,
            origin_address_query: None,
            dest_address_query: None,
            origin_tz: None,
            dest_tz: None,
            flight_detail: Some(db::hops::FlightDetail {
                airline: "Air France".into(),
                flight_number: "AF1116".into(),
                aircraft_type: "Airbus A319".into(),
                cabin_class: "Economy".into(),
                seat: "23F".into(),
                pnr: "XYZAF1".into(),
            }),
            rail_detail: None,
            boat_detail: None,
            transport_detail: None,
            cached_carrier: None,
            cost_amount: Some(112.50),
            cost_currency: Some("EUR".into()),
            loyalty_program: Some("Flying Blue".into()),
            miles_earned: Some(500.0),
        },
        db::hops::Row {
            id: 0,
            travel_type: db::hops::TravelType::Air,
            origin_name: "Dublin Airport (DUB)".into(),
            origin_lat: 53.4264,
            origin_lng: -6.2499,
            origin_country: Some("IE".into()),
            dest_name: "London Heathrow (LHR)".into(),
            dest_lat: 51.4700,
            dest_lng: -0.4543,
            dest_country: Some("GB".into()),
            start_date: "2024-04-01".into(),
            end_date: "2024-04-01".into(),
            raw_json: None,
            origin_address_query: None,
            dest_address_query: None,
            origin_tz: None,
            dest_tz: None,
            flight_detail: Some(db::hops::FlightDetail {
                airline: "British Airways".into(),
                flight_number: "BA835".into(),
                aircraft_type: "Airbus A320neo".into(),
                cabin_class: "Economy".into(),
                seat: "18C".into(),
                pnr: "BA2024APR".into(),
            }),
            rail_detail: None,
            boat_detail: None,
            transport_detail: None,
            cached_carrier: None,
            cost_amount: Some(78.00),
            cost_currency: Some("GBP".into()),
            loyalty_program: Some("Avios".into()),
            miles_earned: Some(280.0),
        },
        db::hops::Row {
            id: 0,
            travel_type: db::hops::TravelType::Air,
            origin_name: "London Heathrow (LHR)".into(),
            origin_lat: 51.4700,
            origin_lng: -0.4543,
            origin_country: Some("GB".into()),
            dest_name: "Tokyo Narita (NRT)".into(),
            dest_lat: 35.7647,
            dest_lng: 140.3864,
            dest_country: Some("JP".into()),
            start_date: "2024-04-01".into(),
            end_date: "2024-04-02".into(),
            raw_json: None,
            origin_address_query: None,
            dest_address_query: None,
            origin_tz: None,
            dest_tz: None,
            flight_detail: Some(db::hops::FlightDetail {
                airline: "Japan Airlines".into(),
                flight_number: "JL44".into(),
                aircraft_type: "Boeing 787-9".into(),
                cabin_class: "Premium Economy".into(),
                seat: "28K".into(),
                pnr: "JL44APR01".into(),
            }),
            rail_detail: None,
            boat_detail: None,
            transport_detail: None,
            cached_carrier: None,
            cost_amount: Some(1250.00),
            cost_currency: Some("GBP".into()),
            loyalty_program: Some("JAL Mileage Bank".into()),
            miles_earned: Some(5974.0),
        },
        db::hops::Row {
            id: 0,
            travel_type: db::hops::TravelType::Rail,
            origin_name: "Tokyo Station".into(),
            origin_lat: 35.6812,
            origin_lng: 139.7671,
            origin_country: Some("JP".into()),
            dest_name: "Kyoto Station".into(),
            dest_lat: 34.9857,
            dest_lng: 135.7589,
            dest_country: Some("JP".into()),
            start_date: "2024-04-03".into(),
            end_date: "2024-04-03".into(),
            raw_json: None,
            origin_address_query: None,
            dest_address_query: None,
            origin_tz: None,
            dest_tz: None,
            flight_detail: None,
            rail_detail: Some(db::hops::RailDetail {
                carrier: "JR Central".into(),
                train_number: "Nozomi 225".into(),
                service_class: "Green Car".into(),
                coach_number: "8".into(),
                seats: "3A".into(),
                confirmation_num: "JR-2024-0403".into(),
                booking_site: "smartex.jrcentral.co.jp".into(),
                notes: "Japan Rail Pass not valid for Nozomi".into(),
            }),
            boat_detail: None,
            transport_detail: None,
            cached_carrier: None,
            cost_amount: Some(13320.0),
            cost_currency: Some("JPY".into()),
            loyalty_program: None,
            miles_earned: None,
        },
        db::hops::Row {
            id: 0,
            travel_type: db::hops::TravelType::Rail,
            origin_name: "Kyoto Station".into(),
            origin_lat: 34.9857,
            origin_lng: 135.7589,
            origin_country: Some("JP".into()),
            dest_name: "Osaka Station".into(),
            dest_lat: 34.7024,
            dest_lng: 135.4959,
            dest_country: Some("JP".into()),
            start_date: "2024-04-07".into(),
            end_date: "2024-04-07".into(),
            raw_json: None,
            origin_address_query: None,
            dest_address_query: None,
            origin_tz: None,
            dest_tz: None,
            flight_detail: None,
            rail_detail: Some(db::hops::RailDetail {
                carrier: "JR West".into(),
                train_number: "Special Rapid".into(),
                service_class: "Reserved".into(),
                coach_number: "4".into(),
                seats: "12D".into(),
                confirmation_num: "JR-2024-0407".into(),
                booking_site: "jr-odekake.net".into(),
                notes: String::new(),
            }),
            boat_detail: None,
            transport_detail: None,
            cached_carrier: None,
            cost_amount: Some(580.0),
            cost_currency: Some("JPY".into()),
            loyalty_program: None,
            miles_earned: None,
        },
        db::hops::Row {
            id: 0,
            travel_type: db::hops::TravelType::Air,
            origin_name: "Osaka Kansai (KIX)".into(),
            origin_lat: 34.4347,
            origin_lng: 135.2441,
            origin_country: Some("JP".into()),
            dest_name: "Helsinki Vantaa (HEL)".into(),
            dest_lat: 60.3172,
            dest_lng: 24.9633,
            dest_country: Some("FI".into()),
            start_date: "2024-04-10".into(),
            end_date: "2024-04-10".into(),
            raw_json: None,
            origin_address_query: None,
            dest_address_query: None,
            origin_tz: None,
            dest_tz: None,
            flight_detail: Some(db::hops::FlightDetail {
                airline: "Finnair".into(),
                flight_number: "AY78".into(),
                aircraft_type: "Airbus A350-900".into(),
                cabin_class: "Economy".into(),
                seat: "35A".into(),
                pnr: "FINAY78".into(),
            }),
            rail_detail: None,
            boat_detail: None,
            transport_detail: None,
            cached_carrier: None,
            cost_amount: Some(680.00),
            cost_currency: Some("EUR".into()),
            loyalty_program: Some("Finnair Plus".into()),
            miles_earned: Some(4890.0),
        },
        db::hops::Row {
            id: 0,
            travel_type: db::hops::TravelType::Air,
            origin_name: "Helsinki Vantaa (HEL)".into(),
            origin_lat: 60.3172,
            origin_lng: 24.9633,
            origin_country: Some("FI".into()),
            dest_name: "Dublin Airport (DUB)".into(),
            dest_lat: 53.4264,
            dest_lng: -6.2499,
            dest_country: Some("IE".into()),
            start_date: "2024-04-10".into(),
            end_date: "2024-04-10".into(),
            raw_json: None,
            origin_address_query: None,
            dest_address_query: None,
            origin_tz: None,
            dest_tz: None,
            flight_detail: Some(db::hops::FlightDetail {
                airline: "Finnair".into(),
                flight_number: "AY939".into(),
                aircraft_type: "Airbus A321".into(),
                cabin_class: "Economy".into(),
                seat: "7F".into(),
                pnr: "FINAY939".into(),
            }),
            rail_detail: None,
            boat_detail: None,
            transport_detail: None,
            cached_carrier: None,
            cost_amount: None,
            cost_currency: None,
            loyalty_program: Some("Finnair Plus".into()),
            miles_earned: Some(1820.0),
        },
        db::hops::Row {
            id: 0,
            travel_type: db::hops::TravelType::Air,
            origin_name: "Dublin Airport (DUB)".into(),
            origin_lat: 53.4264,
            origin_lng: -6.2499,
            origin_country: Some("IE".into()),
            dest_name: "London Gatwick (LGW)".into(),
            dest_lat: 51.1537,
            dest_lng: -0.1821,
            dest_country: Some("GB".into()),
            start_date: "2024-07-10".into(),
            end_date: "2024-07-10".into(),
            raw_json: None,
            origin_address_query: None,
            dest_address_query: None,
            origin_tz: None,
            dest_tz: None,
            flight_detail: Some(db::hops::FlightDetail {
                airline: "Ryanair".into(),
                flight_number: "FR114".into(),
                aircraft_type: "Boeing 737-800".into(),
                cabin_class: "Economy".into(),
                seat: "6C".into(),
                pnr: "RYR114JUL".into(),
            }),
            rail_detail: None,
            boat_detail: None,
            transport_detail: None,
            cached_carrier: None,
            cost_amount: Some(45.00),
            cost_currency: Some("EUR".into()),
            loyalty_program: None,
            miles_earned: None,
        },
        db::hops::Row {
            id: 0,
            travel_type: db::hops::TravelType::Air,
            origin_name: "London Gatwick (LGW)".into(),
            origin_lat: 51.1537,
            origin_lng: -0.1821,
            origin_country: Some("GB".into()),
            dest_name: "Grantley Adams Intl (BGI)".into(),
            dest_lat: 13.0747,
            dest_lng: -59.4925,
            dest_country: Some("BB".into()),
            start_date: "2024-07-10".into(),
            end_date: "2024-07-10".into(),
            raw_json: None,
            origin_address_query: None,
            dest_address_query: None,
            origin_tz: None,
            dest_tz: None,
            flight_detail: Some(db::hops::FlightDetail {
                airline: "Virgin Atlantic".into(),
                flight_number: "VS147".into(),
                aircraft_type: "Airbus A330-300".into(),
                cabin_class: "Premium".into(),
                seat: "14A".into(),
                pnr: "VS147JUL".into(),
            }),
            rail_detail: None,
            boat_detail: None,
            transport_detail: None,
            cached_carrier: None,
            cost_amount: Some(650.00),
            cost_currency: Some("GBP".into()),
            loyalty_program: Some("Virgin Points".into()),
            miles_earned: Some(4220.0),
        },
        db::hops::Row {
            id: 0,
            travel_type: db::hops::TravelType::Boat,
            origin_name: "Bridgetown Harbour, Barbados".into(),
            origin_lat: 13.0969,
            origin_lng: -59.6145,
            origin_country: Some("BB".into()),
            dest_name: "Castries Port, St. Lucia".into(),
            dest_lat: 14.0101,
            dest_lng: -60.9878,
            dest_country: Some("LC".into()),
            start_date: "2024-07-14".into(),
            end_date: "2024-07-14".into(),
            raw_json: None,
            origin_address_query: None,
            dest_address_query: None,
            origin_tz: None,
            dest_tz: None,
            flight_detail: None,
            rail_detail: None,
            boat_detail: Some(db::hops::BoatDetail {
                ship_name: "L'Express des Iles".into(),
                cabin_type: "Business".into(),
                cabin_number: "B12".into(),
                confirmation_num: "LEXI-2024-714".into(),
                booking_site: "express-des-iles.com".into(),
                notes: "Inter-island high-speed ferry".into(),
            }),
            transport_detail: None,
            cached_carrier: None,
            cost_amount: Some(95.00),
            cost_currency: Some("USD".into()),
            loyalty_program: None,
            miles_earned: None,
        },
        db::hops::Row {
            id: 0,
            travel_type: db::hops::TravelType::Transport,
            origin_name: "Castries Port, St. Lucia".into(),
            origin_lat: 14.0101,
            origin_lng: -60.9878,
            origin_country: Some("LC".into()),
            dest_name: "Rodney Bay, St. Lucia".into(),
            dest_lat: 14.0722,
            dest_lng: -60.9524,
            dest_country: Some("LC".into()),
            start_date: "2024-07-14".into(),
            end_date: "2024-07-14".into(),
            raw_json: None,
            origin_address_query: None,
            dest_address_query: None,
            origin_tz: None,
            dest_tz: None,
            flight_detail: None,
            rail_detail: None,
            boat_detail: None,
            transport_detail: Some(db::hops::TransportDetail {
                carrier_name: "Island Tours".into(),
                vehicle_description: "Minibus shuttle".into(),
                confirmation_num: "ISL-2024-0714".into(),
                notes: "Shared airport transfer to resort area".into(),
            }),
            cached_carrier: None,
            cost_amount: Some(35.00),
            cost_currency: Some("USD".into()),
            loyalty_program: None,
            miles_earned: None,
        },
        db::hops::Row {
            id: 0,
            travel_type: db::hops::TravelType::Boat,
            origin_name: "Castries Port, St. Lucia".into(),
            origin_lat: 14.0101,
            origin_lng: -60.9878,
            origin_country: Some("LC".into()),
            dest_name: "Bridgetown Harbour, Barbados".into(),
            dest_lat: 13.0969,
            dest_lng: -59.6145,
            dest_country: Some("BB".into()),
            start_date: "2024-07-18".into(),
            end_date: "2024-07-18".into(),
            raw_json: None,
            origin_address_query: None,
            dest_address_query: None,
            origin_tz: None,
            dest_tz: None,
            flight_detail: None,
            rail_detail: None,
            boat_detail: Some(db::hops::BoatDetail {
                ship_name: "L'Express des Iles".into(),
                cabin_type: "Economy".into(),
                cabin_number: "E45".into(),
                confirmation_num: "LEXI-2024-718".into(),
                booking_site: "express-des-iles.com".into(),
                notes: "Return crossing".into(),
            }),
            transport_detail: None,
            cached_carrier: None,
            cost_amount: Some(85.00),
            cost_currency: Some("USD".into()),
            loyalty_program: None,
            miles_earned: None,
        },
        db::hops::Row {
            id: 0,
            travel_type: db::hops::TravelType::Air,
            origin_name: "Grantley Adams Intl (BGI)".into(),
            origin_lat: 13.0747,
            origin_lng: -59.4925,
            origin_country: Some("BB".into()),
            dest_name: "London Gatwick (LGW)".into(),
            dest_lat: 51.1537,
            dest_lng: -0.1821,
            dest_country: Some("GB".into()),
            start_date: "2024-07-20".into(),
            end_date: "2024-07-21".into(),
            raw_json: None,
            origin_address_query: None,
            dest_address_query: None,
            origin_tz: None,
            dest_tz: None,
            flight_detail: Some(db::hops::FlightDetail {
                airline: "Virgin Atlantic".into(),
                flight_number: "VS148".into(),
                aircraft_type: "Airbus A330-300".into(),
                cabin_class: "Economy".into(),
                seat: "31D".into(),
                pnr: "VS148JUL".into(),
            }),
            rail_detail: None,
            boat_detail: None,
            transport_detail: None,
            cached_carrier: None,
            cost_amount: Some(580.00),
            cost_currency: Some("GBP".into()),
            loyalty_program: Some("Virgin Points".into()),
            miles_earned: Some(4220.0),
        },
        db::hops::Row {
            id: 0,
            travel_type: db::hops::TravelType::Air,
            origin_name: "London Gatwick (LGW)".into(),
            origin_lat: 51.1537,
            origin_lng: -0.1821,
            origin_country: Some("GB".into()),
            dest_name: "Dublin Airport (DUB)".into(),
            dest_lat: 53.4264,
            dest_lng: -6.2499,
            dest_country: Some("IE".into()),
            start_date: "2024-07-21".into(),
            end_date: "2024-07-21".into(),
            raw_json: None,
            origin_address_query: None,
            dest_address_query: None,
            origin_tz: None,
            dest_tz: None,
            flight_detail: Some(db::hops::FlightDetail {
                airline: "Ryanair".into(),
                flight_number: "FR115".into(),
                aircraft_type: "Boeing 737-800".into(),
                cabin_class: "Economy".into(),
                seat: "15A".into(),
                pnr: "RYR115JUL".into(),
            }),
            rail_detail: None,
            boat_detail: None,
            transport_detail: None,
            cached_carrier: None,
            cost_amount: Some(52.00),
            cost_currency: Some("EUR".into()),
            loyalty_program: None,
            miles_earned: None,
        },
    ]
}

const HOP_TRIP_ASSIGNMENTS: &[usize] = &[
    0, 0, 0, 0, // Europe Winter 2024 — 4 hops
    1, 1, 1, 1, 1, 1, // Japan Spring 2024 — 6 hops
    2, 2, 2, 2, 2, 2, 2, 2, // Caribbean Summer 2024 — 8 hops
];

async fn seed_hops(pool: &SqlitePool, user_id: i64, trip_ids: &[i64]) -> Result<(), SeedError> {
    let hops = build_seed_hops();

    // Insert all hops in one batch using a seed-specific scoped trip_id.
    let inserted = (db::hops::Create {
        trip_id: "seed-data",
        user_id,
        hops: &hops,
    })
    .execute(pool)
    .await?;

    tracing::info!(user_id, count = inserted, "inserted seed hops");

    // Fetch all hops back to get their DB-assigned IDs (ordered by start_date).
    let all_hops = (db::hops::GetAll {
        user_id,
        travel_type_filter: None,
    })
    .execute(pool)
    .await?;

    // Assign each hop to its trip.
    for (hop, &trip_idx) in all_hops.iter().zip(HOP_TRIP_ASSIGNMENTS) {
        if trip_idx < trip_ids.len() {
            (db::trips::AssignHop {
                hop_id: hop.id,
                trip_id: trip_ids[trip_idx],
                user_id,
            })
            .execute(pool)
            .await?;
        }
    }

    tracing::info!(user_id, "assigned hops to trips");
    Ok(())
}

async fn run() -> Result<(), SeedError> {
    let cli = Cli::parse();
    let pool = travel_mapper::db::create_pool(&cli.database_url).await?;

    let encryption_key = cli
        .encryption_key
        .as_deref()
        .map(parse_encryption_key)
        .transpose()?;

    let tripit_creds: Option<(&str, &str)> =
        match (&cli.tripit_access_token, &cli.tripit_access_token_secret) {
            (Some(token), Some(secret)) => Some((token.as_str(), secret.as_str())),
            (None, None) => None,
            _ => return Err(SeedError::IncompleteCredentials),
        };

    let username = "test";
    let hash = travel_mapper::auth::hash_password("test")?;

    let user_id = match (db::users::Create {
        username,
        password_hash: &hash,
        email: "test@example.com",
        first_name: "Test",
        last_name: "Traveller",
    })
    .execute(&pool)
    .await
    {
        Ok(id) => {
            tracing::info!(username, id, "created user");
            id
        }
        Err(sqlx::Error::Database(err)) if err.is_unique_violation() => {
            tracing::info!(username, "user already exists, skipping creation");
            let user = (db::users::GetByUsername { username })
                .execute(&pool)
                .await?
                .ok_or_else(|| SeedError::UserNotFound(username.to_owned()))?;
            user.id
        }
        Err(err) => return Err(SeedError::Database(err)),
    };

    let trip_ids = seed_trips(&pool, user_id).await?;
    seed_hops(&pool, user_id, &trip_ids).await?;

    if let (Some((access_token, access_token_secret)), Some(key)) = (tripit_creds, &encryption_key)
    {
        seed_tripit_credentials(
            &pool,
            user_id,
            username,
            access_token,
            access_token_secret,
            key,
        )
        .await?;
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    travel_mapper::telemetry::init();

    if let Err(error) = run().await {
        tracing::error!(%error, "seed failed");
        std::process::exit(1);
    }
}

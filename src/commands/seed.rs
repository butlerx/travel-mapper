use crate::auth::{CryptoError, parse_encryption_key};
use crate::db;
use clap::Args as ClapArgs;
use sqlx::SqlitePool;

#[derive(ClapArgs)]
pub struct Args {
    #[arg(long, env = "DATABASE_URL", default_value = "sqlite:travel.db")]
    database_url: String,

    #[arg(long, env = "ENCRYPTION_KEY")]
    encryption_key: Option<String>,

    #[arg(long, env = "TRIPIT_ACCESS_TOKEN")]
    tripit_access_token: Option<String>,

    #[arg(long, env = "TRIPIT_ACCESS_TOKEN_SECRET")]
    tripit_access_token_secret: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Crypto(#[from] CryptoError),

    #[error("{0}")]
    Database(#[from] sqlx::Error),

    #[error("failed to hash password: {0}")]
    HashPassword(argon2::password_hash::Error),

    #[error("TRIPIT_ACCESS_TOKEN and TRIPIT_ACCESS_TOKEN_SECRET must both be set or both be unset")]
    IncompleteCredentials,

    #[error("user {0:?} not found after unique violation")]
    UserNotFound(String),
}

impl From<argon2::password_hash::Error> for Error {
    fn from(err: argon2::password_hash::Error) -> Self {
        Self::HashPassword(err)
    }
}

async fn seed_tripit_credentials(
    pool: &SqlitePool,
    user_id: i64,
    username: &str,
    access_token: &str,
    access_token_secret: &str,
    encryption_key: &[u8; 32],
) -> Result<(), Error> {
    let (token_enc, nonce_token) = crate::auth::encrypt_token(access_token, encryption_key)?;
    let (secret_enc, nonce_secret) =
        crate::auth::encrypt_token(access_token_secret, encryption_key)?;

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

async fn seed_trips(pool: &SqlitePool, user_id: i64) -> Result<Vec<i64>, Error> {
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

struct HopDef {
    travel_type: db::hops::TravelType,
    origin: (&'static str, f64, f64, &'static str),
    dest: (&'static str, f64, f64, &'static str),
    dates: (&'static str, &'static str),
    flight: Option<db::hops::FlightDetail>,
    rail: Option<db::hops::RailDetail>,
    boat: Option<db::hops::BoatDetail>,
    transport: Option<db::hops::TransportDetail>,
    cost: Option<(f64, &'static str)>,
    loyalty: Option<(&'static str, f64)>,
}

impl Default for HopDef {
    fn default() -> Self {
        Self {
            travel_type: db::hops::TravelType::Air,
            origin: ("", 0.0, 0.0, ""),
            dest: ("", 0.0, 0.0, ""),
            dates: ("", ""),
            flight: None,
            rail: None,
            boat: None,
            transport: None,
            cost: None,
            loyalty: None,
        }
    }
}

impl From<HopDef> for db::hops::Row {
    fn from(h: HopDef) -> Self {
        Self {
            id: 0,
            travel_type: h.travel_type,
            origin_name: h.origin.0.into(),
            origin_lat: h.origin.1,
            origin_lng: h.origin.2,
            origin_country: Some(h.origin.3.into()),
            dest_name: h.dest.0.into(),
            dest_lat: h.dest.1,
            dest_lng: h.dest.2,
            dest_country: Some(h.dest.3.into()),
            start_date: h.dates.0.into(),
            end_date: h.dates.1.into(),
            raw_json: None,
            origin_address_query: None,
            dest_address_query: None,
            origin_tz: None,
            dest_tz: None,
            flight_detail: h.flight,
            rail_detail: h.rail,
            boat_detail: h.boat,
            transport_detail: h.transport,
            cached_carrier: None,
            cost_amount: h.cost.map(|(amount, _)| amount),
            cost_currency: h.cost.map(|(_, currency)| currency.into()),
            loyalty_program: h.loyalty.map(|(program, _)| program.into()),
            miles_earned: h.loyalty.map(|(_, miles)| miles),
        }
    }
}

fn europe_winter_hops() -> Vec<HopDef> {
    vec![
        HopDef {
            origin: ("Dublin Airport (DUB)", 53.4264, -6.2499, "IE"),
            dest: ("London Heathrow (LHR)", 51.4700, -0.4543, "GB"),
            dates: ("2024-01-15", "2024-01-15"),
            flight: Some(db::hops::FlightDetail {
                airline: "Aer Lingus".into(),
                flight_number: "EI154".into(),
                aircraft_type: "Airbus A320".into(),
                cabin_class: "Economy".into(),
                seat: "12A".into(),
                pnr: "ABC123".into(),
            }),
            cost: Some((89.99, "EUR")),
            loyalty: Some(("AerClub", 450.0)),
            ..HopDef::default()
        },
        HopDef {
            travel_type: db::hops::TravelType::Rail,
            origin: ("London St Pancras", 51.5317, -0.1262, "GB"),
            dest: ("Paris Gare du Nord", 48.8809, 2.3553, "FR"),
            dates: ("2024-01-16", "2024-01-16"),
            rail: Some(db::hops::RailDetail {
                carrier: "Eurostar".into(),
                train_number: "ES9024".into(),
                service_class: "Standard Premier".into(),
                coach_number: "7".into(),
                seats: "42".into(),
                confirmation_num: "EUR-778899".into(),
                booking_site: "eurostar.com".into(),
                notes: "Booked early for cheaper fare".into(),
            }),
            cost: Some((150.00, "GBP")),
            ..HopDef::default()
        },
        HopDef {
            travel_type: db::hops::TravelType::Transport,
            origin: ("Paris Gare du Nord", 48.8809, 2.3553, "FR"),
            dest: ("Le Marais, Paris", 48.8566, 2.3522, "FR"),
            dates: ("2024-01-16", "2024-01-16"),
            transport: Some(db::hops::TransportDetail {
                carrier_name: "G7 Taxis".into(),
                vehicle_description: "Taxi sedan".into(),
                confirmation_num: "G7-2024-0116".into(),
                notes: "Flat rate from Gare du Nord".into(),
            }),
            cost: Some((25.00, "EUR")),
            ..HopDef::default()
        },
        HopDef {
            origin: ("Paris Charles de Gaulle (CDG)", 49.0097, 2.5479, "FR"),
            dest: ("Dublin Airport (DUB)", 53.4264, -6.2499, "IE"),
            dates: ("2024-01-20", "2024-01-20"),
            flight: Some(db::hops::FlightDetail {
                airline: "Air France".into(),
                flight_number: "AF1116".into(),
                aircraft_type: "Airbus A319".into(),
                cabin_class: "Economy".into(),
                seat: "23F".into(),
                pnr: "XYZAF1".into(),
            }),
            cost: Some((112.50, "EUR")),
            loyalty: Some(("Flying Blue", 500.0)),
            ..HopDef::default()
        },
    ]
}

fn japan_outbound_hops() -> Vec<HopDef> {
    vec![
        HopDef {
            origin: ("Dublin Airport (DUB)", 53.4264, -6.2499, "IE"),
            dest: ("London Heathrow (LHR)", 51.4700, -0.4543, "GB"),
            dates: ("2024-04-01", "2024-04-01"),
            flight: Some(db::hops::FlightDetail {
                airline: "British Airways".into(),
                flight_number: "BA835".into(),
                aircraft_type: "Airbus A320neo".into(),
                cabin_class: "Economy".into(),
                seat: "18C".into(),
                pnr: "BA2024APR".into(),
            }),
            cost: Some((78.00, "GBP")),
            loyalty: Some(("Avios", 280.0)),
            ..HopDef::default()
        },
        HopDef {
            origin: ("London Heathrow (LHR)", 51.4700, -0.4543, "GB"),
            dest: ("Tokyo Narita (NRT)", 35.7647, 140.3864, "JP"),
            dates: ("2024-04-01", "2024-04-02"),
            flight: Some(db::hops::FlightDetail {
                airline: "Japan Airlines".into(),
                flight_number: "JL44".into(),
                aircraft_type: "Boeing 787-9".into(),
                cabin_class: "Premium Economy".into(),
                seat: "28K".into(),
                pnr: "JL44APR01".into(),
            }),
            cost: Some((1250.00, "GBP")),
            loyalty: Some(("JAL Mileage Bank", 5974.0)),
            ..HopDef::default()
        },
        HopDef {
            travel_type: db::hops::TravelType::Rail,
            origin: ("Tokyo Station", 35.6812, 139.7671, "JP"),
            dest: ("Kyoto Station", 34.9857, 135.7589, "JP"),
            dates: ("2024-04-03", "2024-04-03"),
            rail: Some(db::hops::RailDetail {
                carrier: "JR Central".into(),
                train_number: "Nozomi 225".into(),
                service_class: "Green Car".into(),
                coach_number: "8".into(),
                seats: "3A".into(),
                confirmation_num: "JR-2024-0403".into(),
                booking_site: "smartex.jrcentral.co.jp".into(),
                notes: "Japan Rail Pass not valid for Nozomi".into(),
            }),
            cost: Some((13320.0, "JPY")),
            ..HopDef::default()
        },
    ]
}

fn japan_return_hops() -> Vec<HopDef> {
    vec![
        HopDef {
            travel_type: db::hops::TravelType::Rail,
            origin: ("Kyoto Station", 34.9857, 135.7589, "JP"),
            dest: ("Osaka Station", 34.7024, 135.4959, "JP"),
            dates: ("2024-04-07", "2024-04-07"),
            rail: Some(db::hops::RailDetail {
                carrier: "JR West".into(),
                train_number: "Special Rapid".into(),
                service_class: "Reserved".into(),
                coach_number: "4".into(),
                seats: "12D".into(),
                confirmation_num: "JR-2024-0407".into(),
                booking_site: "jr-odekake.net".into(),
                notes: String::new(),
            }),
            cost: Some((580.0, "JPY")),
            ..HopDef::default()
        },
        HopDef {
            origin: ("Osaka Kansai (KIX)", 34.4347, 135.2441, "JP"),
            dest: ("Helsinki Vantaa (HEL)", 60.3172, 24.9633, "FI"),
            dates: ("2024-04-10", "2024-04-10"),
            flight: Some(db::hops::FlightDetail {
                airline: "Finnair".into(),
                flight_number: "AY78".into(),
                aircraft_type: "Airbus A350-900".into(),
                cabin_class: "Economy".into(),
                seat: "35A".into(),
                pnr: "FINAY78".into(),
            }),
            cost: Some((680.00, "EUR")),
            loyalty: Some(("Finnair Plus", 4890.0)),
            ..HopDef::default()
        },
        HopDef {
            origin: ("Helsinki Vantaa (HEL)", 60.3172, 24.9633, "FI"),
            dest: ("Dublin Airport (DUB)", 53.4264, -6.2499, "IE"),
            dates: ("2024-04-10", "2024-04-10"),
            flight: Some(db::hops::FlightDetail {
                airline: "Finnair".into(),
                flight_number: "AY939".into(),
                aircraft_type: "Airbus A321".into(),
                cabin_class: "Economy".into(),
                seat: "7F".into(),
                pnr: "FINAY939".into(),
            }),
            loyalty: Some(("Finnair Plus", 1820.0)),
            ..HopDef::default()
        },
    ]
}

fn caribbean_outbound_hops() -> Vec<HopDef> {
    vec![
        HopDef {
            origin: ("Dublin Airport (DUB)", 53.4264, -6.2499, "IE"),
            dest: ("London Gatwick (LGW)", 51.1537, -0.1821, "GB"),
            dates: ("2024-07-10", "2024-07-10"),
            flight: Some(db::hops::FlightDetail {
                airline: "Ryanair".into(),
                flight_number: "FR114".into(),
                aircraft_type: "Boeing 737-800".into(),
                cabin_class: "Economy".into(),
                seat: "6C".into(),
                pnr: "RYR114JUL".into(),
            }),
            cost: Some((45.00, "EUR")),
            ..HopDef::default()
        },
        HopDef {
            origin: ("London Gatwick (LGW)", 51.1537, -0.1821, "GB"),
            dest: ("Grantley Adams Intl (BGI)", 13.0747, -59.4925, "BB"),
            dates: ("2024-07-10", "2024-07-10"),
            flight: Some(db::hops::FlightDetail {
                airline: "Virgin Atlantic".into(),
                flight_number: "VS147".into(),
                aircraft_type: "Airbus A330-300".into(),
                cabin_class: "Premium".into(),
                seat: "14A".into(),
                pnr: "VS147JUL".into(),
            }),
            cost: Some((650.00, "GBP")),
            loyalty: Some(("Virgin Points", 4220.0)),
            ..HopDef::default()
        },
        HopDef {
            travel_type: db::hops::TravelType::Boat,
            origin: ("Bridgetown Harbour, Barbados", 13.0969, -59.6145, "BB"),
            dest: ("Castries Port, St. Lucia", 14.0101, -60.9878, "LC"),
            dates: ("2024-07-14", "2024-07-14"),
            boat: Some(db::hops::BoatDetail {
                ship_name: "L'Express des Iles".into(),
                cabin_type: "Business".into(),
                cabin_number: "B12".into(),
                confirmation_num: "LEXI-2024-714".into(),
                booking_site: "express-des-iles.com".into(),
                notes: "Inter-island high-speed ferry".into(),
            }),
            cost: Some((95.00, "USD")),
            ..HopDef::default()
        },
        HopDef {
            travel_type: db::hops::TravelType::Transport,
            origin: ("Castries Port, St. Lucia", 14.0101, -60.9878, "LC"),
            dest: ("Rodney Bay, St. Lucia", 14.0722, -60.9524, "LC"),
            dates: ("2024-07-14", "2024-07-14"),
            transport: Some(db::hops::TransportDetail {
                carrier_name: "Island Tours".into(),
                vehicle_description: "Minibus shuttle".into(),
                confirmation_num: "ISL-2024-0714".into(),
                notes: "Shared airport transfer to resort area".into(),
            }),
            cost: Some((35.00, "USD")),
            ..HopDef::default()
        },
    ]
}

fn caribbean_return_hops() -> Vec<HopDef> {
    vec![
        HopDef {
            travel_type: db::hops::TravelType::Boat,
            origin: ("Castries Port, St. Lucia", 14.0101, -60.9878, "LC"),
            dest: ("Bridgetown Harbour, Barbados", 13.0969, -59.6145, "BB"),
            dates: ("2024-07-18", "2024-07-18"),
            boat: Some(db::hops::BoatDetail {
                ship_name: "L'Express des Iles".into(),
                cabin_type: "Economy".into(),
                cabin_number: "E45".into(),
                confirmation_num: "LEXI-2024-718".into(),
                booking_site: "express-des-iles.com".into(),
                notes: "Return crossing".into(),
            }),
            cost: Some((85.00, "USD")),
            ..HopDef::default()
        },
        HopDef {
            origin: ("Grantley Adams Intl (BGI)", 13.0747, -59.4925, "BB"),
            dest: ("London Gatwick (LGW)", 51.1537, -0.1821, "GB"),
            dates: ("2024-07-20", "2024-07-21"),
            flight: Some(db::hops::FlightDetail {
                airline: "Virgin Atlantic".into(),
                flight_number: "VS148".into(),
                aircraft_type: "Airbus A330-300".into(),
                cabin_class: "Economy".into(),
                seat: "31D".into(),
                pnr: "VS148JUL".into(),
            }),
            cost: Some((580.00, "GBP")),
            loyalty: Some(("Virgin Points", 4220.0)),
            ..HopDef::default()
        },
        HopDef {
            origin: ("London Gatwick (LGW)", 51.1537, -0.1821, "GB"),
            dest: ("Dublin Airport (DUB)", 53.4264, -6.2499, "IE"),
            dates: ("2024-07-21", "2024-07-21"),
            flight: Some(db::hops::FlightDetail {
                airline: "Ryanair".into(),
                flight_number: "FR115".into(),
                aircraft_type: "Boeing 737-800".into(),
                cabin_class: "Economy".into(),
                seat: "15A".into(),
                pnr: "RYR115JUL".into(),
            }),
            cost: Some((52.00, "EUR")),
            ..HopDef::default()
        },
    ]
}

fn build_seed_hops() -> Vec<db::hops::Row> {
    let mut defs = europe_winter_hops();
    defs.extend(japan_outbound_hops());
    defs.extend(japan_return_hops());
    defs.extend(caribbean_outbound_hops());
    defs.extend(caribbean_return_hops());
    defs.into_iter().map(Into::into).collect()
}

const HOP_TRIP_ASSIGNMENTS: &[usize] = &[
    0, 0, 0, 0, // Europe Winter 2024 — 4 hops
    1, 1, 1, 1, 1, 1, // Japan Spring 2024 — 6 hops
    2, 2, 2, 2, 2, 2, 2, 2, // Caribbean Summer 2024 — 8 hops
];

async fn seed_hops(pool: &SqlitePool, user_id: i64, trip_ids: &[i64]) -> Result<(), Error> {
    let hops = build_seed_hops();

    let inserted = (db::hops::Create {
        trip_id: "seed-data",
        user_id,
        hops: &hops,
    })
    .execute(pool)
    .await?;

    tracing::info!(user_id, count = inserted, "inserted seed hops");

    let all_hops = (db::hops::GetAll {
        user_id,
        travel_type_filter: None,
    })
    .execute(pool)
    .await?;

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

/// Seed the database with a test user and sample travel data.
///
/// # Errors
///
/// Returns an error if database access, encryption, or data insertion fails.
pub async fn run(args: Args) -> Result<(), Error> {
    let pool = crate::db::create_pool(&args.database_url).await?;

    let encryption_key = args
        .encryption_key
        .as_deref()
        .map(parse_encryption_key)
        .transpose()?;

    let tripit_creds: Option<(&str, &str)> =
        match (&args.tripit_access_token, &args.tripit_access_token_secret) {
            (Some(token), Some(secret)) => Some((token.as_str(), secret.as_str())),
            (None, None) => None,
            _ => return Err(Error::IncompleteCredentials),
        };

    let username = "test";
    let hash = crate::auth::hash_password("test")?;

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
                .ok_or_else(|| Error::UserNotFound(username.to_owned()))?;
            user.id
        }
        Err(err) => return Err(Error::Database(err)),
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

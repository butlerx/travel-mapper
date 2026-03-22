-- Recreate hops with non-nullable coordinates, country columns, and
-- cross-source dedup.  Unique key uses (user_id, ...) instead of
-- (trip_id, ...) so the same flight imported from TripIt and Flighty
-- deduplicates automatically.
CREATE TABLE IF NOT EXISTS hops_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    trip_id TEXT NOT NULL,
    travel_type TEXT NOT NULL,
    origin_name TEXT NOT NULL,
    origin_lat REAL NOT NULL DEFAULT 0.0,
    origin_lng REAL NOT NULL DEFAULT 0.0,
    origin_country TEXT,
    dest_name TEXT NOT NULL,
    dest_lat REAL NOT NULL DEFAULT 0.0,
    dest_lng REAL NOT NULL DEFAULT 0.0,
    dest_country TEXT,
    start_date TEXT NOT NULL,
    end_date TEXT NOT NULL,
    raw_json TEXT,
    user_id INTEGER REFERENCES users(id) ON DELETE CASCADE,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(user_id, travel_type, origin_name, dest_name, start_date)
);
INSERT OR IGNORE INTO hops_new (
    trip_id, travel_type, origin_name, origin_lat, origin_lng,
    dest_name, dest_lat, dest_lng, start_date, end_date,
    raw_json, user_id, created_at, updated_at
)
SELECT trip_id, travel_type, origin_name, COALESCE(origin_lat, 0.0), COALESCE(origin_lng, 0.0),
       dest_name, COALESCE(dest_lat, 0.0), COALESCE(dest_lng, 0.0), start_date, end_date,
       raw_json, user_id, created_at, updated_at
FROM hops
ORDER BY id;
DROP TABLE hops;
ALTER TABLE hops_new RENAME TO hops;
CREATE INDEX IF NOT EXISTS idx_hops_user_id ON hops(user_id);

CREATE TABLE IF NOT EXISTS flight_details (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    hop_id INTEGER NOT NULL UNIQUE REFERENCES hops(id) ON DELETE CASCADE,

    airline TEXT NOT NULL DEFAULT '',
    flight_number TEXT NOT NULL DEFAULT '',

    dep_terminal TEXT NOT NULL DEFAULT '',
    dep_gate TEXT NOT NULL DEFAULT '',
    arr_terminal TEXT NOT NULL DEFAULT '',
    arr_gate TEXT NOT NULL DEFAULT '',

    canceled INTEGER NOT NULL DEFAULT 0,
    diverted_to TEXT NOT NULL DEFAULT '',

    gate_dep_scheduled TEXT NOT NULL DEFAULT '',
    gate_dep_actual TEXT NOT NULL DEFAULT '',
    takeoff_scheduled TEXT NOT NULL DEFAULT '',
    takeoff_actual TEXT NOT NULL DEFAULT '',
    landing_scheduled TEXT NOT NULL DEFAULT '',
    landing_actual TEXT NOT NULL DEFAULT '',
    gate_arr_scheduled TEXT NOT NULL DEFAULT '',
    gate_arr_actual TEXT NOT NULL DEFAULT '',

    aircraft_type TEXT NOT NULL DEFAULT '',
    tail_number TEXT NOT NULL DEFAULT '',

    pnr TEXT NOT NULL DEFAULT '',
    seat TEXT NOT NULL DEFAULT '',
    seat_type TEXT NOT NULL DEFAULT '',
    cabin_class TEXT NOT NULL DEFAULT '',

    flight_reason TEXT NOT NULL DEFAULT '',
    notes TEXT NOT NULL DEFAULT '',

    airline_id TEXT NOT NULL DEFAULT '',
    dep_airport_id TEXT NOT NULL DEFAULT '',
    arr_airport_id TEXT NOT NULL DEFAULT '',
    diverted_airport_id TEXT NOT NULL DEFAULT '',
    aircraft_type_id TEXT NOT NULL DEFAULT ''
);
CREATE INDEX IF NOT EXISTS idx_flight_details_hop_id ON flight_details(hop_id);

CREATE TABLE IF NOT EXISTS rail_details (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    hop_id INTEGER NOT NULL UNIQUE REFERENCES hops(id) ON DELETE CASCADE,

    carrier TEXT NOT NULL DEFAULT '',
    train_number TEXT NOT NULL DEFAULT '',
    service_class TEXT NOT NULL DEFAULT '',
    coach_number TEXT NOT NULL DEFAULT '',
    seats TEXT NOT NULL DEFAULT '',

    confirmation_num TEXT NOT NULL DEFAULT '',
    booking_site TEXT NOT NULL DEFAULT '',
    notes TEXT NOT NULL DEFAULT ''
);
CREATE INDEX IF NOT EXISTS idx_rail_details_hop_id ON rail_details(hop_id);

CREATE TABLE IF NOT EXISTS cruise_details (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    hop_id INTEGER NOT NULL UNIQUE REFERENCES hops(id) ON DELETE CASCADE,

    ship_name TEXT NOT NULL DEFAULT '',
    cabin_type TEXT NOT NULL DEFAULT '',
    cabin_number TEXT NOT NULL DEFAULT '',

    confirmation_num TEXT NOT NULL DEFAULT '',
    booking_site TEXT NOT NULL DEFAULT '',
    notes TEXT NOT NULL DEFAULT ''
);
CREATE INDEX IF NOT EXISTS idx_cruise_details_hop_id ON cruise_details(hop_id);

CREATE TABLE IF NOT EXISTS transport_details (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    hop_id INTEGER NOT NULL UNIQUE REFERENCES hops(id) ON DELETE CASCADE,

    carrier_name TEXT NOT NULL DEFAULT '',
    vehicle_description TEXT NOT NULL DEFAULT '',

    confirmation_num TEXT NOT NULL DEFAULT '',
    notes TEXT NOT NULL DEFAULT ''
);
CREATE INDEX IF NOT EXISTS idx_transport_details_hop_id ON transport_details(hop_id);

-- Cache of Nominatim geocoding results to avoid repeated API calls.
CREATE TABLE IF NOT EXISTS geocode_cache (
    query      TEXT PRIMARY KEY,
    lat        REAL NOT NULL,
    lng        REAL NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

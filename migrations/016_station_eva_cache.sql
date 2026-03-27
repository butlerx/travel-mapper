-- Cache of German station EVA numbers from DB RIS API responses.
CREATE TABLE IF NOT EXISTS station_eva_cache (
    eva_number INTEGER PRIMARY KEY,
    name       TEXT NOT NULL,
    lat        REAL,
    lng        REAL,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX idx_station_eva_name ON station_eva_cache(name);

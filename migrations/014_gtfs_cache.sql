-- Static GTFS feed caching for Transitland rail status enrichment.
-- Data is sourced from Transitland feeds, parsed with gtfs-structures, and cached for 24 hours.

-- Feed metadata: tracks downloaded feed versions and refresh times
CREATE TABLE IF NOT EXISTS gtfs_feeds (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    onestop_id      TEXT    NOT NULL UNIQUE,  -- Transitland feed ID (e.g., f-u0-sncf~transilien~rer)
    feed_version    TEXT    NOT NULL,          -- Version string from feed_info.txt or download timestamp
    downloaded_at   TEXT    NOT NULL DEFAULT (datetime('now')),
    expires_at      TEXT    NOT NULL,          -- 24-hour TTL
    static_url      TEXT    NOT NULL           -- Original download URL for refresh
);

CREATE INDEX idx_gtfs_feeds_onestop_id ON gtfs_feeds(onestop_id);
CREATE INDEX idx_gtfs_feeds_expires_at ON gtfs_feeds(expires_at);

-- Stops: station/platform locations with fuzzy-match support
CREATE TABLE IF NOT EXISTS gtfs_stops (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    feed_id         INTEGER NOT NULL REFERENCES gtfs_feeds(id) ON DELETE CASCADE,
    stop_id         TEXT    NOT NULL,          -- GTFS stop_id
    stop_name       TEXT    NOT NULL,          -- Original name
    stop_name_normalized TEXT NOT NULL,        -- Lowercase, diacritics removed, for fuzzy matching
    lat             REAL    NOT NULL,
    lng             REAL    NOT NULL,
    UNIQUE(feed_id, stop_id)
);

CREATE INDEX idx_gtfs_stops_feed_id ON gtfs_stops(feed_id);
CREATE INDEX idx_gtfs_stops_stop_id ON gtfs_stops(feed_id, stop_id);
CREATE INDEX idx_gtfs_stops_normalized ON gtfs_stops(feed_id, stop_name_normalized);

-- Trips: service patterns with route and calendar info
CREATE TABLE IF NOT EXISTS gtfs_trips (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    feed_id         INTEGER NOT NULL REFERENCES gtfs_feeds(id) ON DELETE CASCADE,
    trip_id         TEXT    NOT NULL,
    route_id        TEXT    NOT NULL,
    service_id      TEXT    NOT NULL,          -- Links to calendar or calendar_dates
    UNIQUE(feed_id, trip_id)
);

CREATE INDEX idx_gtfs_trips_feed_id ON gtfs_trips(feed_id);
CREATE INDEX idx_gtfs_trips_trip_id ON gtfs_trips(feed_id, trip_id);
CREATE INDEX idx_gtfs_trips_route_id ON gtfs_trips(feed_id, route_id);

-- Stop times: when each trip visits each stop
CREATE TABLE IF NOT EXISTS gtfs_stop_times (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    feed_id         INTEGER NOT NULL REFERENCES gtfs_feeds(id) ON DELETE CASCADE,
    trip_id         TEXT    NOT NULL,
    stop_id         TEXT    NOT NULL,
    stop_sequence   INTEGER NOT NULL,
    departure_time  TEXT    NOT NULL,          -- HH:MM:SS format (can be >24:00:00 for next-day service)
    arrival_time    TEXT    NOT NULL,          -- HH:MM:SS format
    FOREIGN KEY (feed_id, trip_id) REFERENCES gtfs_trips(feed_id, trip_id) ON DELETE CASCADE,
    FOREIGN KEY (feed_id, stop_id) REFERENCES gtfs_stops(feed_id, stop_id) ON DELETE CASCADE
);

CREATE INDEX idx_gtfs_stop_times_feed_trip ON gtfs_stop_times(feed_id, trip_id);
CREATE INDEX idx_gtfs_stop_times_feed_stop ON gtfs_stop_times(feed_id, stop_id);
CREATE INDEX idx_gtfs_stop_times_departure ON gtfs_stop_times(feed_id, trip_id, departure_time);

-- Calendar: service patterns by weekday
CREATE TABLE IF NOT EXISTS gtfs_calendar (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    feed_id         INTEGER NOT NULL REFERENCES gtfs_feeds(id) ON DELETE CASCADE,
    service_id      TEXT    NOT NULL,
    monday          INTEGER NOT NULL DEFAULT 0,
    tuesday         INTEGER NOT NULL DEFAULT 0,
    wednesday       INTEGER NOT NULL DEFAULT 0,
    thursday        INTEGER NOT NULL DEFAULT 0,
    friday          INTEGER NOT NULL DEFAULT 0,
    saturday        INTEGER NOT NULL DEFAULT 0,
    sunday          INTEGER NOT NULL DEFAULT 0,
    start_date      TEXT    NOT NULL,          -- YYYYMMDD
    end_date        TEXT    NOT NULL,          -- YYYYMMDD
    UNIQUE(feed_id, service_id)
);

CREATE INDEX idx_gtfs_calendar_feed_service ON gtfs_calendar(feed_id, service_id);

-- Calendar dates: exceptions to regular calendar (holidays, special service)
CREATE TABLE IF NOT EXISTS gtfs_calendar_dates (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    feed_id         INTEGER NOT NULL REFERENCES gtfs_feeds(id) ON DELETE CASCADE,
    service_id      TEXT    NOT NULL,
    date            TEXT    NOT NULL,          -- YYYYMMDD
    exception_type  INTEGER NOT NULL,          -- 1 = service added, 2 = service removed
    FOREIGN KEY (feed_id, service_id) REFERENCES gtfs_calendar(feed_id, service_id) ON DELETE CASCADE
);

CREATE INDEX idx_gtfs_calendar_dates_feed_service ON gtfs_calendar_dates(feed_id, service_id);
CREATE INDEX idx_gtfs_calendar_dates_date ON gtfs_calendar_dates(feed_id, date);

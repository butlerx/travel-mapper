CREATE TABLE IF NOT EXISTS trips (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    start_date TEXT,
    end_date TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_trips_user_id ON trips(user_id);

ALTER TABLE hops ADD COLUMN user_trip_id INTEGER REFERENCES trips(id) ON DELETE SET NULL;
CREATE INDEX IF NOT EXISTS idx_hops_user_trip_id ON hops(user_trip_id);

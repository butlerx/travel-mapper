ALTER TABLE trips ADD COLUMN tripit_id TEXT;
CREATE UNIQUE INDEX IF NOT EXISTS idx_trips_user_tripit_id
    ON trips(user_id, tripit_id)
    WHERE tripit_id IS NOT NULL;

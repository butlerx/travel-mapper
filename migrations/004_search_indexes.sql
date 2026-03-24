-- Indexes for search & filter on flight_details columns.
CREATE INDEX IF NOT EXISTS idx_flight_details_airline ON flight_details (airline);
CREATE INDEX IF NOT EXISTS idx_flight_details_flight_number ON flight_details (flight_number);
CREATE INDEX IF NOT EXISTS idx_flight_details_cabin_class ON flight_details (cabin_class);
CREATE INDEX IF NOT EXISTS idx_flight_details_flight_reason ON flight_details (flight_reason);

-- Indexes for search across origin/destination names.
CREATE INDEX IF NOT EXISTS idx_hops_origin_name ON hops (origin_name);
CREATE INDEX IF NOT EXISTS idx_hops_dest_name ON hops (dest_name);

-- Index for date range filtering.
CREATE INDEX IF NOT EXISTS idx_hops_start_date ON hops (start_date);

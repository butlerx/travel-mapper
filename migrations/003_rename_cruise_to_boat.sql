UPDATE hops SET travel_type = 'boat' WHERE travel_type = 'cruise';

ALTER TABLE cruise_details RENAME TO boat_details;

-- SQLite keeps the old index name after ALTER TABLE RENAME; drop and recreate.
DROP INDEX IF EXISTS idx_cruise_details_hop_id;
CREATE INDEX IF NOT EXISTS idx_boat_details_hop_id ON boat_details(hop_id);

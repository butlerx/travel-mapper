-- Aircraft registration (tail number) captured from flight status enrichment.
-- This is the keying mechanism for inbound-aircraft delay prediction (Phase 1.3):
-- the same airframe's prior leg can be tracked by registration.
ALTER TABLE status_enrichments ADD COLUMN aircraft_reg TEXT NOT NULL DEFAULT '';

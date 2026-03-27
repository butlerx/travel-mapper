-- Add platform columns for rail status enrichment.
ALTER TABLE status_enrichments ADD COLUMN dep_platform TEXT NOT NULL DEFAULT '';
ALTER TABLE status_enrichments ADD COLUMN arr_platform TEXT NOT NULL DEFAULT '';

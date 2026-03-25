-- Flight/train status enrichment data from external providers.
CREATE TABLE IF NOT EXISTS status_enrichments (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    hop_id      INTEGER NOT NULL REFERENCES hops(id) ON DELETE CASCADE,
    provider    TEXT    NOT NULL DEFAULT 'aviationstack',
    status      TEXT    NOT NULL DEFAULT '',
    delay_minutes INTEGER,
    dep_gate    TEXT    NOT NULL DEFAULT '',
    dep_terminal TEXT   NOT NULL DEFAULT '',
    arr_gate    TEXT    NOT NULL DEFAULT '',
    arr_terminal TEXT   NOT NULL DEFAULT '',
    raw_json    TEXT    NOT NULL DEFAULT '{}',
    fetched_at  TEXT    NOT NULL DEFAULT (datetime('now')),
    UNIQUE(hop_id, provider)
);

CREATE INDEX idx_status_enrichments_hop_id ON status_enrichments(hop_id);

CREATE TABLE IF NOT EXISTS attachments (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    hop_id       INTEGER NOT NULL REFERENCES hops(id) ON DELETE CASCADE,
    user_id      INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    filename     TEXT    NOT NULL,
    content_type TEXT    NOT NULL,
    size_bytes   INTEGER NOT NULL,
    storage_path TEXT    NOT NULL,
    created_at   TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_attachments_hop_id  ON attachments(hop_id);
CREATE INDEX idx_attachments_user_id ON attachments(user_id);

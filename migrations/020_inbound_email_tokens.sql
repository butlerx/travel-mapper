CREATE TABLE IF NOT EXISTS inbound_email_tokens (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id    INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash TEXT    NOT NULL UNIQUE,
    label      TEXT    NOT NULL DEFAULT 'email import',
    created_at TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_inbound_email_tokens_user ON inbound_email_tokens(user_id);

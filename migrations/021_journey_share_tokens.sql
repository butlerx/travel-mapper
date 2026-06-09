CREATE TABLE journey_share_tokens (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id    INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    hop_id     INTEGER NOT NULL REFERENCES hops(id) ON DELETE CASCADE,
    token_hash TEXT    NOT NULL UNIQUE,
    expires_at TEXT    NOT NULL,
    created_at TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_journey_share_tokens_hop ON journey_share_tokens(hop_id);
CREATE INDEX idx_journey_share_tokens_user ON journey_share_tokens(user_id);

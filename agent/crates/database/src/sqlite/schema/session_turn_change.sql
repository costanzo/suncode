CREATE TABLE IF NOT EXISTS session_turn_change (
    turn_id TEXT NOT NULL CHECK(length(turn_id) > 0),
    relative_path TEXT NOT NULL CHECK(length(relative_path) > 0),
    session_id TEXT NOT NULL,
    ordinal INTEGER NOT NULL CHECK(ordinal >= 0),
    baseline_sha256 TEXT,
    baseline_stored_on TEXT,
    current_sha256 TEXT,
    current_stored_on TEXT,
    is_binary INTEGER NOT NULL DEFAULT 0 CHECK(is_binary IN (0, 1)),
    additions INTEGER NOT NULL DEFAULT 0 CHECK(additions >= 0),
    deletions INTEGER NOT NULL DEFAULT 0 CHECK(deletions >= 0),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    expires_at TEXT NOT NULL,
    PRIMARY KEY (turn_id, relative_path)
);

CREATE INDEX IF NOT EXISTS session_turn_change_session_idx
    ON session_turn_change (session_id, turn_id);
CREATE INDEX IF NOT EXISTS session_turn_change_expiry_idx
    ON session_turn_change (expires_at);
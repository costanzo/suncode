CREATE TABLE IF NOT EXISTS session_turn_todo (
    turn_id TEXT NOT NULL REFERENCES session_turn(turn_id) ON DELETE CASCADE,
    ordinal INTEGER NOT NULL CHECK(ordinal >= 0),
    content TEXT NOT NULL CHECK(length(content) > 0 AND length(content) <= 500),
    status TEXT NOT NULL CHECK(status IN ('pending', 'in_progress', 'completed', 'cancelled')),
    priority TEXT NOT NULL CHECK(priority IN ('high', 'medium', 'low')),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    completed_at TEXT,
    PRIMARY KEY(turn_id, ordinal)
);

CREATE INDEX IF NOT EXISTS session_turn_todo_turn_status_idx
    ON session_turn_todo(turn_id, status, ordinal);

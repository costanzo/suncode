CREATE TABLE IF NOT EXISTS session_message (
    message_id TEXT PRIMARY KEY CHECK(length(message_id) > 0),
    session_id TEXT NOT NULL,
    turn_id TEXT,
    session_call_id TEXT,
    role TEXT NOT NULL CHECK(role IN ('user', 'assistant', 'thinking')),
    message_json TEXT NOT NULL CHECK(json_valid(message_json)),
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS session_message_session_created_idx
    ON session_message(session_id, created_at, message_id);
CREATE INDEX IF NOT EXISTS session_message_call_idx
    ON session_message(session_call_id, created_at, message_id);

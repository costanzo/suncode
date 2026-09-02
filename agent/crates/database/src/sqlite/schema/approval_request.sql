CREATE TABLE IF NOT EXISTS approval_request (
    approval_id TEXT PRIMARY KEY CHECK(length(approval_id) > 0),
    project_id TEXT,
    session_id TEXT NOT NULL,
    turn_id TEXT NOT NULL,
    tool_call_id TEXT NOT NULL,
    operation TEXT NOT NULL CHECK(length(operation) > 0),
    arguments_json TEXT NOT NULL CHECK(json_valid(arguments_json)),
    idempotency_key TEXT NOT NULL UNIQUE CHECK(length(idempotency_key) > 0),
    status TEXT NOT NULL CHECK(status IN ('pending', 'approved', 'denied')),
    decision TEXT,
    decision_source TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS approval_request_session_status_idx
    ON approval_request(session_id, status, created_at);

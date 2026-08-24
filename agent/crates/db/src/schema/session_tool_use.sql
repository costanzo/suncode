CREATE TABLE IF NOT EXISTS session_tool_use (
    turn_id TEXT NOT NULL,
    tool_call_id TEXT NOT NULL CHECK(length(tool_call_id) > 0),
    session_call_id TEXT REFERENCES session_call(call_id) ON DELETE SET NULL,
    name TEXT NOT NULL CHECK(length(name) > 0),
    request_json TEXT CHECK(request_json IS NULL OR json_valid(request_json)),
    result_json TEXT CHECK(result_json IS NULL OR json_valid(result_json)),
    state TEXT NOT NULL CHECK(state IN (
        'requested', 'validating', 'policy_check', 'denied',
        'awaiting_approval', 'authorized', 'executing', 'succeeded',
        'failed', 'timed_out', 'unknown_completion', 'reconciling'
    )),
    ordinal INTEGER CHECK(ordinal IS NULL OR ordinal >= 0),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    completed_at TEXT,
    error_code TEXT,
    PRIMARY KEY(turn_id, tool_call_id)
);

CREATE INDEX IF NOT EXISTS session_tool_use_turn_state_idx
    ON session_tool_use(turn_id, state, created_at);
CREATE INDEX IF NOT EXISTS session_tool_use_call_idx
    ON session_tool_use(session_call_id, created_at);

CREATE TABLE IF NOT EXISTS session_call (
    call_id TEXT PRIMARY KEY CHECK(length(call_id) > 0),
    session_id TEXT NOT NULL REFERENCES session(session_id) ON DELETE CASCADE,
    turn_id TEXT NOT NULL REFERENCES session_turn(turn_id) ON DELETE CASCADE,
    provider TEXT NOT NULL CHECK(length(provider) > 0),
    model_id TEXT NOT NULL CHECK(length(model_id) > 0),
    wire_model TEXT NOT NULL CHECK(length(wire_model) > 0),
    provider_request_id TEXT,
    provider_response_id TEXT,
    state TEXT NOT NULL CHECK(state IN ('started', 'completed', 'failed')),
    iteration INTEGER NOT NULL CHECK(iteration > 0),
    started_at TEXT NOT NULL,
    completed_at TEXT,
    input_messages_json TEXT NOT NULL CHECK(json_valid(input_messages_json)),
    output_message_json TEXT CHECK(output_message_json IS NULL OR json_valid(output_message_json)),
    tool_calls_json TEXT NOT NULL CHECK(json_valid(tool_calls_json)),
    usage_json TEXT CHECK(usage_json IS NULL OR json_valid(usage_json)),
    finish_reason TEXT,
    error_json TEXT CHECK(error_json IS NULL OR json_valid(error_json))
);

CREATE INDEX IF NOT EXISTS session_call_session_started_idx
    ON session_call(session_id, started_at DESC, call_id);
CREATE INDEX IF NOT EXISTS session_call_turn_idx
    ON session_call(turn_id, started_at, call_id);
CREATE INDEX IF NOT EXISTS session_call_started_idx
    ON session_call(started_at, call_id)
    WHERE state = 'started';

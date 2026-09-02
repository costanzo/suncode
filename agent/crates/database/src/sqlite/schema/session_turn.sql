CREATE TABLE IF NOT EXISTS session_turn (
    turn_id TEXT PRIMARY KEY CHECK(length(turn_id) > 0),
    session_id TEXT NOT NULL,
    submission_idempotency_key TEXT,
    state TEXT NOT NULL CHECK(state IN (
        'admitted', 'queued', 'preparing', 'calling_model',
        'resolving_calls', 'compacting', 'completed', 'failed',
        'cancelled', 'interrupted'
    )),
    model_id TEXT,
    input_json TEXT CHECK(input_json IS NULL OR json_valid(input_json)),
    response_json TEXT CHECK(response_json IS NULL OR json_valid(response_json)),
    error_json TEXT CHECK(error_json IS NULL OR json_valid(error_json)),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    admitted_at TEXT,
    started_at TEXT,
    completed_at TEXT,
    error_code TEXT,
    input_tokens INTEGER NOT NULL DEFAULT 0 CHECK(input_tokens >= 0),
    output_tokens INTEGER NOT NULL DEFAULT 0 CHECK(output_tokens >= 0),
    total_tokens INTEGER NOT NULL DEFAULT 0 CHECK(total_tokens >= 0),
    recovery_approval_id TEXT,
    recovery_snapshot_json TEXT CHECK(
        recovery_snapshot_json IS NULL OR json_valid(recovery_snapshot_json)
    ),
    recovery_status TEXT CHECK(
        recovery_status IS NULL OR recovery_status IN ('pending', 'resuming', 'completed', 'denied', 'failed')
    ),
    recovery_created_at TEXT,
    recovery_updated_at TEXT,
    UNIQUE(session_id, submission_idempotency_key)
);

CREATE INDEX IF NOT EXISTS session_turn_session_created_idx
    ON session_turn(session_id, created_at DESC, turn_id);
CREATE INDEX IF NOT EXISTS session_turn_recovery_idx
    ON session_turn(updated_at, turn_id)
    WHERE state NOT IN ('completed', 'failed', 'cancelled', 'interrupted');
CREATE INDEX IF NOT EXISTS session_turn_resuming_idx
    ON session_turn(recovery_updated_at, turn_id)
    WHERE recovery_status = 'resuming';

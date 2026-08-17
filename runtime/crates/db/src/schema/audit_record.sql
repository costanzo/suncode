CREATE TABLE IF NOT EXISTS audit_record (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id TEXT,
    session_id TEXT,
    turn_id TEXT,
    occurred_at TEXT NOT NULL,
    event_type TEXT NOT NULL CHECK(length(event_type) > 0),
    payload_json TEXT NOT NULL CHECK(json_valid(payload_json))
);

CREATE TRIGGER IF NOT EXISTS audit_record_no_update
BEFORE UPDATE ON audit_record
BEGIN
    SELECT RAISE(ABORT, 'audit records are immutable');
END;

CREATE TRIGGER IF NOT EXISTS audit_record_no_delete
BEFORE DELETE ON audit_record
BEGIN
    SELECT RAISE(ABORT, 'audit records are immutable');
END;

CREATE INDEX IF NOT EXISTS audit_record_occurred_idx
    ON audit_record(occurred_at);
CREATE INDEX IF NOT EXISTS audit_record_project_time_idx
    ON audit_record(project_id, occurred_at);
CREATE INDEX IF NOT EXISTS audit_record_session_time_idx
    ON audit_record(session_id, occurred_at);
CREATE INDEX IF NOT EXISTS audit_record_turn_time_idx
    ON audit_record(turn_id, occurred_at);

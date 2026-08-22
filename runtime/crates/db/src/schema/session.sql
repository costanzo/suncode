CREATE TABLE IF NOT EXISTS session (
    session_id TEXT PRIMARY KEY CHECK(length(session_id) > 0),
    project_id TEXT NOT NULL REFERENCES project(project_id) ON DELETE RESTRICT,
    title TEXT,
    model_id TEXT,
    status TEXT NOT NULL CHECK(status IN ('active', 'archived')),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    last_activity_at TEXT NOT NULL,
    archived_at TEXT,
    CHECK(
        (status = 'active' AND archived_at IS NULL)
        OR (status = 'archived' AND archived_at IS NOT NULL)
    )
);

CREATE INDEX IF NOT EXISTS session_project_activity_idx
    ON session(project_id, status, last_activity_at DESC, session_id);

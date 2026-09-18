CREATE TABLE IF NOT EXISTS session (
    session_id TEXT PRIMARY KEY CHECK(length(session_id) > 0),
    project_id TEXT NOT NULL,
    title TEXT,
    model_id TEXT,
    reasoning_effort TEXT,
    kind TEXT NOT NULL DEFAULT 'primary' CHECK(kind IN ('primary', 'child')),
    parent_session_id TEXT,
    agent_id TEXT,
    agent_version INTEGER,
    status TEXT NOT NULL CHECK(status IN ('active', 'archived')),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    last_activity_at TEXT NOT NULL,
    pin_at TEXT,
    archived_at TEXT,
    CHECK(
        (status = 'active' AND archived_at IS NULL)
        OR (status = 'archived' AND archived_at IS NOT NULL)
    ),
    CHECK(
        (kind = 'primary' AND parent_session_id IS NULL AND agent_id IS NULL AND agent_version IS NULL)
        OR (kind = 'child' AND parent_session_id IS NOT NULL AND agent_id IS NOT NULL AND agent_version IS NOT NULL)
    )
);

CREATE INDEX IF NOT EXISTS session_project_activity_idx
    ON session(project_id, status, last_activity_at DESC, session_id);

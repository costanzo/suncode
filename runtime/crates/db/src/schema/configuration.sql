CREATE TABLE IF NOT EXISTS configuration (
    configuration_id INTEGER PRIMARY KEY,
    scope TEXT NOT NULL CHECK(scope IN ('global', 'project', 'session')),
    project_id TEXT REFERENCES projects(project_id) ON DELETE CASCADE,
    session_id TEXT REFERENCES session(session_id) ON DELETE CASCADE,
    key TEXT NOT NULL CHECK(length(trim(key)) > 0),
    value_json TEXT NOT NULL CHECK(json_valid(value_json)),
    updated_at TEXT NOT NULL,
    CHECK(
        (scope = 'global' AND project_id IS NULL AND session_id IS NULL)
        OR (scope = 'project' AND project_id IS NOT NULL AND session_id IS NULL)
        OR (scope = 'session' AND project_id IS NULL AND session_id IS NOT NULL)
    )
);

CREATE UNIQUE INDEX IF NOT EXISTS configuration_global_key_idx
    ON configuration(key) WHERE scope = 'global';
CREATE UNIQUE INDEX IF NOT EXISTS configuration_project_key_idx
    ON configuration(project_id, key) WHERE scope = 'project';
CREATE UNIQUE INDEX IF NOT EXISTS configuration_session_key_idx
    ON configuration(session_id, key) WHERE scope = 'session';

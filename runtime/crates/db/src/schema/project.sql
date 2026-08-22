CREATE TABLE IF NOT EXISTS project (
    project_id TEXT PRIMARY KEY CHECK(length(project_id) > 0),
    canonical_root TEXT NOT NULL UNIQUE CHECK(length(canonical_root) > 0),
    display_name TEXT NOT NULL CHECK(length(display_name) > 0),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    last_opened_at TEXT NOT NULL,
    archived_at TEXT
);

CREATE INDEX IF NOT EXISTS project_last_opened_idx
    ON project(archived_at, last_opened_at DESC);

CREATE TABLE IF NOT EXISTS project_dependency (
    dependency_id TEXT PRIMARY KEY CHECK(length(dependency_id) > 0),
    project_id TEXT NOT NULL,
    canonical_root TEXT NOT NULL CHECK(length(canonical_root) > 0),
    display_name TEXT NOT NULL CHECK(length(display_name) > 0),
    created_at TEXT NOT NULL,
    UNIQUE(project_id, canonical_root)
);

CREATE INDEX IF NOT EXISTS project_dependency_project_name_idx
    ON project_dependency(project_id, display_name, dependency_id);

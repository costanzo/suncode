CREATE TABLE IF NOT EXISTS checkpoint (
    checkpoint_id TEXT PRIMARY KEY CHECK(length(checkpoint_id) > 0),
    session_id TEXT NOT NULL,
    turn_id TEXT,
    tool_call_id TEXT,
    relative_path TEXT,
    status TEXT NOT NULL CHECK(status IN ('available', 'restored', 'invalidated')),
    created_at TEXT NOT NULL,
    restored_at TEXT,
    invalidated_at TEXT,
    manifest_id TEXT,
    ordinal INTEGER CHECK(ordinal IS NULL OR ordinal >= 0)
);

CREATE INDEX IF NOT EXISTS checkpoint_manifest_ordinal_idx
    ON checkpoint(manifest_id, ordinal);

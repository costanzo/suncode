CREATE TABLE IF NOT EXISTS checkpoint_manifest (
    manifest_id TEXT PRIMARY KEY CHECK(length(manifest_id) > 0),
    session_id TEXT NOT NULL REFERENCES session(session_id) ON DELETE CASCADE,
    turn_id TEXT REFERENCES session_turn(turn_id) ON DELETE SET NULL,
    status TEXT NOT NULL CHECK(status IN (
        'available',
        'restoring',
        'restored',
        'partial',
        'conflict',
        'expired'
    )),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    expires_at TEXT NOT NULL,
    restored_at TEXT
);

CREATE INDEX IF NOT EXISTS checkpoint_manifest_session_status_idx
    ON checkpoint_manifest(session_id, status, created_at DESC);
CREATE INDEX IF NOT EXISTS checkpoint_manifest_expiry_idx
    ON checkpoint_manifest(expires_at, manifest_id)
    WHERE status = 'available';
CREATE UNIQUE INDEX IF NOT EXISTS checkpoint_manifest_turn_idx
    ON checkpoint_manifest(turn_id)
    WHERE turn_id IS NOT NULL;

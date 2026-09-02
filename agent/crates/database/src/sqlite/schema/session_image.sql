CREATE TABLE IF NOT EXISTS session_image (
    image_id TEXT PRIMARY KEY CHECK(length(trim(image_id)) > 0),
    session_id TEXT NOT NULL,
    display_name TEXT NOT NULL CHECK(length(trim(display_name)) > 0),
    source_kind TEXT NOT NULL CHECK(source_kind IN ('file', 'clipboard')),
    original_path TEXT,
    storage_path TEXT NOT NULL CHECK(length(trim(storage_path)) > 0),
    thumbnail_base64 TEXT NOT NULL CHECK(length(trim(thumbnail_base64)) > 0),
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS session_image_session_created_idx
    ON session_image(session_id, created_at, image_id);

CREATE TABLE IF NOT EXISTS language_server (
    language_server_id TEXT PRIMARY KEY CHECK(length(trim(language_server_id)) > 0),
    display_name TEXT NOT NULL CHECK(length(trim(display_name)) > 0),
    config_json TEXT NOT NULL CHECK(json_valid(config_json)),
    enabled INTEGER NOT NULL DEFAULT 1 CHECK(enabled IN (0, 1)),
    sort_order INTEGER NOT NULL DEFAULT 0 CHECK(sort_order >= 0),
    revision INTEGER NOT NULL DEFAULT 1 CHECK(revision > 0),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS language_server_display_name_idx
    ON language_server(display_name COLLATE NOCASE);
CREATE INDEX IF NOT EXISTS language_server_enabled_order_idx
    ON language_server(enabled, sort_order, language_server_id);

CREATE TABLE IF NOT EXISTS mcp_server (
    mcp_server_id TEXT PRIMARY KEY CHECK(length(trim(mcp_server_id)) > 0),
    display_name TEXT NOT NULL CHECK(length(trim(display_name)) > 0),
    tool_prefix TEXT NOT NULL CHECK(length(trim(tool_prefix)) > 0),
    transport_type TEXT NOT NULL CHECK(transport_type IN ('stdio', 'streamable_http')),
    transport_config_json TEXT NOT NULL CHECK(json_valid(transport_config_json)),
    enabled INTEGER NOT NULL DEFAULT 1 CHECK(enabled IN (0, 1)),
    sort_order INTEGER NOT NULL DEFAULT 0 CHECK(sort_order >= 0),
    revision INTEGER NOT NULL DEFAULT 1 CHECK(revision > 0),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS mcp_server_display_name_idx
    ON mcp_server(display_name COLLATE NOCASE);
CREATE UNIQUE INDEX IF NOT EXISTS mcp_server_tool_prefix_idx
    ON mcp_server(tool_prefix COLLATE NOCASE);
CREATE INDEX IF NOT EXISTS mcp_server_enabled_order_idx
    ON mcp_server(enabled, sort_order, mcp_server_id);

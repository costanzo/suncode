CREATE TABLE IF NOT EXISTS session_subagent_invocation (
    invocation_id TEXT PRIMARY KEY CHECK(length(trim(invocation_id)) > 0),
    parent_session_id TEXT NOT NULL,
    parent_turn_id TEXT NOT NULL,
    parent_tool_call_id TEXT NOT NULL,
    child_session_id TEXT NOT NULL UNIQUE,
    agent_id TEXT NOT NULL,
    agent_version INTEGER NOT NULL CHECK(agent_version > 0),
    task_json TEXT NOT NULL,
    allowed_tools_json TEXT NOT NULL,
    model_id TEXT NOT NULL,
    state TEXT NOT NULL CHECK(state IN ('created', 'running', 'awaiting_approval', 'completed', 'failed', 'cancelled', 'interrupted')),
    result_json TEXT,
    error_code TEXT,
    created_at TEXT NOT NULL,
    started_at TEXT,
    completed_at TEXT
);

CREATE INDEX IF NOT EXISTS session_subagent_invocation_parent_idx
    ON session_subagent_invocation(parent_session_id, created_at DESC, invocation_id);

CREATE INDEX IF NOT EXISTS session_subagent_invocation_turn_idx
    ON session_subagent_invocation(parent_turn_id, parent_tool_call_id);

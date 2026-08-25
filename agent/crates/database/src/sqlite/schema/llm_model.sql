CREATE TABLE IF NOT EXISTS llm_model (
    model_id TEXT PRIMARY KEY CHECK(length(model_id) > 0),
    provider_id TEXT NOT NULL REFERENCES llm_model_provider(provider_id) ON DELETE CASCADE,
    display_name TEXT NOT NULL CHECK(length(display_name) > 0),
    request_model TEXT NOT NULL CHECK(length(request_model) > 0),
    context_tokens INTEGER NOT NULL CHECK(context_tokens >= 16000),
    auto_compact_tokens INTEGER NOT NULL CHECK(auto_compact_tokens >= 1000 AND auto_compact_tokens < context_tokens),
    max_output_tokens INTEGER CHECK(max_output_tokens IS NULL OR max_output_tokens > 0),
    supports_streaming INTEGER NOT NULL DEFAULT 1 CHECK(supports_streaming IN (0, 1)),
    supports_tool_use INTEGER NOT NULL DEFAULT 1 CHECK(supports_tool_use IN (0, 1)),
    supports_vision INTEGER NOT NULL DEFAULT 0 CHECK(supports_vision IN (0, 1)),
    supports_structured_output INTEGER NOT NULL DEFAULT 0 CHECK(supports_structured_output IN (0, 1)),
    supports_cancellation INTEGER NOT NULL DEFAULT 1 CHECK(supports_cancellation IN (0, 1)),
    supports_reasoning_effort INTEGER NOT NULL DEFAULT 0 CHECK(supports_reasoning_effort IN (0, 1)),
    enabled INTEGER NOT NULL DEFAULT 1 CHECK(enabled IN (0, 1)),
    sort_order INTEGER NOT NULL DEFAULT 0 CHECK(sort_order >= 0),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE(provider_id, model_id)
);

CREATE INDEX IF NOT EXISTS llm_model_by_provider_enabled_order_idx
    ON llm_model(provider_id, enabled, sort_order, model_id);

CREATE INDEX IF NOT EXISTS llm_model_enabled_order_idx
    ON llm_model(enabled, sort_order, model_id);

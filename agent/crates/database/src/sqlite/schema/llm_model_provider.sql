CREATE TABLE IF NOT EXISTS llm_model_provider (
    provider_id TEXT PRIMARY KEY CHECK(length(provider_id) > 0),
    display_name TEXT NOT NULL CHECK(length(display_name) > 0),
    endpoint TEXT NOT NULL CHECK(length(endpoint) > 0),
    adapter_type TEXT NOT NULL DEFAULT 'openai' CHECK(adapter_type IN ('openai')),
    api_key TEXT,
    enabled INTEGER NOT NULL DEFAULT 1 CHECK(enabled IN (0, 1)),
    sort_order INTEGER NOT NULL DEFAULT 0 CHECK(sort_order >= 0),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS llm_model_provider_enabled_order_idx
    ON llm_model_provider(enabled, sort_order, provider_id);

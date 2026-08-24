INSERT INTO llm_model (
    model_id, provider_id, display_name, request_model,
    context_tokens, auto_compact_tokens, max_output_tokens,
    supports_streaming, supports_tool_use, supports_vision,
    supports_structured_output, supports_cancellation,
    enabled, sort_order, created_at, updated_at
) VALUES
    ('deepseek-v4-flash', 'deepseek', 'DeepSeek V4 Flash', 'deepseek-v4-flash', 64000, 47616, NULL, 1, 1, 0, 0, 1, 1, 10, '2026-08-19T00:00:00.000Z', '2026-08-19T00:00:00.000Z'),
    ('deepseek-v4-pro', 'deepseek', 'DeepSeek V4 Pro', 'deepseek-v4-pro', 64000, 47616, NULL, 1, 1, 0, 0, 1, 1, 20, '2026-08-19T00:00:00.000Z', '2026-08-19T00:00:00.000Z'),
    ('glm-5.2', 'zhipu', 'GLM 5.2', 'glm-5.2', 1000000, 983616, 128000, 1, 1, 0, 0, 1, 1, 30, '2026-08-19T00:00:00.000Z', '2026-08-19T00:00:00.000Z'),
    ('glm-5.3', 'zhipu', 'GLM 5.3', 'glm-5.3', 1000000, 983616, 128000, 1, 1, 0, 0, 1, 1, 40, '2026-08-19T00:00:00.000Z', '2026-08-19T00:00:00.000Z'),
    ('gpt-5.6-sol', 'openai', 'GPT 5.6 SOL', 'gpt-5.6-sol', 1048576, 1032192, 128000, 1, 1, 0, 0, 1, 1, 50, '2026-08-19T00:00:00.000Z', '2026-08-19T00:00:00.000Z'),
    ('gpt-5.5', 'openai', 'GPT 5.5', 'gpt-5.5', 1048576, 1032192, 128000, 1, 1, 0, 0, 1, 1, 60, '2026-08-19T00:00:00.000Z', '2026-08-19T00:00:00.000Z'),
    ('kimi-k2.7-code', 'kimi', 'Kimi K2.7 Code', 'kimi-k2.7-code', 262144, 245760, NULL, 1, 1, 0, 0, 1, 1, 70, '2026-08-19T00:00:00.000Z', '2026-08-19T00:00:00.000Z'),
    ('kimi-k3', 'kimi', 'Kimi K3', 'kimi-k3', 262144, 245760, NULL, 1, 1, 0, 0, 1, 1, 80, '2026-08-19T00:00:00.000Z', '2026-08-19T00:00:00.000Z'),
    ('claude-opus-5', 'claude', 'Claude Opus 5', 'claude-opus-5', 1000000, 983616, NULL, 1, 1, 0, 0, 1, 1, 90, '2026-08-19T00:00:00.000Z', '2026-08-19T00:00:00.000Z'),
    ('claude-sonnet-5', 'claude', 'Claude Sonnet 5', 'claude-sonnet-5', 1000000, 983616, NULL, 1, 1, 0, 0, 1, 1, 100, '2026-08-19T00:00:00.000Z', '2026-08-19T00:00:00.000Z'),
    ('gemini-3.6-flash', 'gemini', 'Gemini 3.6 Flash', 'gemini-3.6-flash', 64000, 47616, NULL, 1, 1, 0, 0, 1, 1, 110, '2026-08-19T00:00:00.000Z', '2026-08-19T00:00:00.000Z'),
    ('gemini-3.5', 'gemini', 'Gemini 3.5', 'gemini-3.5', 64000, 47616, NULL, 1, 1, 0, 0, 1, 1, 120, '2026-08-19T00:00:00.000Z', '2026-08-19T00:00:00.000Z')
ON CONFLICT(model_id) DO NOTHING;

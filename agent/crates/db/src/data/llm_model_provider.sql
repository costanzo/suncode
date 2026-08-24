INSERT INTO llm_model_provider (
    provider_id, display_name, endpoint, adapter_type, api_key, enabled, sort_order, created_at, updated_at
) VALUES
    ('deepseek', 'DeepSeek', 'https://api.deepseek.com', 'openai', NULL, 1, 10, '2026-08-19T00:00:00.000Z', '2026-08-19T00:00:00.000Z'),
    ('zhipu', 'Zhipu GLM', 'https://open.bigmodel.cn/api/paas/v4', 'openai', NULL, 1, 20, '2026-08-19T00:00:00.000Z', '2026-08-19T00:00:00.000Z'),
    ('openai', 'OpenAI', 'https://api.openai.com/v1', 'openai', NULL, 1, 30, '2026-08-19T00:00:00.000Z', '2026-08-19T00:00:00.000Z'),
    ('kimi', 'Kimi', 'https://api.moonshot.ai/v1', 'openai', NULL, 1, 40, '2026-08-19T00:00:00.000Z', '2026-08-19T00:00:00.000Z'),
    ('claude', 'Claude', 'https://api.anthropic.com/v1', 'openai', NULL, 1, 50, '2026-08-19T00:00:00.000Z', '2026-08-19T00:00:00.000Z'),
    ('gemini', 'Gemini', 'https://generativelanguage.googleapis.com/v1beta/openai', 'openai', NULL, 1, 60, '2026-08-19T00:00:00.000Z', '2026-08-19T00:00:00.000Z')
ON CONFLICT(provider_id) DO NOTHING;

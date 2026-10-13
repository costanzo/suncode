INSERT INTO llm_model_provider (
    provider_id, display_name, endpoint, default_endpoint, adapter_type, api_key, enabled, sort_order, created_at, updated_at
) VALUES
    ('deepseek', 'DeepSeek', 'https://api.deepseek.com', 'https://api.deepseek.com', 'openai', NULL, 1, 10, '2026-08-19T00:00:00.000Z', '2026-08-19T00:00:00.000Z'),
    ('qwen', 'Qwen', 'https://dashscope.aliyuncs.com/compatible-mode/v1', 'https://dashscope.aliyuncs.com/compatible-mode/v1', 'openai', NULL, 1, 15, '2026-10-08T00:00:00.000Z', '2026-10-08T00:00:00.000Z'),
    ('zhipu', 'Zhipu GLM', 'https://open.bigmodel.cn/api/paas/v4', 'https://open.bigmodel.cn/api/paas/v4', 'openai', NULL, 1, 20, '2026-08-19T00:00:00.000Z', '2026-08-19T00:00:00.000Z'),
    ('openai', 'OpenAI', 'https://api.openai.com/v1', 'https://api.openai.com/v1', 'openai', NULL, 1, 30, '2026-08-19T00:00:00.000Z', '2026-08-19T00:00:00.000Z'),
    ('kimi', 'Kimi', 'https://api.moonshot.ai/v1', 'https://api.moonshot.ai/v1', 'openai', NULL, 1, 40, '2026-08-19T00:00:00.000Z', '2026-08-19T00:00:00.000Z'),
    ('claude', 'Claude', 'https://api.anthropic.com/v1', 'https://api.anthropic.com/v1', 'anthropic', NULL, 1, 50, '2026-08-19T00:00:00.000Z', '2026-08-19T00:00:00.000Z'),
    ('gemini', 'Gemini', 'https://generativelanguage.googleapis.com/v1beta/openai', 'https://generativelanguage.googleapis.com/v1beta/openai', 'openai', NULL, 1, 60, '2026-08-19T00:00:00.000Z', '2026-08-19T00:00:00.000Z'),
    ('xai', 'xAI', 'https://api.x.ai/v1', 'https://api.x.ai/v1', 'openai', NULL, 1, 70, '2026-10-12T00:00:00.000Z', '2026-10-12T00:00:00.000Z'),
    ('minimax', 'MiniMax', 'https://api.minimax.io/v1', 'https://api.minimax.io/v1', 'openai', NULL, 1, 80, '2026-10-12T00:00:00.000Z', '2026-10-12T00:00:00.000Z'),
    ('meta', 'Meta', 'https://api.meta.ai/v1', 'https://api.meta.ai/v1', 'openai', NULL, 1, 90, '2026-10-12T00:00:00.000Z', '2026-10-12T00:00:00.000Z'),
    ('mimo', 'Xiaomi MiMo', 'https://api.xiaomimimo.com/v1', 'https://api.xiaomimimo.com/v1', 'openai', NULL, 1, 100, '2026-10-12T00:00:00.000Z', '2026-10-12T00:00:00.000Z')
ON CONFLICT(provider_id) DO NOTHING;

# Architecture

`session` remains the root for turns, calls, messages, tools, approvals, checkpoints, and session-scoped configuration.

`configuration` uses explicit nullable `project_id` and `session_id` foreign keys. A scope CHECK permits exactly one ownership shape: neither owner for global, only project for project, and only session for session. Partial unique indexes enforce one key per scope owner.

`llm_model_provider.adapter_type` separates provider identity from protocol implementation. Runtime core dispatches the value to an implementation exported by `suncode-llm`. The only current persisted adapter is `openai`, backed by `OpenAiCompatibleProvider`.

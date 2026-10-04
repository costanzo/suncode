# Context Compaction

**Status:** Implemented and focused-tested

Rust core shortens provider context automatically for long sessions. It does this without changing the transcript, audit rows, the SDK surface, or the schema.

- Before each provider request, core estimates request size. The estimate covers history, the current system prompt, advertised tool schemas, image estimates, and an output reserve. Compaction starts at the model's `auto_compact_tokens` threshold, or at the context window minus a 16,384-token reserve when no threshold is set. Assumed defaults are a 64,000-token window and a 20,000-token recent tail.
- Dropped history is removed one complete unit at a time, so an assistant tool call and its results are never split. The retained tail is never cut in the middle of a tool group.
- Core first builds a local summary. The selected model is then asked for a structured summary with `objective`, `important_constraints`, `completed_work`, `active_work`, `blockers`, and `next_action`, based on up to 48,000 characters of dropped content, with a 30-second limit. The model's summary replaces the local one only if it is valid, complete, and fits the retained budget. Summary input is treated as untrusted. The summary call appears as an ordinary provider exchange with its real usage.
- Each compaction is stored as an internal `session_call` checkpoint with `finish_reason=context_compacted` and null usage. It holds the summary, the retained messages without transient image bytes, and message/tool rowid boundaries. Later turns replay the latest checkpoint plus every row after its boundaries, including rows written in the same millisecond.
- The public `context.compacted` event carries summary metadata only, never the retained raw messages.
- A provider context-overflow error triggers one forced compact-and-retry per turn.

Not implemented: provider-native token counting, a separate compaction model, and manual compaction.

Decision: `ADR-20260927-durable-context-compaction`. Storage rules: [`contracts/persistence.md`](../../../contracts/persistence.md) and [`contracts/sqlite-schema.md`](../../../contracts/sqlite-schema.md).

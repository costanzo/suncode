# CLI One-shot Session Resume

**Status:** Implemented and focused-tested

`suncode session resume SESSION_ID (--prompt TEXT | --stdin)` reopens an archived primary session when needed, establishes atomic `watch_session`, and submits one new turn using the session's durable conversation context. Optional model and reasoning flags follow the same explicit-over-`SUNCODE_` precedence as `run`.

Resume and run share one typed event, lag recovery, tail-drain, cancellation, and terminal-response implementation. Text mode writes only final assistant content to stdout; JSONL emits typed events followed by `session.resume.result`.

The Rust SDK exposes the current pending approval for a validated session, while pending question state comes from the atomic snapshot. Resume checks both before provider submission and returns exit status 4 when either exists. It does not resolve approvals or questions and never reuses prompt stdin for continuation input.

Interactive multi-turn chat remains deferred.

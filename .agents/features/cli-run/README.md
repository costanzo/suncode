# CLI One-shot Run

**Status:** Implemented and focused-tested

`suncode run [PATH] (--prompt TEXT | --stdin)` opens the project through `AsyncAgentSdk`, creates a primary session, establishes an atomic `watch_session`, and submits one turn. `--model` and `--reasoning-effort` override `SUNCODE_MODEL` and `SUNCODE_REASONING_EFFORT`; ordinary SDK configuration supplies later defaults.

The command consumes typed SDK events with the standard Rust stream interface. JSONL mode adapts every received event into the versioned CLI envelope and finishes with `run.result`. Text mode sends turn/tool/context progress to stderr and emits only the final assistant text to stdout. Events already queued when submission returns are drained before the result is written. Lag establishes a fresh atomic watch rather than treating terminal state as authoritative.

The first interrupt requests `cancel_turn` once the active turn ID is known. A second interrupt returns status 130, and normal command exit still consumes SDK shutdown. A turn suspended for an approval or structured question returns status 4; `run` does not reuse prompt stdin for interactive continuation. Browser Use and Computer Use remain disabled by the CLI host capability ceiling.

Interactive `chat`, approval/question prompts, richer terminal rendering, and named non-interactive policy profiles remain separate deliveries. Session list/archive and one-shot resume are implemented by their dedicated CLI features.

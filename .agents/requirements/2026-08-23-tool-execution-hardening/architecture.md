# Architecture

The model-facing schema and audited execution modules live together in the `suncode-tool` package under `agent/crates/tools`. The package exposes neutral built-in definitions, while agent core translates them into provider request DTOs and owns orchestration, policy, approval, and conversation-only tool handling. No client or provider bypasses the audited execution boundary.

Read ranges are decoded only for UTF-8 text; binary reads remain byte-oriented when no line range is requested. Mutation preconditions compare the exact pre-image bytes. Process status is returned structurally and projected into the tool-use state without converting a process failure into a turn-level runtime failure.

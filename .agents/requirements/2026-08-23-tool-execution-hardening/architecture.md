# Architecture

The model-facing schema remains in `runtime/crates/core/src/tools`. The agent translates the compact schema into the narrower audited operations contract. `suncode-tool` owns path scope, ignore-aware traversal, bounded reads, preconditioned mutations, and process lifecycle. No client or provider bypasses this boundary.

Read ranges are decoded only for UTF-8 text; binary reads remain byte-oriented when no line range is requested. Mutation preconditions compare the exact pre-image bytes. Process status is returned structurally and projected into the tool-use state without converting a process failure into a turn-level runtime failure.

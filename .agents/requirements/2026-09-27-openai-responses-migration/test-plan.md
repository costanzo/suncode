# Test Plan

## Scope

Responses request shape, text streaming, function calls, usage normalization, proxy routing, agent tool loops, compaction, and CLI resume behavior.

## Commands and results

- `cargo test -p suncode-llm -p suncode-agent`: passed.
- `cargo test --manifest-path apps/cli/Cargo.toml --test cli`: passed.

## Residual risks

Provider-specific gateways may advertise an OpenAI-compatible endpoint while implementing only Chat Completions. Those endpoints now fail until configured or upgraded to Responses compatibility.

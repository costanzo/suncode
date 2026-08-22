# Test Plan

## Scope

Normalized session timeline snapshots and Avalonia live/collapsed turn presentation.

## Unit tests

- Rust groups messages and tool uses by turn in stable order.
- Avalonia projects active and terminal turns correctly.
- Live assistant/tool updates remain ordered and idempotent.
- Expand/collapse preserves process items and final copy eligibility.

## Integration and conformance tests

- Runtime SDK serialization includes additive `conversationTurns` while retaining `messages`.

## Regression checks

- Empty assistant tool-call messages stay hidden.
- Duplicate message delivery stays idempotent.
- Separate turns remain separate conversation rounds.

## Manual checks

- Run a tool-using turn and inspect live ordering, terminal collapse, expansion, copy controls, and session reload.

## Commands and results

- `cargo test --manifest-path runtime/Cargo.toml -p suncode-db conversation_turns_group_messages_and_tools`: passed.
- `cargo test --manifest-path runtime/Cargo.toml -p suncode-runtime session_snapshot_serializes_normalized_conversation_turns`: passed.
- `dotnet test apps/desktop-avalonia-tests/SunCode.Desktop.Tests.csproj --no-restore`: passed, 34 tests, including stable streaming/final message identity.
- `dotnet build apps/desktop-avalonia/SunCode.Desktop.csproj --no-restore`: passed with no warnings or errors.
- `cargo test --manifest-path runtime/Cargo.toml --workspace`: passed, 86 unit tests and all doc tests.
- `git diff --check`: passed.

## Residual risks

- Manual validation requires a configured provider.

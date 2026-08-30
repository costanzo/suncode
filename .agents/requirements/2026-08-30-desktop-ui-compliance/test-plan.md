# Test Plan

## Scope

SDK attachment contracts, desktop projections, Review ordering, responsive restoration, accessibility metadata, Markdown states, Settings, theme tokens, and bounded image/conversation behavior.

## Unit tests

- Attachment validation, ownership, limits, capability, and message projection.
- Session status projection and Review summary helpers.
- Compact visibility preference preservation and surface commands.
- Bounded thumbnail calculation and file validation.

## Integration and conformance tests

- Rust SDK submit and conversation snapshot attachment round trip.
- C ABI and C# payload compatibility.
- Existing text-only submission compatibility.

## Regression checks

- Existing approval, question, checkpoint, Git, trace, settings, and responsive tests.
- Design-system production build.

## Manual checks

- Dark/light ProjectHub, Workspace, Settings, and dialogs.
- Compact and expanded panel restoration.
- Keyboard-only focus order, overlays, copy feedback, radio/checkbox behavior.
- Long conversation, long paths, large/invalid images, and unsupported model.

## Commands and results

- `cargo test --workspace --all-targets` — passed: 94 Rust tests across core, data, database, LLM, common, and tools.
- `cargo fmt --all --check` — passed.
- `dotnet test apps/desktop-avalonia/tests/SunCode.Desktop.Tests.csproj --no-restore` — passed: 51 tests.
- `dotnet build apps/desktop-avalonia/SunCode.Desktop.csproj --no-restore` — passed with zero warnings and zero errors.
- `npm run build` in `design-system/` — passed.
- `git diff --check` — passed.
- Source-backed dark/light, compact/expanded, focus, overflow, and state specimens were reconciled in `design-system/`; native screen-reader and GPU/memory profiling remain platform-manual residual checks.

## Residual risks

- Native screen-reader behavior and GPU/memory profiling require platform-level manual verification.

# Test Plan

## Scope

Built-in agent catalog, delegation, persistence, SDK projection, and desktop presentation.

## Unit tests

- Catalog IDs/names are unique and tool names resolve.
- Each role advertises its exact allowed tools.
- Child continuations cannot delegate or use MCP.

## Integration and conformance tests

- Delegation creates correlated invocation and child session rows.
- Child completion/failure is returned to the parent tool call.
- Public child submissions and session mutations are rejected.
- Rust/C/C# DTO fields serialize with the documented names.

## Regression checks

- Primary session creation/listing/submission is unchanged.
- Existing policy, approval, checkpoint, image, trace, and MCP tests pass.
- Existing databases upgrade only from the immediately preceding valid schema.

## Manual checks

- Settings catalog navigation and detail.
- Right-side child session list and central read-only detail.
- ContentSwitcher restore and compact-width behavior.

## Commands and results

- `cargo test --workspace` in `agent`: passed (all workspace tests, including the 16-to-17-table compatibility case).
- `cargo test` in `sdks/rust`: passed (20 tests).
- `cargo test` in `sdks/c`: passed (2 tests).
- `dotnet build sdks/csharp/SunCode.Sdk.csproj`: passed.
- `dotnet test apps/desktop-avalonia/tests`: passed (103 tests).
- `npm run build` in `design-system`: passed; Vite emitted only the existing large-chunk advisory.
- `git diff --check`: passed.

## Residual risks

- Unified parent-turn undo for child mutations requires explicit operation grouping and must be reported honestly if deferred.

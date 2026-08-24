# Test Plan

## Scope

Persistence ownership, path containment, SDK DTO privacy, dependency routing, Explorer compilation, and regression coverage.

## Unit tests

- Store registration uniqueness, ownership, removal, and schema bootstrap.
- SDK root canonicalization, overlap rejection, directory browsing, and removal.
- Dependency path parsing and read/glob/grep result alias preservation.
- Operations directory containment, ordering, bounds, and symlink omission through existing/focused coverage.

## Integration and conformance tests

- Rust workspace tests.
- Avalonia project build against the hand-written C ABI wrapper.
- Runtime SDK shared vector JSON validation.

## Regression checks

- Cargo formatting check.
- `git diff --check`.
- Inspect the final diff without modifying unrelated logging/session work.

## Manual checks

- Open Explorer, expand project and dependencies, refresh, add a folder, and remove it.
- Confirm removing a registration leaves the dependency folder untouched.
- Confirm user-visible error reporting for invalid or overlapping roots.

## Commands and results

- `dotnet build apps/desktop-avalonia/SunCode.Desktop.csproj --no-restore`: passed, 0 warnings/errors.
- `cargo fmt --manifest-path agent/Cargo.toml --all -- --check`: passed.
- `cargo test --manifest-path agent/Cargo.toml --workspace`: passed (84 tests: DB 32, LLM 3, runtime 27, operations 22).
- `jq empty contracts/vectors/runtime-sdk.json`: passed.
- `node .agents/skills/impeccable/scripts/detect.mjs --json ...`: passed with no findings.
- `git diff --check`: passed.

## Residual risks

- No automated Avalonia interaction test exercises the native folder picker or expansion gestures; compilation and manual smoke testing cover that UI boundary.

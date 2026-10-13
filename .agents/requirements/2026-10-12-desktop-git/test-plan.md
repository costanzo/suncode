# Test plan

Real temporary Git repositories cover scope semantics, unborn/detached HEAD, linked worktrees, subtree boundaries, renames, conflicts, untracked/binary files, unusual paths, output limits, and read-only index behavior. Fake services cover absence, re-detection, cancellation, and stale results. Run desktop tests/build, focused Rust dispatcher tests and SDK/C/CLI checks, Cargo dependency scans, design-system build and both-theme/narrow review, and git diff --check.

## Commands and results

- `dotnet test apps/desktop-avalonia/tests/SunCode.Desktop.Tests.csproj --no-restore`: 316 passed; includes native Rust build, managed SDK, production XAML compilation, and focused Git integration/presentation tests.
- `cargo test --offline --manifest-path agent/Cargo.toml -p suncode-tool --lib --locked`: 35 passed.
- `cargo test --offline --manifest-path sdks/rust/Cargo.toml --lib --locked`: 34 passed.
- `cargo test --offline --manifest-path sdks/c/Cargo.toml --lib --locked`: 4 passed.
- `cargo check --offline --manifest-path sdks/c/Cargo.toml --locked` and CLI equivalent: passed.
- `npm run build` in design-system: passed (existing bundle-size advisory).
- Dependency/source scans: no git2/libgit2 packages or dedicated Git SDK symbols remain.
- `git diff --check`: passed.

## Residual risks

Real Git integration was executed on macOS. Windows/Linux process discovery and Windows NUL handling require platform CI/live verification.

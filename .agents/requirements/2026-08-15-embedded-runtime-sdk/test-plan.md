# Test Plan

## Scope

Verify lifecycle, named SDK methods, C ABI ownership and errors, ordered event replay/live delivery, Qt behavior, and complete removal of inbound server behavior.

## Unit tests

- Rust SDK lifecycle opens state, performs recovery, and reports an already-active data directory.
- Each named service method validates input and preserves the previous handler behavior.
- SDK errors serialize bounded codes, messages, and redacted details without HTTP status dependence.
- C ABI rejects null and invalid UTF-8 pointers without unwinding.
- C ABI results and subscriptions can be released safely.
- Event replay/live handoff does not lose or duplicate durable sequences.
- Receiver lag produces replay recovery or a typed resync error.

## Integration and conformance tests

- Shared SDK vectors are accepted by Rust and the Qt adapter.
- Project, session, snapshot, credential, settings, model, turn, cancellation, approval, checkpoint, and diagnostic flows pass through named SDK methods.
- Qt can open the embedded runtime, load existing history, subscribe, submit a turn, and close subscriptions.

## Regression checks

- Provider outbound streaming and tool calls remain unchanged.
- SQLite schema and migration tests pass.
- Policy, approvals, checkpoints, and operation boundary tests pass.
- No production source binds a client-facing listener or exports `request_json`.

## Manual checks

- Launch the Qt desktop app with an isolated data directory.
- Open a project, create a session, select a configured model, submit a turn, and observe streaming state.
- Exercise approval allow/deny and checkpoint restore when credentials permit.

## Commands and results

- `cargo test --workspace`: passed, 35 tests plus doc tests.
- `cargo fmt --all -- --check`: passed.
- `cargo clippy --workspace --all-targets`: completed with one pre-existing `manual_clamp` warning in `context.rs`.
- `cmake --build apps/desktop-qt/build -j2`: passed; existing macOS object target-version linker warnings remain.
- `cmake --build apps/desktop-qt/build --target all_qmllint -j2`: passed with existing repository-wide import-resolution and unqualified-access warnings.
- Offscreen Qt startup with an isolated data directory reached the event loop without output or QML runtime errors and was manually interrupted; reopening the same directory confirmed that shutdown released the runtime lock.
- `cargo metadata --no-deps`: confirmed the runtime package exposes only `rlib` and `staticlib` targets.
- Normal dependency inspection confirmed Axum is absent from the production dependency graph; test-only Axum servers remain for provider simulation, while `reqwest` retains outbound HTTP dependencies.
- Static-library symbol inspection confirmed named SDK exports and no `request_json` symbol.
- `jq empty contracts/vectors/runtime-sdk.json` and `git diff --check`: passed.

## Residual risks

- Live provider calls require user credentials and are not part of deterministic repository verification.
- TypeScript and Python native package builds are deferred until their packaging deliveries.
- QML lint still reports the repository's existing import-resolution, layout-positioning, and unqualified-access warnings.
- The existing macOS link target-version warnings remain unchanged.

# Test Plan

## Scope

Qt Quick presentation and interaction wiring for the Phase 1 desktop client.

## Unit tests

No standalone QML unit harness exists; compile-time validation covers component/property errors.

## Integration and conformance tests

No runtime contracts change. Existing Rust and Qt focused tests remain applicable.

## Regression checks

- Configure and build `apps/desktop-qt`.
- Launch the binary and capture the default window.
- Verify the default launch is the project hub, not an auto-selected project window.
- Verify recent projects render from runtime project storage when no project is selected.
- Verify opening another project creates an additional project window rather than merging projects.
- Verify left and right panel toggle behavior.
- Verify connection, project/session, credential, composer, approval, undo, and diagnostic controls retain bindings.

## Manual checks

- Default 1440×900 hierarchy and alignment.
- Minimum 900×620 layout.
- Empty, disconnected, disabled, approval, active-turn, and long-content states where available.
- Keyboard focus visibility and text contrast.

## Commands and results

- `cmake --build apps/desktop-qt/build -j2` passed.
- Offscreen startup of `apps/desktop-qt/build/suncode-desktop` reached the project hub without QML TypeError or Runtime SDK connection-race errors.
- `node .agents/skills/impeccable/scripts/detect.mjs --json` returned `[]`.
- `qmllint apps/desktop-qt/qml/*.qml` completed with exit code 0; it reported existing broad QML style warnings for unqualified access and layout-managed width/height.
- `git diff --check` passed.

## Residual risks

- Runtime-populated states not reachable without a configured local runtime require source-level review if unavailable during validation.

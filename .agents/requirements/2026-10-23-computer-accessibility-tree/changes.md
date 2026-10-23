# Changes

Landed on 2026-10-23.

## Source

- `agent/crates/computer/src/accessibility/` (new): trait, normalized `Role`/`StateFlags`, request and snapshot types, shared walk and bounds, outline rendering and redaction, and `macos.rs`, `windows.rs`, `linux.rs` collectors behind `cfg(target_os)`.
- `agent/crates/computer/src/executor.rs`: frame epoch, `FrameSnapshot`, `ComputerError::PermissionRequired`, `ActionOutcome::Accessibility`.
- `agent/crates/computer/src/action.rs`: shares coordinate normalization with accessibility query parsing (`AccessibilityQuery` lives in the accessibility module, not `ComputerAction`).
- `agent/crates/computer/src/frame.rs`: `DisplayGeometry::input_point` so collection maps points without a frame copy; the inverse mapping for boxes is in `accessibility/outline.rs`.
- `agent/crates/computer/src/runtime.rs`: member registration, tool definition, accessibility backend installation, shared cancellable worker, short-lock frame copy, frame-epoch re-check, runtime-info fields.
- `agent/crates/computer/src/provider.rs`: risk mapping, catalog gating, `ActionOutcome::Accessibility` result.
- `agent/crates/core/src/agent/tools.rs`, `context.rs`, and `support/messages.rs`: retention of the newest tree outlines and redaction from compaction and approval continuations.
- `agent/crates/core/src/agent/computer.rs`: installs the platform accessibility backend with the input backend.
- `agent/crates/core/src/agent/system_prompt.rs`: accessibility workflow and untrusted-content guidance.
- `sdks/csharp`: additive optional runtime-info fields. The Rust SDK and C ABI pass the JSON DTO through unchanged; their lockfiles gain only new dependency edges.
- `agent/Cargo.toml`: pinned `core-foundation`, `windows`, and `dbus` workspace entries, all already locked.
- `apps/desktop-avalonia`: `ComputerRuntimeViewModel`, `ComputerUseSettingsControl.axaml`, en-US/zh-CN strings.

## Contracts and generated artifacts

- `contracts/agent-sdk/README.md`: tool contract, role/state vocabulary, redaction and retention rules, runtime-info fields, error codes.

## Configuration and persistence

- None. No new settings and no SQLite change.

## Tests

- See `test-plan.md`.

## Documentation

- `DESIGN.md` Computer Use Settings, and the `design-system/.../ComputerUsePanel.jsx` specimen.
- `.agents/features/computer-use/README.md`, and a new ADR in `.agents/DECISIONS.md` at closeout.

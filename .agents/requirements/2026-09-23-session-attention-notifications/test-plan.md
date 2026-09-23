# Test Plan

## Scope

Verify notification eligibility, global attention delivery, deduplication and reconciliation, foreground suppression, native platform delivery, activation IPC, session navigation, lifecycle cleanup, and preservation of the embedded-agent boundary.

## Unit tests

### Rust

- Primary completed and failed turns emit attention events.
- Child completion and failure do not emit attention events.
- Primary and child approvals emit with correct parent/child identity.
- Primary questions emit; invalid child-question state is not introduced.
- Cancelled and interrupted turns do not emit.
- Candidate reconciliation returns deterministic bounded results from normalized state.
- Attention subscribers are independent of session subscribers and cannot block publication.
- Lag returns one terminal lag error and allows a fresh subscription.

### Desktop

- Any active SunCode top-level marks the application foreground.
- No active window, minimized windows, or another active application marks SunCode background.
- Foreground events enter the ledger as `suppressed_foreground` and are never delivered later.
- Delivered, suppressed, duplicated, and reconciled correlation IDs are handled idempotently.
- Completion, failure, approval, question, and child-approval content is bounded and contains no paths or conversation content.
- Resolved approval/question activation still navigates without recreating interaction state.
- Ledger parsing rejects future versions safely and enforces retention bounds.

### IPC

- Valid `activate.session` round trip returns the matching request ID.
- Unknown versions/types, missing IDs, oversized frames, invalid UTF-8/JSON, and trailing frames are rejected.
- Secondary bootstrap exits without opening the SDK.
- Startup races elect one primary and never create two SDK owners.
- Unix stale-socket cleanup removes only the exact owned endpoint.
- Queued duplicate activation replaces the older request with the same correlation key.
- Shutdown rejects new requests and releases owned endpoint resources.

## Integration and conformance tests

- Rust attention DTOs serialize through C and deserialize into the hand-written C# types.
- C subscription close stops callbacks before returning and is safe during callback teardown.
- Coordinator start/stop ordering precedes final shared SDK disposal.
- Selected-session UI watch and global attention stream can observe the same turn without duplicate system notifications.
- Lag reconciliation plus the desktop ledger produces exactly-once user-visible delivery within the bounded reconciliation model.
- Activation resolves project/session ownership through the SDK rather than trusting the IPC payload.
- Child approval activation selects the parent session, child session, and existing approval surface.

## Regression checks

- Existing atomic session watch, stale-load guards, resync, session switching, and window coordination tests continue to pass.
- Existing project window reuse remains unchanged for ordinary UI navigation.
- Closing a project window does not terminate application-scoped monitoring while the embedded agent remains open.
- Agent turns complete normally when notification permissions or transports are unavailable.
- CLI behavior and the one-process-per-data-directory agent lock remain unchanged.

## Manual checks

### macOS

- First-turn authorization request, allow, deny, and later system-settings disablement.
- Background completion, failure, approval, and question notification display.
- Foreground suppression across Workspace, ProjectHub, Settings, About, and dialogs.
- Click activation with the primary process running and from a stale delivered notification after relaunch.
- Signed/notarized app-bundle behavior.

### Windows

- Installed and supported unpackaged/packaged registration model selected by the spike.
- Notification display, icon, grouping, activation arguments, secondary launch, Named Pipe forwarding, and primary-window focus.
- Registration cleanup after uninstall or replacement.

### Linux

- GNOME and KDE notification display and default-action activation.
- Capability behavior when notification actions are not advertised.
- Wayland and X11 window-focus suppression where supported by Avalonia window activation state.
- Desktop entry, icon, and secondary-process Unix socket forwarding.

## Commands and results

- `cargo test --manifest-path agent/Cargo.toml -p suncode-agent --lib`: 61 passed.
- `cargo test --manifest-path sdks/rust/Cargo.toml --lib`: 31 passed.
- `cargo test --manifest-path sdks/c/Cargo.toml --lib`: 5 passed.
- Focused `suncode-data` attention candidate test: passed.
- Full `suncode-data` suite: 14 passed, 1 unrelated existing model-catalog assertion failed because `glm-5.3` now advertises `max` in addition to the test's `low/high` expectation.
- `dotnet build apps/desktop-avalonia/SunCode.Desktop.csproj -c Release`: passed with zero warnings.
- Forced Windows-target desktop build from macOS: passed with zero warnings.
- `dotnet test apps/desktop-avalonia/tests/SunCode.Desktop.Tests.csproj -c Release`: 115 passed.
- macOS Swift User Notifications bridge compilation: passed.
- `git diff --check`: passed.
- Installed-application notification display/click smoke tests remain pending on macOS, Windows, GNOME, and KDE.

## Residual risks

- Linux notification servers are not behaviorally uniform and may omit click actions.
- Windows activation reliability still requires installed-host validation of toast and protocol registration.
- Foreground state is sampled immediately before dispatch and can race a simultaneous user focus change.

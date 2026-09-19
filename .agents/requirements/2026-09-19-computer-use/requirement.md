# Requirement

## Background

SunCode needs a first-party Computer Use capability that can observe and operate the user's graphical desktop, including native applications. The capability must be built into the Rust agent and configured through Settings. It must not be implemented through MCP and must remain separate from the Playwright-based Browser Use work.

The accepted direction uses the Rust `enigo` project as the low-level cross-platform desktop backend. The maintained source at `/Users/shuyi/Projects/github/costanzo/enigo` already provides keyboard and pointer injection, improved Windows DPI behavior, Linux portal input, restore tokens, and an internal Linux portal screenshot call. It must be extended with a public capture/display contract before it can serve a complete Computer Use runtime.

Claude's native `computer_toolset_20260801` is the first provider protocol. It exposes 17 ordered desktop actions and requires image-bearing tool results for `screenshot` and `zoom`.

## Goals

- Provide a first-party Rust-owned Computer Use runtime for the real desktop.
- Use Enigo behind an internal adapter for input, capture, display topology, permissions, and coordinate mapping.
- Support Claude's native Computer Use toolset and ordered batch semantics.
- Keep policy, approvals, auditing, cancellation, recovery, settings, and SDK state Rust-owned.
- Ship explicit permission, control, and emergency-stop surfaces in Avalonia Settings.
- Support macOS arm64, Windows x64, Linux X11, and Linux Wayland when its portal/PipeWire implementation passes conformance tests.

## Non-goals

- Reusing Browser Use, Playwright, Chromium, browser profiles, or browser tools.
- Exposing Computer Use through MCP.
- Claiming OS isolation around the embedded agent or the user's desktop session.
- Operating login screens, UAC secure desktop, locked desktops, or higher-integrity Windows applications.
- Multi-display control in the initial release.
- Non-interactive Computer Use in the initial release.
- Provider-neutral emulation of Claude's native toolset in the first delivery.

## Requirements

### Environment and actions

- Initial operation is limited to the primary display in an unlocked, logged-in desktop session.
- Implement all 17 Claude Computer Use members, or explicitly disable a member in the provider toolset configuration when the current backend cannot implement it.
- Execute batch members strictly in response order and stop after the first failure.
- Return one result for every requested member. Members skipped after a failure use the exact halt text required by the provider contract.
- `screenshot` and `zoom` return image content, not base64 embedded in ordinary text JSON.
- All input coordinates are mapped from the exact full-display screenshot coordinate space seen by the model.

### Authority

- Enabling Computer Use makes the capability available but grants no authority to act.
- Observation-only actions may run without approval; all input-producing actions require interactive approval in the initial release.
- Session Full Control does not bypass Computer Use approval.
- Non-interactive execution denies Computer Use.
- Page, application, document, screenshot, and other on-screen content is untrusted and cannot authorize an action.
- A user emergency stop cancels pending actions and releases every held key and mouse button.

### Privacy and persistence

- Screenshot bytes and typed sensitive text never enter logs, public SDK DTOs, provider traces, or SQLite JSON payloads.
- Screenshots are retained only for the active turn unless an approval continuation requires a bounded temporary frame.
- Provider traces use a redacted attachment marker.
- Restart never replays an input action with unknown completion. A resumed continuation captures a fresh frame and invalidates prior coordinates.
- Linux portal restore tokens are write-only secrets owned by Rust configuration.

### Settings and SDK

- Settings exposes enablement, model support, primary-display status, capture/input permission state, limits, current control owner, tests, stop, and temporary-data cleanup.
- Settings does not expose Enigo paths, MCP configuration, arbitrary executables, raw portal tokens, or an approval bypass.
- Rust, C, and C# SDKs expose named Computer Use management methods and redacted runtime DTOs.
- Browser Use configuration and runtime remain unchanged.

## Edge cases

- macOS Retina screenshots and CGEvent coordinates differ in scale.
- Windows processes may be DPI-virtualized and cannot inject into higher-integrity processes.
- X11 display coordinates may differ from screen capture dimensions under scaling.
- Wayland absolute input requires a ScreenCast stream associated with the RemoteDesktop session.
- Display resolution, scale, orientation, or primary-display identity may change after a screenshot.
- User input can race with agent input; control ownership must be exclusive.
- Cancellation may occur while a modifier or mouse button is held.
- Screenshot dimensions may exceed provider image limits.
- A provider response may contain both ordinary tools and multiple Computer Use members.

## Acceptance criteria

- Computer Use is first-party, built in, and independent of MCP and Browser Use.
- Disabled or unsupported configurations advertise no Computer Use toolset.
- Claude's native toolset and image results pass focused wire-contract tests.
- Batch calls execute sequentially, stop on failure, and answer every member.
- Coordinate conformance passes on supported platform/DPI combinations.
- Cancellation and emergency stop leave no held inputs.
- Full Control and non-interactive modes follow the authority requirements above.
- No screenshot bytes, sensitive typed text, or portal token appears in prohibited persistence and diagnostic surfaces.
- Settings and production Avalonia behavior match the design-system specimen in both themes and constrained sizes.
- Applicable Rust, SDK, desktop, packaging, and platform smoke tests pass.

## Open questions

- The stable Wayland availability date depends on portal/PipeWire conformance across GNOME and KDE.
- Additional provider-native Computer Use adapters are deferred until the Claude delivery is stable.

# Test Plan

## Scope

Verify the Enigo capture/input backend, screenshot coordinate space, Claude client-toolset protocol, ordered execution, authority, privacy, recovery, SDK parity, desktop presentation, and target packaging.

## Unit tests

- Display/frame geometry and screenshot-to-input transforms.
- Coordinate, region, duration, repeat, text, modifier, and key-chord validation.
- Screenshot resize limits and aspect-ratio preservation.
- Zoom cropping without changing the full-frame coordinate space.
- Ordered batch success, first-failure halt, skipped results, and result completeness.
- Held-input cleanup on success, failure, cancellation, panic containment, and drop.
- Provider-neutral client-toolset call/result serialization.
- Configuration bounds and secret redaction.

## Integration and conformance tests

- Capture a known fixture desktop, click known targets, and confirm pixel-level round trips.
- Exercise all 17 members against the mock and supported platform backends.
- Verify Claude Messages requests and multi-member responses with image tool results.
- Prove input actions always pass through approval and are denied non-interactively.
- Prove Full Control does not bypass Computer Use approval.
- Prove user takeover and emergency stop halt subsequent actions and invalidate frames.
- Prove display/DPI changes invalidate coordinates.
- Prove restart never replays unknown-completion input.
- Prove screenshots and sensitive typed text do not enter SQLite, logs, events, traces, or public DTOs.

## Regression checks

- Existing built-in tools, providers, MCP, LSP, Browser Use work, approvals, recovery, SDK, and desktop tests remain green.
- Computer Use disabled leaves the existing provider requests and tool catalog unchanged.
- Custom OpenAI-compatible Claude gateways are preserved and do not falsely advertise native Computer Use.
- Filesystem checkpoint and undo claims remain limited to filesystem operations.

## Manual checks

- macOS Screen Recording and Accessibility onboarding, Retina scaling, cancellation, and locked-screen failure.
- Windows DPI scaling, UIPI denial, ordinary desktop applications, and secure-desktop failure.
- Linux X11 capture/input alignment.
- Linux GNOME and KDE Wayland portal authorization, restore, PipeWire frames, and absolute pointer alignment.
- Settings and approval states in light/dark themes and constrained window sizes.
- Emergency stop from keyboard and Settings while an action is active.

## Commands and results

- `cargo check --all-features` in the Enigo repository — passed.
- `cargo test screen::tests --lib` in the Enigo repository — passed, 3 tests; no real input was generated.
- `cargo check --all-features` in Enigo — passed after adding macOS permission status/request APIs at revision `36c08bbe4a00150d2b1817553985a2c115169e3a`.
- `cargo test -p suncode-computer` — passed, 8 tests, including provider resize, resized-coordinate mapping, and frame retirement; no real input was generated.
- `cargo test -p suncode-agent computer_context_tests --lib` — passed, proving older screenshot bytes are pruned without dropping correlated tool results.
- `cargo check --manifest-path agent/Cargo.toml -p suncode-computer -p suncode-agent` — passed.
- `cargo check --manifest-path sdks/c/Cargo.toml` — passed after runtime-info and emergency-stop wiring.
- `dotnet build sdks/csharp/SunCode.Sdk.csproj --no-restore` — passed with zero warnings and errors.
- `dotnet build apps/desktop-avalonia/SunCode.Desktop.csproj --no-restore` — passed with zero warnings and errors.
- `npm run build` in `design-system` — passed; Vite reported only its existing large-chunk advisory.
- `git diff --check` in SunCode — passed.
- The broad Enigo `cargo test --lib` suite was intentionally discontinued because it operates the real mouse and keyboard. Before the Retina normalization change it exposed two existing 2× coordinate failures, which motivated the physical-pixel fix; broad real-input verification remains a controlled manual/platform test.

## Residual risks

- Desktop environments and applications vary in how they process synthetic input.
- Screen content can contain prompt injection or sensitive data despite policy and approval controls.
- Wayland compositor behavior may require an experimental support label after the rest of the delivery is stable.

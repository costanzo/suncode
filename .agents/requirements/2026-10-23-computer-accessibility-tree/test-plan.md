# Test Plan

## Scope

Read-only accessibility tree observation for Computer Use on macOS, Windows, and X11 Linux: tool gating, execution safety, coordinate alignment, redaction, retention, runtime state, and Settings presentation.

## Unit tests

- Action parsing: scope variants, `point` requiring `coordinate`, and ceiling clamping or rejection.
- Fake backend: wrapper pruning, depth and node ceilings, deadline truncation with partial results, and offscreen filtering.
- Inverse coordinate mapping round-trips with `input_point` at 1x, 2x, and fractional scales. Non-intersecting boxes become `None`.
- Secure-text redaction, per-field truncation, and total-size bound.
- Outline rendering is stable for a fixed snapshot.
- Missing frame gives `computer_frame_required`. A generation change during collection discards the result.
- Cancellation, user takeover, and emergency stop during a slow fake collection return promptly and do not block on the executor mutex.
- Catalog: the tool is absent when disabled, under user control, host-unavailable, model-unsupported, or accessibility-unavailable.
- Redacted `ToolResult.value` contains no names, values, or titles.

## Integration and conformance tests

- Core: the outline stays in provider context, history and events hold only metadata, and older outlines become omission markers.
- SDK/C/C#: runtime-info fields serialize and deserialize across every layer.
- Per-platform fixture app (a native window with button, checkbox, text field, password field, list, and an offscreen element):
  - Every expected role, name, and state is reported.
  - The password value is redacted.
  - Clicking each reported box center with `computer_left_click` activates the matching control.
  - macOS: Retina and non-Retina. Windows: 100%, 150%, and mixed-DPI monitors. X11: scale 1 and 2.
  - Permission denied (macOS) and the accessibility bus disabled (X11) report the correct state and do not advertise the tool.
  - Windows elevated target is reported as inaccessible.

## Regression checks

- Existing `suncode-computer` executor, frame, and provider tests.
- Existing Computer Use approval, emergency stop, takeover, image retention, and runtime-info tests.
- Avalonia `RuntimeViewModelTests` and `SettingsPageViewModelTests`.

## Manual checks

- Run real-model turns on each platform against Finder/Explorer/Files, a browser, and an Electron app. Check outline usefulness, size, and latency.
- Hang a target application and confirm emergency stop and takeover stay responsive.
- Compare the Settings row against the design-system specimen in light and dark themes, at a narrow width, and in zh-CN.

## Commands and results

2026-10-23, macOS development host:

- `cargo clippy --manifest-path agent/Cargo.toml --workspace --all-targets -- -D warnings`: clean.
- `cargo clippy -p suncode-computer --all-targets --target x86_64-pc-windows-msvc`: clean.
- `cargo clippy -p suncode-computer --all-targets --target x86_64-unknown-linux-gnu` with a typecheck-only `pkg-config` shim for `libdbus-sys`: clean. Not linked or run.
- `cargo test --manifest-path agent/Cargo.toml --workspace --all-targets --no-fail-fast`: 354 passed, 2 failed. Both failures (`policy::tests::full_control_allows_known_risks_but_not_unknown_tools`, `http::tests::extracts_error_message_or_falls_back_to_status`) reproduce on unmodified `main` and are unrelated.
- `cargo test -p suncode-computer --lib collects_the_focused_window -- --ignored`: passed against a live iTerm2 window.
- `cargo test --manifest-path sdks/rust/Cargo.toml --lib`: 39 passed. `sdks/c`: 4 passed. `apps/cli`: 22 passed.
- `dotnet test` (Settings, Localization, Runtime view models): 17 passed.
- `vite build` for `design-system/`: built.

## Residual risks

- Accessibility quality depends on each target application. Custom-drawn UIs may expose little or nothing.
- Wayland remains unsupported along with the rest of Computer Use.

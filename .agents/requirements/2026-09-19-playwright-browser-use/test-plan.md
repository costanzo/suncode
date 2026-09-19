# Test Plan

## Scope

Verify bundled-runtime identity, Rust ownership, browser tool semantics, approval boundaries, persistence isolation, SDK parity, Settings behavior, packaging, and failure recovery.

## Unit tests

- Runtime manifest path, target, version, and checksum validation.
- Framed JSON encode/decode and maximum message bounds.
- Browser state transitions and generation retirement.
- URL scheme/origin validation and structured locator validation.
- Snapshot truncation, reference revision checks, and result redaction.
- Browser-specific policy decisions and non-interactive denial.
- DTO serialization parity across Rust, C, and C#.

## Integration and conformance tests

- Launch the fixed worker and verify its handshake.
- Exercise a local fixture site through navigate, snapshot, click, fill, select, check, press, scroll, wait, console, screenshot, tabs, and close.
- Prove project profile isolation and persistence across clean restarts.
- Prove user-control handoff pauses actions and return-control invalidates references.
- Prove disable, cancellation, timeout, worker crash, Chromium crash, and repeated-crash failure behavior.
- Prove page prompt injection cannot create approval or authorize transmission.
- Prove browser tool rows and artifacts survive snapshot reload.

## Regression checks

- Existing built-in, MCP, LSP, provider, approval, recovery, and desktop tests remain green.
- Browser Use disabled leaves the existing tool catalog and startup behavior unchanged.
- No runtime package or browser download occurs after installation.
- Existing user changes in the desktop project file are preserved.

## Manual checks

- Settings at default and minimum size in light/dark themes.
- Long runtime paths, keyboard focus, screen-reader names, copy actions, disabled actions, and destructive confirmation.
- macOS window restore/takeover and notarized package.
- Windows foreground activation and signed installer.
- Linux X11/XWayland behavior plus explicit Wayland limitation and dependency errors.

## Commands and results

- `cargo test -p suncode-browser -p suncode-tool -p suncode-agent --lib`: passed on macOS arm64 (5 browser, 36 tool, and 48 agent tests).
- `cargo test -p suncode-sdk --lib`: passed on macOS arm64 (23 tests).
- `cargo test -p suncode-sdk-c --lib`: passed on macOS arm64 (2 tests, including ABI 9).
- `dotnet test apps/desktop-avalonia/tests/SunCode.Desktop.Tests.csproj --no-restore`: passed (106 tests).
- `npm --prefix design-system run build`: passed; Vite retained its existing large-chunk warning.
- macOS arm64 runtime package verification, Rust probe, and real headless Chromium navigation to `https://example.com`: passed for both the assembled runtime and the runtime copied into `SunCode.app`.
- The smoke test also verifies that a failed locator returns a stable timeout error without echoing the locator/page content.
- The packaged macOS runtime is 426 MiB, has relative internal Chromium/Playwright symlinks, and contains no probe scratch directory.
- Windows x64 and Linux x64 target-host package, offline smoke, signing, dependency, and installed-package checks: not run locally.

## Residual risks

- Platform window managers can limit reliable foreground activation.
- Browser package size and Linux distribution compatibility remain release-engineering risks until target artifacts pass smoke tests.
- Fine-grained origin and consequential-action classification is not implemented; every browser tool requires approval in the initial implementation.
- Screenshot artifacts are retained as PNGs, but tool-result images are not yet injected into provider vision input.

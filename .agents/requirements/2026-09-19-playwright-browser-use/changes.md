# Changes

## Source

- Add the Rust `suncode-browser` package and core `BrowserManager`.
- Add a bundled JavaScript Playwright worker and target-specific runtime manifest.
- Add browser tool schemas, validation, execution, cancellation, and audit integration.
- Add Browser use design-system and Avalonia Settings surfaces.

## Contracts and generated artifacts

- Add the framed Browser Worker protocol document.
- Add Rust/C/C# Browser Use runtime-management DTOs and named methods.
- Add a target-specific runtime lock/manifest and release verification report.
- No contract generation; language bindings remain hand-implemented.

## Configuration and persistence

- Seed global `browser_use_enabled=false` in `configuration`.
- Keep project browser profiles under the agent data directory.
- Keep runtime state and reported versions memory-only.
- Add browser-origin grants only with a separately reviewed persistent contract.

## Tests

- Rust protocol framing, manifest validation, process lifecycle, cancellation, and state-machine tests.
- Worker fixture-site tests for navigation, snapshots, locators, stale references, console, screenshots, and profile persistence.
- SDK binding parity tests.
- Design-system and Avalonia Settings state tests.
- Offline packaged-runtime smoke tests for all three targets.

## Documentation

- Update product, architecture, decision, SDK, persistence, tool, security, distribution, and design-system records.

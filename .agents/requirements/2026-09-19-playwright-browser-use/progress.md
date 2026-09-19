# Progress

- Status: In progress
- Last updated: 2026-09-19

## Completed

- Product direction confirmed for macOS arm64, Windows x64, and Linux x64.
- Default background operation, explicit user handoff, and project-isolated persistent profiles confirmed.
- Architecture, distribution, authority, UI, and verification design recorded.
- Fixed Node.js 24.11.1, Playwright 1.55.0, Chromium 140.0.7339.16 revision 1187, and FFmpeg revision 1011 in `runtime-lock.json`.
- Added target-native runtime assembly for macOS arm64, Windows x64, and Linux x64 with Node archive checksums, exact npm integrity, runtime tree hashes, relative-symlink enforcement, licenses, and rollback-safe replacement.
- Added the Rust browser protocol/process package, core BrowserManager, primary-session browser tool catalog, mandatory interactive approval policy, cancellation retirement, persistent project profiles, control handoff, and screenshot artifact retention.
- Added Rust/C/C# SDK management methods and ABI 9.
- Added the design-system and Avalonia Browser use Settings surfaces.
- Packaged, verified, probed, and smoke-tested the macOS arm64 runtime and the runtime copied into `SunCode.app`; the packaged runtime is 426 MiB and contains no absolute symlinks.
- Focused Rust, SDK, C ABI, Avalonia, and design-system builds/tests pass.

## In progress

- Windows x64 and Linux x64 target-host packaging, offline smoke tests, signing/dependency evidence, and installed-package checks.
- Broader Browser Use fixture coverage for profile persistence, handoff, cancellation/crash recovery, and all documented Settings states.
- Fine-grained origin/action policy preflight. The initial implementation deliberately requires approval for every browser call, including observations.
- A provider attachment path for screenshot artifacts; the first implementation retains PNGs but returns text metadata to the model.

## Blocked

- None.

## Log

### 2026-09-19

- Requirement initialized from the repository template.
- Approved first-party Playwright integration without MCP.

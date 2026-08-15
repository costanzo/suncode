# Progress

- Status: Complete
- Last updated: 2026-08-15

## Completed

- Confirmed that the runtime is an embedded SDK and will not provide a client-facing server or replacement IPC.
- Defined native Qt, future N-API, and future PyO3 binding boundaries.
- Defined typed service, domain error, C ABI ownership, and event subscription requirements.
- Replaced Axum request routing with named Rust SDK methods and public result DTOs.
- Added an ABI-versioned, method-oriented C boundary and migrated every Qt request site.
- Removed the runtime server binary, listener configuration, HTTP authentication, endpoint discovery, and generic `request_json` export.
- Fixed replay/live handoff ordering and lag recovery for direct subscriptions.
- Updated architecture, decisions, product descriptions, feature/specification notes, contracts, vectors, and SDK packaging direction.
- Passed the Rust workspace tests, Qt build, QML lint target, offscreen startup, formatting, JSON, symbol, dependency, and diff checks.

## In progress

- None.

## Blocked

- None.

## Log

### 2026-08-15

- Requirement initialized after the embedded-only SDK direction was approved.
- The implementation baseline includes the committed six-provider catalog from commit `3559efa`.
- The final production crate exposes only `rlib` and `staticlib` targets; no runtime executable target remains.
- `cargo clippy --workspace --all-targets` passes with one pre-existing `manual_clamp` warning in `context.rs` from the baseline provider-catalog work.

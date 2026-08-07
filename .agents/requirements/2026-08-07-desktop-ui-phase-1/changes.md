# Changes

## Source

- None. This delivery defines requirements only.

## Contracts

- None. `architecture.md` records the client-runtime API capabilities a later contract delivery must cover. Nothing is generated; this surface will hand-write its adapter against that contract document.

## Configuration and persistence

- None. The requirement distinguishes runtime-owned settings from device-local UI preferences but does not define schemas or storage.

## Tests

- Added a requirements review and future implementation test plan.

## Documentation

- Added the Phase 1 Qt desktop UI requirement package.
- Defined product scope, layout, interactions, accessibility, performance, privacy, failure handling, and acceptance criteria.
- Recorded open decisions and an architecture documentation discrepancy that must be resolved before implementation.

### Revision, 2026-08-07

- Recorded that desktop is the second surface, behind the CLI/TUI, and that implementation waits until the client API is proven.
- Renamed "task" to "session" and "workspace" to "project" throughout, per the vocabulary decision.
- Marked `project-window-amendment.md` accepted and normative, and rewrote the information-architecture section to defer to it rather than contradict it.
- Brought undo into scope with its own requirements section, replacing the clause that excluded discard and revert.
- Added an authority review section covering audit visibility and grant revocation.
- Added inspector and activity requirements for sibling tool calls from one assistant message.
- Replaced generated-SDK language with a hand-written adapter verified by shared test vectors.
- Resolved remote runtime selection as out of scope; split open questions into blocking, resolved, and non-blocking.
- Added test coverage for undo states and conflicts, authority attribution, sibling tool calls, and client/runtime version skew.

## Files intentionally not changed

- No source, contracts, features, or specs. The decisions this package depends on were recorded in `.agents/DECISIONS.md`.


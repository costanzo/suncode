# Progress

- Status: Draft
- Last updated: 2026-08-07

## Completed

- Defined the Phase 1 desktop user jobs and non-goals.
- Defined the responsive three-region layout and visual direction.
- Defined functional requirements for projects, sessions, conversation, composer, approvals, inspector, settings, diagnostics, and recovery.
- Documented presentation boundaries, proposed state ownership, and the required client-runtime API capability backlog.
- Adopted the project-per-window model and marked `project-window-amendment.md` accepted and normative, with `requirement.md` pointing to it instead of restating it.
- Aligned vocabulary on project, session, and turn, retiring the separate UI-facing "task" and "workspace" terms.
- Recorded that the desktop client is the second surface; the CLI/TUI ships first and proves the client API.
- Brought undo into scope as a first-class surface rather than a deferred destructive action, with honest wording about external side effects.
- Added an authority review surface so users can see what was authorized and what resulted.
- Added activity requirements for several sibling tool calls from one assistant message.
- Replaced generated-SDK assumptions with a hand-written protocol adapter verified against shared test vectors.
- Resolved remote runtime selection as out of scope under the local-first decision.
- Defined acceptance criteria and a future verification strategy.

## In progress

- Product and architecture review.

## Blocked

- Implementation is intentionally not started, and should not begin until the CLI/TUI has exercised the client API.
- The client-runtime contract document, the OS and Qt technology decisions, and the adapter-sharing question must be settled first.

## Log

### 2026-08-07

- Requirement package initialized as documentation-only work.
- Recorded the stale ownership summary in `../../DECISIONS.md` as a discrepancy to resolve before implementation. Now resolved by `ADR-20260807-runtime-owns-durable-state`.
- Revised after architecture review. The substantive changes were bringing undo and authority review into scope, since neither can be retrofitted, and reordering this surface behind the CLI so the client API is proven by a cheaper client first.

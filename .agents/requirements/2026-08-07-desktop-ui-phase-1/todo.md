# Todo

## Product decisions

- [ ] Approve the Phase 1 primary workflow and explicit non-goals.
- [x] Remote runtime selection is out of scope; Suncode is local-first and the client attaches via the discovery file.
- [x] Desktop is the second surface; the CLI/TUI ships first and proves the client API.
- [x] Undo is in scope and exposed in the UI rather than deferred as a destructive action.
- [ ] Define supported desktop operating systems and minimum versions.
- [ ] Approve undo wording, including how it states that external side effects are not reversed.
- [ ] Decide how undo is presented when a checkpoint has partially expired.
- [ ] Decide session deletion semantics, draft retention, and background behavior after the client closes.
- [ ] Define model and interaction modes exposed in the first release.
- [ ] Approve notification defaults and privacy behavior.

## Design

- [ ] Create annotated wide, medium, and narrow wireframes.
- [ ] Define Suncode visual tokens for color, typography, spacing, density, icons, focus, and motion.
- [ ] Specify all empty, loading, pending, error, offline, and recovery states.
- [ ] Review the approval experience with the security owner.
- [ ] Design the authority review surface: what was authorized, by whom or which profile, and what resulted.
- [ ] Design activity rendering for several sibling tool calls from one assistant message.
- [ ] Validate keyboard navigation, screen-reader behavior, scaling, and IME handling.

## Architecture and contracts

- [ ] Reconcile the stale `ADR-20260804-foundational-architecture` summary with the approved architecture.
- [ ] Select Qt Quick/QML or Qt Widgets after a focused proof of concept.
- [ ] Specify authenticated runtime discovery and client handoff.
- [ ] Define the client-runtime contract document and its shared test vectors.
- [ ] Define event replay, snapshot fallback, idempotency, and multiple-client conflict behavior.
- [ ] Define safe Markdown, link, attachment, clipboard, notification, and external-editor policies.
- [ ] Establish concrete performance limits and reference hardware.

## Implementation

- [ ] Create the Qt desktop project only after this requirement is approved.
- [ ] Implement the hand-written protocol adapter and deterministic view models.
- [ ] Implement the application surfaces in the order described by `plan.md`.
- [ ] Add focused and cross-platform tests.

## Closeout

- [ ] Promote implemented stable behavior into `.agents/features/`.
- [ ] Publish current UI/runtime contracts and UI architecture in `.agents/specs/` or the repository documentation paths required by `../../ARCHITECTURE.md`.
- [ ] Record durable Qt and client-state decisions in `../../DECISIONS.md`.


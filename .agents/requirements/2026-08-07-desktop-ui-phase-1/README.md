# Phase 1 Qt Desktop UI

- Date: 2026-08-07
- Status: Draft
- Related features: None; the desktop UI is not implemented.
- Related specifications: `../../ARCHITECTURE.md`
- Related decisions: `../../DECISIONS.md` — `ADR-20260807-domain-vocabulary`, `ADR-20260807-hand-written-protocol-contracts`, `ADR-20260807-local-first-scope`
- Related requirements: `../2026-08-07-agent-runtime-phase-1/README.md`

## Summary

Define the Suncode desktop experience as a focused, Codex-inspired Qt application for opening projects, managing agent sessions, reviewing streamed work, responding to approvals, undoing changes, and configuring the client. This delivery is requirements and design work only; it does not authorize or claim an implementation.

"Codex-inspired" means adopting the useful interaction model of a quiet desktop workbench organized around sessions and their activity. Suncode must use its own name, assets, visual tokens, wording, and implementation rather than copying OpenAI branding or treating undocumented Codex behavior as a contract.

## Scheduling

Per `PRODUCT.md`, the desktop application is the **second** committed surface; the CLI/TUI ships first and proves the client API. Schedule this package after that, and expect the API to exist and to have been exercised by a real client.

There is no generated client SDK. This surface hand-writes its transport adapter against the client-runtime contract document and its shared test vectors.

## Documents

- `requirement.md` - product behavior, layout, states, and acceptance criteria
- `project-window-amendment.md` - **normative** for window and navigation structure; governs where `requirement.md` differs
- `architecture.md` - UI boundaries, state model, and runtime dependencies
- `changes.md` - documentation changed by this delivery
- `plan.md` - proposed later implementation sequence
- `progress.md` - current documentation status
- `todo.md` - decisions and future implementation work
- `test-plan.md` - requirement validation and future UI verification
- `project-window-review-status.md` - review record for the amendment


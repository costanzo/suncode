# Architecture

## Current state

`design-system/src/components/universal/modal/Modal.jsx` owns the reusable modal shell, focus trap, Escape handling, backdrop dismissal, and focus restoration. `SessionPanel` in `WorkspacePrimitives.jsx` owns local session specimen behavior and currently removes archived sessions immediately.

## Proposed design

Add `ConfirmationDialog.jsx` beside the existing modal. It composes `Modal` with a safe initial-focus marker and standardized cancel/confirm actions while allowing callers to supply decision copy and target content. Update `Modal` to generate unique accessible identifiers so several dialogs can coexist in one review page.

SessionPanel stores the pending archive index separately from its session collection. Opening Archive records the target and displays the shared confirmation dialog. Cancel clears the pending target; confirmation removes it from the local review list.

## Boundaries and dependencies

- Only `design-system/`, `DESIGN.md`, and this delivery record are in scope.
- Reuse existing universal modal, button, token, and danger-action language.
- Do not change Avalonia, Rust, SDK contracts, or persistence.

## Data and control flow

- The session menu sets `archiveIndex` and closes itself.
- A non-null archive index opens `ConfirmationDialog` and supplies the matching session title.
- Dismissal resets `archiveIndex` without changing `items`.
- Confirmation filters the target from `items`, restores the specimen selection to its default, and closes the dialog.

## Security and failure handling

This is a design-only local state simulation and does not perform a real archive operation. The safe default focus and dismiss-without-commit behavior reduce accidental confirmation in the reviewed interaction.

## Compatibility and migration

Existing modal consumers continue to use `Modal`. The new component is additive, while unique dialog IDs improve accessibility for all modal specimens.

## Risks and rollback

The primary risk is making a reversible archive action feel overly destructive. The dialog copy explains that the session can be reopened, while the danger action still communicates that it leaves the active list. Rollback is limited to design-system source and documentation.

## Open questions

- None.

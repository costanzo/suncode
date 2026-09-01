# Requirement

## Background

The design system has a reusable modal shell, but consequential actions do not have a dedicated confirmation component. Session Archive currently acts immediately in the review specimen, leaving no opportunity to verify the target or cancel the operation.

## Goals

- Add a reusable confirmation dialog to the universal component catalog.
- Require a second explicit decision before a session is archived in the design-system specimen.
- Add a dedicated Sessions review state for the archive confirmation dialog.
- Keep the delivery limited to the React design review browser and design authority documents.

## Non-goals

- Change the Avalonia desktop client.
- Change the Rust agent, SDK contracts, persistence, or session archive behavior.
- Define confirmation requirements for every product action in this delivery.

## Requirements

- The confirmation dialog must compose the existing universal modal shell rather than duplicate its focus and dismissal behavior.
- The component must accept title, consequence description, optional target content, cancel and confirmation labels, action styling, and callbacks.
- Cancel must receive initial keyboard focus.
- Escape, close, backdrop dismissal, and Cancel must dismiss the dialog without committing the action.
- Choosing Archive from a session menu must open the confirmation dialog and name the affected session.
- Only the explicit Archive session button may remove the session from the active specimen list.
- The Sessions page must include a dedicated review state with the archive confirmation visible.

## Edge cases

- Multiple modal instances on one review page must have unique accessible title and description identifiers.
- A long session title must truncate safely inside the dialog target block.
- Canceling the dialog must preserve the current session list and selection.
- The dialog must remain usable at narrow review-browser widths and in both themes.

## Acceptance criteria

- A reusable `ConfirmationDialog` is exported from the universal modal module.
- The universal overlays specimen demonstrates the confirmation component.
- Session Archive opens a second-step confirmation instead of removing the item immediately.
- The Sessions review page includes an archive-confirmation state.
- `npm run build` succeeds in `design-system/`.
- `git diff --check` succeeds.

## Open questions

- None.

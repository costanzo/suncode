# Architecture

## Current state

`design-system/src/projects/desktop/settings/index.jsx` owns the Network settings specimen inline. It currently exposes only the HTTPS verification toggle and warning state. `design-system/src/styles/review.css` owns the shared settings styling primitives, including the existing directory-style path field.

## Proposed design

Extend the Network settings specimen with local React state for:

- HTTPS verification enabled or disabled
- system certificate usage enabled or disabled
- custom certificate path draft

Render a subordinate certificate-source group when HTTPS verification remains enabled. Reuse the existing settings field structure and path-picker visuals, but adapt the selector to a file-oriented certificate picker that matches the intended `SCFileSelector` behavior.

## Boundaries and dependencies

- Only the design-system React specimen, settings styles, and design authority documents are in scope.
- No Avalonia, Rust, SQLite, or SDK changes are included.

## Data and control flow

- Local component state controls the verify toggle, system-certificate toggle, and path draft.
- A file input simulates the `SCFileSelector` browse interaction in the review browser.
- The save action remains specimen-only and does not persist beyond local UI state.

## Security and failure handling

This is a design-only simulation. No certificate files are opened or trusted by the app.

## Compatibility and migration

The change extends the existing Network panel without altering route structure or provider settings.

## Risks and rollback

The main risk is visual drift from the Avalonia `SCFileSelector` pattern. Reusing the existing field-with-browse-button structure keeps the review specimen close to the intended desktop control.

## Open questions

- None.

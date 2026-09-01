# Test Plan

## Scope

Verify the design-only universal confirmation dialog and session archive confirmation state.

## Unit tests

- None. The design-system does not currently have focused automated interaction tests for these specimens.

## Integration and conformance tests

- Build the design-system review browser successfully.
- Verify formatting with the configured Prettier check.

## Regression checks

- Confirm create and rename session modals still render.
- Confirm existing modal consumers continue to build with unique accessible IDs.
- Confirm the session list is unchanged after cancellation and changes only after confirmation.

## Manual checks

- Inspect the archive-confirmation state on the Sessions review page.
- Confirm Cancel receives initial focus.
- Confirm Escape, close, backdrop, and Cancel dismiss without archiving.
- Confirm Archive session removes the named session.
- Inspect narrow layout and both themes.

## Commands and results

- `npm run format:check` could not run because the current install lacks the Prettier binary; `npm install` could not restore it because the configured package mirror returned `SELF_SIGNED_CERT_IN_CHAIN`.
- `npm run build` passed.
- `git diff --check` passed.
- Local browser review passed for cancellation, confirmation, safe initial focus, dark theme, and a 620px-wide viewport.

## Residual risks

- The specimen verifies local React behavior only and does not prove the future production archive call or recovery flow.

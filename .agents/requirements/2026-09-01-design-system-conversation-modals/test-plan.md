# Test Plan

## Scope

Verify the design-only conversation additions in the React review browser.

## Unit tests

- None. The design-system does not currently have focused automated UI tests for these specimens.

## Integration and conformance tests

- Build the design-system review browser successfully.

## Regression checks

- Confirm existing conversation states still render.
- Confirm existing attachment preview and tool detail modals still render.

## Manual checks

- Open the Conversation review page and inspect the expanded composer state.
- Open the Conversation review page and inspect the live tool-output state.
- Open the Conversation review page and inspect the thinking state.
- Verify the live character count and streaming output presentation in both themes.

## Commands and results

- `npm run build`
- `git diff --check`

## Residual risks

- The live command output is simulated in the design review browser and does not prove production streaming behavior.

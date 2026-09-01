# Test Plan

## Scope

Verify the design-only Network settings certificate-source additions in the React review browser.

## Unit tests

- None. The design-system does not currently have focused automated UI tests for this specimen.

## Integration and conformance tests

- Build the design-system review browser successfully.

## Regression checks

- Confirm the existing HTTPS verification warning still renders when verification is disabled.
- Confirm settings navigation and provider pages still render.

## Manual checks

- Open Settings -> Network and inspect verification enabled and disabled states.
- Toggle system certificates on and off and confirm the certificate path field disables and enables accordingly.
- Use the browse button in custom mode to simulate certificate-file selection.

## Commands and results

- `npm run build`
- `git diff --check`

## Residual risks

- The review browser simulates the intended `SCFileSelector` file-picking interaction but does not prove Avalonia behavior.

# Test Plan

## Scope

Design-system route registration, fixture composition, semantic states, and responsive layout.

## Unit tests

None; the review browser has no unit-test harness for static specimens.

## Integration and conformance tests

None; no API or crypto code is introduced.

## Regression checks

- Existing design-system routes continue to build.
- Web child routes resolve through the hash router.

## Manual checks

- Web shell, pairing, Session, and security pages in light and dark themes.
- Loading/empty/degraded/approval/question states.
- Width below 760px and at the compact conversation minimum.
- Keyboard focus for tabs and actions.

## Commands and results

- `npm run build` (from `design-system/`) — passed; Vite emitted only the existing chunk-size advisory.
- `git diff --check` (repository root) — passed.
- Manual route review — Web shell, pairing, Session, and security routes resolve through the hash router; responsive CSS includes 1200px, 900px, and 680px behavior.

## Residual risks

Visual fixtures do not verify eventual WebCrypto, SSE, token refresh, or API behavior.

# Test Plan

## Scope

Design-system UI only. Real OS access and production behavior are not verified by this review.

## Unit tests

No new unit tests for the reversible review-only change.

## Integration and conformance tests

Run Vite build and validate routes, in-page provider state, draft cancellation, masked replacement, and confirmations in the browser.

## Regression checks

Provider endpoint controls and existing Settings navigation remain accessible.

## Manual checks

Both themes; default and constrained sizes; native radio keyboard selection; shared dialog focus and Escape; empty, configured, plaintext, loading, access failures, missing entry, save/remove failure; successful changes survive provider navigation.

## Commands and results

- `npm run build`: passed; bundle-size advisory remains.
- `git diff --check`: passed.
- Browser verification: in progress.

## Residual risks

Native store permissions, signing, headless environments, and durability are deferred to approved production implementation.

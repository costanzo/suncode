# Test Plan

## Scope

Design-system catalog and vtsls detail review in this phase. Production verification is deferred until approval.

## Unit tests

- None required for fixture-only design work.

## Integration and conformance tests

- Design-system production build and route rendering.

## Regression checks

- Existing jdtls catalog and detail routes remain functional.
- Unscoped catalog disables both entries.

## Manual checks

- Catalog with two server rows.
- vtsls default, custom TypeScript SDK, advanced, dirty, reset, save, indexing, ready, failed, and retry states.
- Light and dark themes at desktop and constrained widths.
- Keyboard focus, long paths, and overflow.

## Commands and results

- `npm run build`: passed; Vite retained the existing bundle-size warning.
- Targeted `prettier --check`: passed.
- `git diff --check`: passed.
- Interactive visual review remains pending user confirmation.

## Residual risks

- The fixture models proposed typed configuration and does not prove the later SDK contract or local vtsls compatibility.

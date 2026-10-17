# Test Plan

## Scope

Design-system catalog and basedpyright detail review in this phase.

## Unit tests

- None required for fixture-only design work.

## Integration and conformance tests

- Design-system production build and direct route rendering.

## Regression checks

- Existing jdtls and vtsls routes remain functional.
- Unscoped catalog disables all three entries.

## Manual checks

- Catalog with three server rows.
- basedpyright default, custom configuration path, dirty, reset, save, indexing, ready, failed, and retry states.
- Light and dark themes at desktop and constrained widths.
- Keyboard focus, long paths, and overflow.

## Commands and results

- `npm run build`: passed; Vite retained the existing bundle-size warning.
- Targeted `prettier --write`: completed with no formatting changes.
- Headless Chrome screenshots: desktop catalog, desktop detail, and narrow detail passed visual inspection.
- `git diff --check`: passed.

## Residual risks

- The fixture will not prove interpreter discovery or a live basedpyright protocol handshake.

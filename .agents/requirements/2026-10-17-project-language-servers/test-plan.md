# Test Plan

## Scope

Design-system prototype now; production catalog, persistence, runtime, bindings, and Avalonia behavior after design approval.

## Unit tests

- Deferred until production implementation.

## Integration and conformance tests

- Deferred until production implementation.

## Regression checks

- Design-system production build.
- Existing Settings navigation and unrelated panels remain available.

## Manual checks

- Catalog overview and project context.
- jdtls detail, data-directory modes, advanced settings, and runtime transitions.
- Dark and light themes at desktop and constrained widths.
- Keyboard focus and text overflow.

## Commands and results

- `npm run build`: passed; Vite retained its existing bundle-size warning.
- Targeted `prettier --check`: passed for every changed design-system and requirement file.
- `git diff --check`: passed.
- Headless Chromium screenshots: passed for light desktop, dark desktop, and constrained-width detail views.
- CDP interaction check: configure navigation, custom data path, advanced disclosure, indexing, and ready transitions passed.
- Repository-wide `npm run format:check`: still reports unrelated pre-existing formatting findings outside this change.

## Residual risks

- The prototype uses fixture paths and states; it does not validate the eventual SDK contract.

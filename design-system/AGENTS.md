# Design System Agent Instructions

These instructions apply to work under `design-system/`. Repository-level instructions remain authoritative.

## Before Changes

- Read the root `AGENTS.md` and [`DESIGN.md`](../DESIGN.md).
- Read [`README.md`](README.md) and [`CONTRIBUTING.md`](CONTRIBUTING.md) before adding a route, module, component, asset, or style.
- Inspect the existing route and a representative neighboring implementation before creating a new surface.
- Preserve existing user changes and keep edits limited to the design-system review surface unless the request explicitly includes a production client.

## Ownership

- `src/styles/tokens/` is the only source for semantic colors, typography, spacing, radii, control dimensions, and theme values.
- `src/styles/foundation.css` owns document reset and global focus behavior.
- `src/styles/layout.css` owns small reusable layout primitives.
- `src/components/universal/<component>/` owns universal component specimens and colocated component CSS.
- `src/core/pages/` owns Core review pages and their page styles.
- `src/platforms/` owns platform adaptations and platform-specific components.
- `src/projects/` owns product/project surfaces. Desktop Workspace styles belong under `src/projects/desktop/workspace/styles/`.
- `src/styles/browser.css` owns the catalog browser shell. Do not add new component or project rules there.
- `src/styles/review.css` is a compatibility entrypoint for remaining shared review rules. Do not add new component or page rules there.

## Verification

After a UI or interaction change:

1. Run `npm run build` from `design-system/`.
2. Run `git diff --check` from the repository root.
3. Review the affected hash route in both themes and at a constrained width when the change affects layout.
4. Check keyboard focus, disabled/error/empty/loading states, and overflow where applicable.

Do not claim a design-system change is complete when the review route, build, or required state coverage has not been checked.

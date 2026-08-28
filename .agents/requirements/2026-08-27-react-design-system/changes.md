# Changes

## Source

- Added Vite/React application scaffolding under `design-system/`.
- Added the shared browser shell and route catalog under `src/app/`; all React source now follows the conventional `src/` application tree.
- Added layer-owned React pages under `components/`, `platforms/`, and `projects/`.
- Added an Avalonia-aligned Desktop Workspace overview with independently routed Sessions, Explorer, Conversation, Review, Source control, and Provider trace modules.
- Made the contextual sidebar and search catalog recursively index nested route groups.
- Replaced standalone review HTML pages with hash routes.

## Contracts and generated artifacts

- No production protocol contract changes.
- `design-system/dist/` is generated and ignored.

## Configuration and persistence

- Theme choice is stored locally in the review browser only.
- No production configuration or persistence changes.

## Tests

- Production build passed.
- Browser route, theme, desktop, and narrow-viewport checks passed.
- All seven Workspace routes passed direct navigation, active-state, interaction, console, and horizontal-overflow checks.
- Design detector and repository whitespace validation passed.

## Documentation

- Updated `design-system/README.md`, root `README.md`, and `DESIGN.md` to describe the React review browser and its tooling-only boundary.

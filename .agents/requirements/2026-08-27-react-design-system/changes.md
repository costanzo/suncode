# Changes

## Source

- Added Vite/React application scaffolding under `design-system/`.
- Added the shared browser shell and route catalog under `core/react/`.
- Added layer-owned React pages under `components/`, `platforms/`, and `projects/`.
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
- Design detector and repository whitespace validation passed.

## Documentation

- Updated `design-system/README.md`, root `README.md`, and `DESIGN.md` to describe the React review browser and its tooling-only boundary.

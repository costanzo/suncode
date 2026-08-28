# Architecture

## Current state

The design catalog is a set of linked static HTML pages sharing token CSS, assets, and a review stylesheet. Content ownership already follows four layers, but navigation state and theme selection do not span pages.

## Proposed design

Use a small Vite/React application with `index.html` and `src/main.jsx` as the executable entry. A shell under `core/react/` owns global navigation, responsive behavior, theme state, and hash routing. Each existing ownership layer exports its own page components directly from that layer:

- `core/react/` — application shell, routing metadata, and foundational review pages.
- `components/universal/react/` — shared component inventory and specimens.
- `components/platform-specific/react/` — platform-only component boundary index.
- `platforms/*/index.jsx` — platform adaptation pages.
- `projects/avalonia-desktop/index.jsx` — business-project mapping.

## Boundaries and dependencies

React, React DOM, and Vite are dependencies of `design-system/` only. They are not referenced by the Avalonia application, Rust workspace, native SDK facade, or production packaging. Shared design tokens and approved assets remain plain files under `core/`.

## Data and control flow

The root shell reads `window.location.hash`, resolves a route from a static catalog, and renders the owning layer's page component. Navigation writes hashes. Theme state updates the root `data-theme` attribute and is stored in local storage when available.

## Security and failure handling

The browser contains no credentials, provider access, filesystem access, or production data. Unknown routes render a local not-found state. Local storage failures are non-fatal and fall back to the light theme.

## Compatibility and migration

The old standalone HTML review pages are replaced after their content is migrated into React specimens. The token and asset paths remain stable. The build uses relative Vite output paths so it can be hosted below any static path.

## Risks and rollback

The primary risk is losing state coverage during migration. The React inventory therefore preserves the categories and representative states from both static theme pages. Rollback is limited to the isolated `design-system/` tool and does not affect production binaries.

## Open questions

- None.

# Architecture

## Current state

The design catalog is a set of linked static HTML pages sharing token CSS, assets, and a review stylesheet. Content ownership already follows four layers, but navigation state and theme selection do not span pages.

## Proposed design

Use a conventional Vite/React application with `index.html` and `src/main.jsx` as the executable entry. React source is organized under `src/`: `app/` owns global navigation, responsive behavior, theme state, and hash routing; `core/pages/` owns foundational review pages; `components/` owns component pages and specimens; `platforms/` and `projects/` own their review pages; `shared/` owns reusable primitives; `styles/` owns browser CSS and tokens; and `assets/` owns source-imported resources. Navigation has three levels: a name-only primary module switcher in the upper-right for Core, Components, Platforms, and Projects; a contextual left sidebar containing only the active module's submodules; and expandable child entries beneath each submodule. Child entries may target page sections for compact catalogs, but every Universal component category owns a separate route and page so its content can grow independently.

- `src/styles/tokens/` — semantic color, typography, and spacing sources.
- `src/assets/` — source-imported logos, icons, fonts, and platform resources.
- `public/assets/` — browser-direct static files such as the favicon.
- `src/app/` — application shell and route metadata.
- `src/components/universal/<component>/` — component-owned specimens, documentation, and stable exports.
- `src/components/universal/modules/<module>/` — independently routable category pages that compose component specimens.
- `src/components/universal/UniversalComponentsPage.jsx` — universal module index.
- `src/components/platform-specific/` — platform-only component boundary index.
- `src/platforms/*/index.jsx` — platform adaptation pages.
- `src/projects/avalonia-desktop/index.jsx` — business-project mapping.

## Boundaries and dependencies

React, React DOM, and Vite are dependencies of `design-system/` only. They are not referenced by the Avalonia application, Rust workspace, native SDK facade, or production packaging. Shared design tokens and approved assets remain plain files under the conventional `src/styles/tokens/`, `src/assets/`, and `public/assets/` boundaries.

Universal component markup and interaction state stay co-located with their documentation under `components/universal/<component>/` and are exported through that directory's `index.js`. Category pages under `components/universal/modules/<module>/` import and compose the relevant specimens; `UniversalComponentsPage.jsx` links to those modules and does not own their implementations.

## Data and control flow

The root shell reads `window.location.hash`, resolves a route from a static catalog, derives the active primary module, and renders the owning layer's page component. Primary navigation writes the default child route for a module; first-level sidebar links write routes within that active module; Universal second-level links write distinct category routes, while compact catalogs may still use stable section IDs. Theme state updates the root `data-theme` attribute and is stored in local storage when available.

## Security and failure handling

The browser contains no credentials, provider access, filesystem access, or production data. Unknown routes render a local not-found state. Local storage failures are non-fatal and fall back to the light theme.

## Compatibility and migration

The old standalone HTML review pages are replaced after their content is migrated into React specimens. Token and asset references migrate into the conventional React source tree; the obsolete root `core/` directory is removed. The build uses relative Vite output paths so it can be hosted below any static path.

## Risks and rollback

The primary risk is losing state coverage during migration. The React inventory therefore preserves the categories and representative states from both static theme pages. Rollback is limited to the isolated `design-system/` tool and does not affect production binaries.

## Open questions

- None.

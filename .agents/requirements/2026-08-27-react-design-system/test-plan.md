# Test Plan

## Scope

The isolated React design browser, its route catalog, global theme state, responsive shell, and local asset references.

## Unit tests

No dedicated unit framework is required for this isolated review application. Route resolution and theme fallback are exercised through browser checks.

## Integration and conformance tests

- Build the Vite application in production mode.
- Open representative hash routes from the built site.
- Confirm route changes do not reload the document.
- Confirm theme choice applies globally and persists across reload.

## Regression checks

- Confirm existing token CSS and approved assets load.
- Confirm all four ownership layers appear in navigation.
- Confirm mobile and TUI are labeled deferred.
- Confirm component samples retain the established semantic categories and states.

## Manual checks

- Desktop viewport: navigation, content hierarchy, hover, focus, and theme switch.
- Narrow viewport: navigation drawer, content flow, tables, and controls.
- Unknown route: useful recovery state.

## Commands and results

- `npm run build` — passed with Vite 7.3.6; 44 modules transformed.
- Playwright desktop check at 1440×1000 — passed: nine navigation entries, module search, theme persistence, no console errors, and zero horizontal overflow.
- Playwright mobile check at 390×844 — passed: drawer opens, dialog renders, no console errors, and zero horizontal overflow.
- Unknown hash check — passed with a recoverable `Module not found` page.
- Post-review interaction check — passed: mobile search opens and focuses, dialog focus/Escape works, and tabs respond to arrow keys with the correct tabpanel.
- Theme contrast check — passed: muted text is 4.56:1 on the light canvas and 5.40:1 on the dark canvas.
- `dotnet test apps/desktop-avalonia/tests/SunCode.Desktop.Tests.csproj --no-restore` — passed, 45/45 tests.
- Two-level navigation check at 1440×1000, 1024×768, and 390×844 — passed: four primary modules, contextual sidebar contents, mobile module menu, route changes, zero horizontal overflow, and no console errors.
- Three-level navigation check at 1440×1000 and 390×844 — passed: name-only primary modules, expandable submodules, section scrolling, active section styling, the same hierarchy in the mobile drawer, zero horizontal overflow, and no console errors.
- Component-ownership refactor check at 1440×1000 — passed: nine sections and nineteen specimens render from component-owned modules; toggle state, modal focus/Escape/restore, tab keyboard navigation, zero horizontal overflow, and console output remain clean.
- Conventional React source-tree check — passed: zero-warning production build, all nine routes, desktop/mobile navigation, component interactions, static asset URLs, zero `react/` source directories, zero horizontal overflow, and clean console output.
- Conventional resource-boundary check — tokens load from `src/styles/tokens/`, source-imported assets are emitted from `src/assets/`, the favicon loads from `public/assets/`, and no root `core/` directory or stale source path remains.
- Post-resource-migration browser check — passed all nine routes at 1440×1000 with successful images and favicon, zero failed requests or console errors, and zero horizontal overflow; the 390×844 Core Assets route also passed drawer, image, and overflow checks.
- Universal route split check — passed: the index and all nine category paths render one module each with the correct heading and sidebar route state; direct mobile navigation closes the drawer and reaches the selected page; toggle, modal focus/Escape, and tab keyboard behavior remain intact; no console errors or horizontal overflow were observed.
- Universal index card check — passed with nine cards in three desktop columns, two tablet columns, and one mobile column; the index has no Ownership block, detail pages retain it, card navigation works, and all checked viewports remain free of console errors and horizontal overflow.
- `node .agents/skills/impeccable/scripts/detect.mjs --json ...` — passed with no findings.
- `git diff --check` — passed.

## Residual risks

- Browser screenshots validate the review surface, not Avalonia runtime rendering.
- Automated checks do not replace manual screen-reader verification.

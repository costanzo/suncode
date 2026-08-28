# Requirement

## Background

The layered `design-system/` catalog is structurally clear, but separate static HTML pages make modules hard to discover and compare. Reviewers need one entry that keeps the four ownership layers visible while making all approved and deferred material easy to browse.

## Goals

- Convert `design-system/` into a React design-review application.
- Preserve the `core/`, `components/`, `platforms/`, and `projects/` ownership model.
- Make every catalog module reachable from persistent navigation at the root entry.
- Preserve the complete universal component and state inventory.
- Provide a global light/dark review switch, with light as the comfortable first visit.

## Non-goals

- Add React, Node.js, Bun, Vite, or a web client to the Phase 1 production runtime.
- Change the Avalonia or Rust production architecture.
- Present mobile, TUI, web, or CLI clients as implemented.
- Generate component runtime code for Avalonia.

## Requirements

- Use one Vite/React entry at `design-system/index.html`.
- Use hash routes so a static build works without rewrite configuration.
- Group navigation by the four design-system ownership layers.
- Present Core, Components, Platforms, and Projects as the four primary modules in the upper-right navigation.
- Keep the primary module switcher compact by showing module names without descriptions.
- Show the active module's submodules and expandable page-section links in the left sidebar instead of a page-level horizontal section menu.
- Give every Universal component category its own stable route and page module; keep `/components/universal` as an index rather than a combined scrolling inventory.
- Keep the left sidebar contextual: it shows only the active primary module's submodules, leaving room for future module growth.
- Expose foundations, assets, universal components, desktop adaptations, deferred platforms, and the Avalonia project mapping.
- Keep semantic status labels honest: implemented, review reference, reserved, or deferred.
- Use existing semantic CSS tokens and approved assets rather than adding an unrelated visual language.
- Persist the selected theme locally without forcing dark appearance on the index.
- Support keyboard navigation, visible focus, and narrow-screen navigation.

## Edge cases

- Unknown hashes fall back to a useful not-found page with a return path.
- Directly opening the built `index.html` must still render route navigation.
- Deferred platform pages must explain their boundary without inventing unsupported component libraries.
- The app must remain understandable when local storage is unavailable.

## Acceptance criteria

- All catalog areas are reachable from the root navigation without opening separate HTML documents.
- Switching a primary module updates the sidebar to that module's submodules on desktop and narrow screens.
- Universal component samples cover foundations, controls, surfaces, navigation, feedback, data, and Markdown.
- Each universal component directory owns its React specimen and stable export; the universal page composes those modules rather than centralizing their markup and interaction state.
- Actions, Fields, Selection, Surfaces, Overlays, Navigation, Feedback, Data, and Markdown are directly reachable as separate Universal routes from the sidebar.
- Keep all React application source under `design-system/src/`, organized by product responsibility; do not introduce `react/` directories as an architectural layer.
- Keep design token sources under `design-system/src/styles/tokens/`, source-imported visual assets under `design-system/src/assets/`, and browser-direct assets under `design-system/public/assets/`; do not retain a duplicate root `core/` source tree.
- Light and dark themes apply to the same content and retain semantic meanings.
- Desktop and narrow viewport layouts are usable.
- `npm run build` succeeds inside `design-system/`.
- Documentation states that this is design-review tooling and not a production dependency.

## Open questions

- None for this delivery.

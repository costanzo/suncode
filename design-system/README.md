# SunCode Design System

This directory is the React review browser and resource catalog for the SunCode visual language. It provides one navigable entry for foundations, components, platform adaptations, and project mappings.

React, Vite, and Node.js are used only to build this design-review tool. They are not dependencies of the Phase 1 Avalonia desktop application or embedded Rust agent.

## Files

- `index.html` and `src/main.jsx` - the single Vite/React entry.
- `src/app/` - application shell, hash routing, and navigation metadata.
- `src/core/pages/` - Core review pages.
- `src/components/` - component-owned React specimens and platform-specific component pages.
- `src/platforms/` and `src/projects/` - platform and business-project review pages.
- `src/shared/` - shared React primitives.
- `src/styles/tokens/` and `src/styles/` - semantic token sources and browser styles.
- `src/assets/` - source-imported logos, icons, fonts, and platform assets.
- `public/assets/` - browser-direct assets such as the favicon.

Install and start the review browser from this directory:

```sh
npm install
npm run dev
```

Create a static build with:

```sh
npm run build
```

The browser uses hash routes, so the generated `dist/index.html` can be hosted at a static subpath without server rewrite rules. The first visit uses the light theme; the global switch persists the selected light or dark appearance locally.

## Source Of Truth

The React routes are the visual contract for review. The upper-right navigation switches between the four primary layers—Core, Components, Platforms, and Projects—and intentionally shows only their names. The left sidebar then exposes the active layer as an expandable tree. Components expands Universal into independently routable Actions, Fields, Selection, Surfaces, Overlays, Navigation, Feedback, Data, and Markdown pages; the Universal root is a module index, not one long scrolling inventory. All React source is organized under `src/`; each component folder owns its implementation and `index.js` export, while `src/components/universal/modules/` owns category pages that compose those specimens. Token sources live under `src/styles/tokens/`; `src/styles/review.css` and `src/styles/browser.css` carry shared review primitives and the catalog shell. Avalonia runtime resources in `apps/desktop-avalonia/App.axaml` must keep the same semantic names and values.

Feature views must consume semantic resources such as `AccentBrush`, `SurfaceRaisedBrush`, `TextSecondaryBrush`, `WarningSurfaceBrush`, and `DangerBorderBrush`. A view should not introduce a new raw color, shadow, radius, or control height without first adding it to the design tokens and both review pages.

## Asset Rules

- Put source-imported product images, logos, illustrations, and reusable icons in `src/assets/`.
- Reserve `public/assets/` for files that must be addressed directly by the browser, such as the favicon.
- Give each asset a descriptive, stable name and record its role in `src/assets/README.md`.
- Prefer SVG for interface icons and PNG for raster brand marks or photos.
- Do not add decorative gradients, glow effects, or unlicensed stock imagery.
- When a runtime client needs an asset in its own packaging boundary, copy the approved asset into that client's build input and keep the design catalog as the review reference.

## Review Checklist

- Does the component use a named token rather than a local color?
- Are rest, hover, pressed, focus, disabled, loading, error, and success states represented where applicable?
- Is the primary action visually distinct without turning the whole surface into a dashboard?
- Is keyboard focus visible and is icon-only UI labeled?
- Does the component preserve the conversation-first hierarchy in a wide desktop window and yield supporting bays when space is constrained?
- Does rendered Markdown cover headings, prose, links, lists, quotes, code, tables, and task-list states in both themes?
- Does the light theme maintain the same semantic meaning and contrast hierarchy as dark?

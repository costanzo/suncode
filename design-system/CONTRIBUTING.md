# Design System Contribution Guide

This guide defines how to add review modules and components to SunCode's React design-system browser. It governs the review surface only; React/Vite remains specification tooling and is not a production runtime dependency.

## Source Of Truth

[`DESIGN.md`](../DESIGN.md) is the visual and interaction authority. This document defines repository structure and contribution workflow. When a required visual or interaction rule is missing, update `DESIGN.md` in the same change instead of inventing a local alternative.

## Choose The Layer

- **Core**: foundations, tokens, typography, assets, and design principles.
- **Components**: reusable cross-platform interaction primitives and their review states.
- **Platforms**: platform-specific behavior, ownership boundaries, and native adaptations.
- **Projects**: product surfaces such as ProjectHub, Desktop Settings, and Workspace panels.

Do not place a project-specific surface in Components, and do not create a Core page for a product workflow.

## Add A Module

1. Inspect the existing hash route and neighboring module page.
2. Add the page under the owning layer, following the existing `index.jsx` and `index.js` conventions.
3. Register the route and its sidebar metadata in `src/app/navigation.js`.
4. Keep parent pages as concise module indexes. Give expandable children stable routes when the child owns substantial content or interaction states.
5. Add a README when the module has non-obvious ownership, behavior, or deferred scope.
6. Add or update the relevant design-system page before changing production UI when the required state is not already specified.

## Add A Universal Component

Each universal component belongs in its own folder under `src/components/universal/` and should contain:

```text
<component>/
├── <Component>.jsx
├── <Component>Specimen.jsx
├── <component>.css       # when it has visual rules
├── index.js
└── README.md              # when behavior or ownership needs explanation
```

The component folder owns its specimen, stable export, and visual rules. Module pages compose the specimen; they must not duplicate component implementation.

## Styles

- Add semantic values to `src/styles/tokens/` before using a new color, spacing value, radius, shadow, or control dimension.
- Use token variables in component and page styles; do not introduce raw visual values for convenience.
- Colocate universal component CSS with the component.
- Keep Core page styles under `src/core/pages/styles/`, Desktop page styles under `src/projects/desktop/styles/`, and Workspace styles under `src/projects/desktop/workspace/styles/`.
- Do not add new rules to `review.css` or `browser.css` unless the rule genuinely belongs to their documented compatibility or browser-shell responsibilities.
- Preserve fixed geometry declared by the existing specimen and `DESIGN.md`; do not replace fixed dimensions with content-sized `auto` values without an explicit design decision.

## Required States

Represent the states that apply to the surface: rest, hover, pressed, focus, disabled, loading, empty, success, warning, error, and constrained-width behavior. Icon-only actions need accessible labels. Destructive or consequential actions must use the shared confirmation pattern.

## Assets And Routes

- Put source-imported logos, icons, fonts, and images in `src/assets/` and record reusable assets in `src/assets/README.md`.
- Put browser-direct files such as the favicon in `public/assets/`.
- Use stable hash routes and update navigation metadata instead of adding scroll-only destinations for substantial modules.

## Review Checklist

- The page/component follows `DESIGN.md` tokens, typography, hierarchy, and interaction language.
- The route is reachable from the correct top-level module and sidebar tree.
- Styles are owned by the narrowest appropriate file.
- Dark and light themes preserve semantic meaning and readable contrast.
- Desktop and constrained-width layouts preserve required order, fixed geometry, and overflow behavior.
- Keyboard focus and applicable empty/error/loading/disabled states are visible.
- `npm run build` passes from `design-system/`.
- `git diff --check` passes from the repository root.

# Requirement

## Background

The repository has a design document and Avalonia styles, but the complete theme contract, component states, and reusable visual assets are not available in one reviewable place.

## Goals

- Define one semantic visual language for all client modules and components.
- Keep dark and light theme values explicit and reviewable.
- Provide complete static component pages for design review.
- Create a central catalog for brand images and interface icons.

## Non-goals

- Rebuild the Avalonia client in this delivery.
- Add a web production client or a frontend build pipeline.
- Replace the runtime client's native control templates.

## Requirements

- Add `design/dark.html` and `design/light.html`.
- Add `design/index.html` as the desktop-first unified review entry point for shared foundations, shell anatomy, semantic roles, rules, assets, and theme navigation.
- Show color tokens, typography, spacing, radius, controls, fields, cards, navigation, status, loading, data, and authority surfaces.
- Add a shared token stylesheet used by both pages.
- Add `design/assets/` and document its ownership rules.
- Link the review artifacts from the durable design documentation.

## Edge cases

- The pages must open directly from the filesystem.
- The pages must remain usable at compact widths.
- Light and dark themes must preserve semantic meaning, not just invert colors.
- Icon-only actions must have accessible labels in the examples.

## Acceptance criteria

- Both HTML pages render without external dependencies.
- The unified entry page renders as a wide-window desktop review surface and links to both theme pages.
- Both pages show the same component inventory with theme-specific token values.
- Assets used by the pages are under `design/assets/`.
- The design rules explicitly require semantic tokens for future Avalonia views.

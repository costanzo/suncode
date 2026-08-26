# SunCode Design System

This directory is the review surface and resource catalog for the SunCode visual language. It is intentionally static: no Node.js, package manager, bundler, or development server is required.

## Files

- `index.html` - top-level entry point for the layered design system.
- `core/` - color, typography, spacing and approved visual assets.
- `components/universal/` - cross-platform primitive inventory and theme reviews.
- `platforms/` - mobile, desktop and TUI adaptation boundaries.
- `projects/` - business-project mappings, beginning with Avalonia desktop.

Start with the unified entry point:

```text
design-system/index.html
```

From there, open either complete theme review page:

```text
design-system/components/universal/themes/dark.html
design-system/components/universal/themes/light.html
```

## Source Of Truth

The HTML pages are the visual contract for review. `index.html` is the shared entry: it shows foundations, theme paths, desktop shell anatomy, and review rules. The universal theme pages show the complete component inventory for each theme. Token sources live under `core/tokens/`; `core/styles/review.css` imports them and carries the shared review and component primitives. Avalonia runtime resources in `apps/desktop-avalonia/App.axaml` must keep the same semantic names and values.

Feature views must consume semantic resources such as `AccentBrush`, `SurfaceRaisedBrush`, `TextSecondaryBrush`, `WarningSurfaceBrush`, and `DangerBorderBrush`. A view should not introduce a new raw color, shadow, radius, or control height without first adding it to the design tokens and both review pages.

## Asset Rules

- Put new product images, logos, illustrations, and reusable icons in `core/assets/`.
- Give each asset a descriptive, stable name and record its role in `core/assets/README.md`.
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

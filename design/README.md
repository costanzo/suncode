# SunCode Design System

This directory is the review surface for the SunCode visual language. It is intentionally static: no Node.js, package manager, bundler, or development server is required.

## Files

- `dark.html` - complete component review page for the graphite theme.
- `light.html` - complete component review page for the paper-and-slate theme.
- `index.html` - desktop-first entry point for shared foundations, shell anatomy, rules, assets, and theme links.
- `tokens.css` - shared review tokens and component primitives used by all review pages.
- `assets/` - the central design asset catalog. New images and icons belong here first.

Start with the unified entry point:

```text
design/index.html
```

From there, open either complete theme review page:

```text
design/dark.html
design/light.html
```

## Source Of Truth

The HTML pages are the visual contract for review. `index.html` is the shared desktop review entry: it shows typography, spacing, radii, control dimensions, semantic color roles, desktop shell anatomy, and review rules. `dark.html` and `light.html` show the complete component inventory for each theme. `tokens.css` defines the named color, typography, spacing, radius, control-size, focus, and surface tokens used by all three pages. Avalonia runtime resources in `apps/desktop-avalonia/App.axaml` must keep the same semantic names and values.

Feature views must consume semantic resources such as `AccentBrush`, `SurfaceRaisedBrush`, `TextSecondaryBrush`, `WarningSurfaceBrush`, and `DangerBorderBrush`. A view should not introduce a new raw color, shadow, radius, or control height without first adding it to the design tokens and both review pages.

## Asset Rules

- Put new product images, logos, illustrations, and reusable icons in `design/assets/`.
- Give each asset a descriptive, stable name and record its role in `assets/README.md`.
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

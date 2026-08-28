# Design Assets

This is the central asset catalog for the SunCode design system and review pages. It is organized by reusable ownership rather than by consuming page.

## Brand

- `logos/suncode-logo.svg` - full-size brand mark for vector-capable surfaces.
- `logos/suncode-logo-small.svg` - compact brand mark used in headers and navigation.
- `logos/suncode-logo-128.png` - raster brand mark for previews and platform surfaces.

## Icons

The `icons/` directory contains the approved interface icon set used by the Avalonia client and React review browser. Icons are monochrome source assets; the consuming surface applies the semantic foreground token.

New icons should remain simple, 24px viewBox SVGs with a consistent stroke language. Add the icon to this catalog before using it in a client view.

## Platform And Window Controls

- `platform/suncode-desktop.icns` - macOS application icon source.
- `window-controls/` - platform window-control states used by the desktop chrome.

The application currently packages its own build-local copies under `apps/desktop-avalonia/Assets/`. Those copies are packaging inputs; this directory is the review catalog and source inventory for visual assets.

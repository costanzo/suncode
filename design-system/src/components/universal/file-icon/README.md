# File Icon

`FileIcon` is the universal review component for file-type glyphs in resource rows, file labels, and technical lists. It uses the VS Code default `vs-seti` file icon glyphs for the supported catalog, while folders remain part of SunCode's monochrome interface-icon family.

The component is decorative by default. Pass `label` only when the glyph appears without adjacent text and must provide its own accessible name.

The component explicitly loads `seti.woff` so the glyph face is ready before it replaces the compact type fallback. The asset is derived from VS Code's `vscode-theme-seti` extension and is included under the Seti UI MIT license. The canonical source association list is recorded in the VS Code checkout at `extensions/theme-seti/icons/vs-seti-icon-theme.json`.

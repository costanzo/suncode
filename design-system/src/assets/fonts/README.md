# Fonts

The review pages use installed `Noto Sans` / `Noto Sans CJK SC` and `JetBrains Mono` with documented fallbacks. Do not commit system-font copies by default.

The `--font-ui` token prioritizes Noto Sans for UI text and Noto Sans CJK SC,
then Noto Sans SC, for Chinese glyphs. The Avalonia client mirrors this order
with bundled Noto Sans and Noto Sans SC faces so installed fonts are optional.
All ordinary text surfaces share this stack; code, paths, identifiers, and
pairing URLs use the `--font-mono` / `MonoFontFamily` JetBrains Mono stack.

`seti.woff` is the exception: it is a licensed, source-derived icon font used by the File Icon review component to render VS Code Seti glyphs. Its origin and MIT attribution are recorded in `SETI-NOTICE.md`.

# Desktop Markdown Rendering

**Status:** Implemented and focused-tested on Windows x64; other installed-platform conformance pending

The Avalonia client uses LiveMarkdown.Avalonia, Math, and Mermaid 2.5.1. `MarkdownText` preserves source and feeds append/clear updates to the existing renderer for assistant and secondary messages. `MarkdownRendering` registers the extended math parser and presentation nodes once, before the shared pipeline is built. No SDK, storage, or agent protocol changes are involved.

- Inline/display mathematics accepts dollar and backslash delimiters. RaTeX renders formulas with inherited inline size/foreground and an 18px scrollable display surface. Incomplete blocks remain source until closed; syntax/layout errors retain source and diagnostics. Bundled Noto Sans SC is registered as a preferred glyph fallback.
- Mermaid uses the managed Mermaider parser and native Avalonia presenter, without Node.js, browser, or subprocess rendering. Its initially fitted viewport is capped at 480px and retains upstream drag/wheel navigation. A localized keyboard-accessible disclosure keeps diagram source selectable and copyable with the normal CodeBlock template. Open fences show expanded source; unsupported/invalid completed diagrams keep their diagnostic and source access. Updates to later content preserve an already opened disclosure.
- Root `DESIGN.md` and the universal Markdown specimen own appearance and states. The React specimen uses static MathML/SVG, not production parsers. Full-response copy continues to use original Markdown; rendered math is not advertised as fully selectable TeX source.

## Native deployment

Math depends on RaTeX.Avalonia/RaTeX 1.0.0. Its package contains Windows x64, macOS x64/ARM64, and Linux x64/ARM64 native assets. **Windows ARM64 has no upstream native asset and displays formula source instead.** Initialization probes the native engine and selects source nodes for missing, incompatible, or wrong-architecture libraries. This is an explicit availability fallback, not full native math support on Windows ARM64. The diagram path remains available.

Windows x64 publication was checked for `ratex_ffi.dll`, and an isolated missing-library run verified source fallback. Actual installed macOS/Linux loading, platform signing, and Windows ARM64 fallback remain release conformance checks. Do not infer installed-platform support solely from NuGet RID files. Supported diagram syntax is limited to the native presenter's implementations; neither complete Mermaid.js parity nor a full TeX document engine is promised.

## Verification

`apps/desktop-avalonia/tests/MarkdownRenderingTests.cs` covers delimiters, matrices, native formula rendering, glyph-error fallback, five common diagram types, source clipboard copying, open/closed fences, syntax errors, replacement/clear, narrow overflow, theme/secondary sizing, selection, and reattachment. The test host's default headless fonts can trigger CJK source fallback; Skia visual checks with bundled fonts verified actual CJK formulas in both themes. Native Skia output and the review route were inspected at wide and constrained widths. Desktop integration checks are run through the existing Avalonia test project.

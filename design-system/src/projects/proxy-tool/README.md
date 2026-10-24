# Proxy Tool inspector

Review route: `#/projects/proxy-tool`. This is an interactive fixture-based reference for `tools/proxy`, not a live capture monitor. The production proxy is unchanged.

The list selects one captured call. Request and Response each default to Readable, with Raw and Headers available alongside it. SSE responses add an Events view. Readable requests expose the generation configuration, collapsed system prompt, ordered transcript, tool arguments, correlated results, and expandable tool schemas. Responses assemble text deltas and report usage only when the provider supplied it.

`data.js` owns illustrative wire payloads and review scenarios; `Inspector.jsx` owns payload rendering; `index.jsx` owns the monitor composition and controls. Page styles use the global semantic theme and spacing tokens.

The preview-state selector covers complete, in-progress, provider error, truncated, unparseable, loading, and empty captures. Search, endpoint filtering, request selection, disclosures, copying, pause/resume presentation, and clear/restore work locally. Pause does not imply that forwarding stops. Clear requires inline confirmation.

Responses and Chat Completions use SSE; the current Messages adapter buffers JSON and must not be presented as an SSE stream. Provider custom URLs can have arbitrary base paths: a future production detector must match terminal endpoint paths rather than one host or a hard-coded `/v1` prefix. Source field paths remain visible beside normalized content, and raw capture data stays available when semantic parsing cannot complete. Fixture display limits are illustrative, not a new capture-store guarantee.

At constrained widths, the capture list precedes the inspector and scrolls independently. Headers and identifiers wrap; the detail body scrolls without widening the page. Both light and dark themes inherit the same semantic states.

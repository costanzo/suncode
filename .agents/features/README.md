# Features

This directory describes stable capabilities that are actually implemented. Known verification gaps remain explicit in the owning feature note. Delivery plans and progress notes are not retained here; current behavior is supported by code, focused tests, specifications, and contracts.

Keep each feature note short and factual:

- what the product can do now
- what component owns the behavior
- what the user can rely on

Do not duplicate architecture, protocol, or migration history here.

## Current capabilities

Agent and SDK:

- [`agent-phase-1/`](agent-phase-1/README.md): embedded Rust agent loop, providers, policy, approvals, recovery, and SDK behavior.
- [`rust-sdk/`](rust-sdk/README.md): async-first Rust facade, blocking adapter, typed event streams, atomic session watch, explicit shutdown, and host capability ceilings.
- [`rust-core-phase-1/`](rust-core-phase-1/README.md): audited filesystem, search, process, artifact, checkpoint, and WebFetch operations.
- [`persistence-phase-1/`](persistence-phase-1/README.md): current SQLite ownership, normalized storage model, and native SDK boundary.
- [`providers-and-models/`](providers-and-models/README.md): provider catalog, custom models, endpoint settings, and the OpenAI Responses wire format.
- [`context-compaction/`](context-compaction/README.md): automatic model-assisted context compaction and overflow recovery.
- [`built-in-subagents/`](built-in-subagents/README.md): fixed specialist catalog, restricted delegation, linked child sessions, and desktop inspection.
- [`mcp-servers/`](mcp-servers/README.md): local and remote MCP servers with background project loading.
- [`language-servers/`](language-servers/README.md): local stdio language server definitions and runtime.
- [`browser-use/`](browser-use/README.md): Rust CDP Browser Use client and embedded CEF preview.
- [`computer-use/`](computer-use/README.md): first-party Computer Use and accessibility observation; cross-platform conformance is incomplete.
- [`network/`](network/README.md): HTTPS certificate verification, proxy settings, and the development proxy tool.
- [`diagnostic-logging/`](diagnostic-logging/README.md): persisted, rotating, redacted diagnostics.

Clients:

- [`avalonia-desktop-phase-1/`](avalonia-desktop-phase-1/README.md): implemented .NET 10 Avalonia workflows and client boundary.
- [`provider-transfer-status/`](provider-transfer-status/README.md): transient provider HTTP body byte rates in the Workspace footer.
- [`session-attention-notifications/`](session-attention-notifications/README.md): background system notifications and activation.
- [`cli-foundation/`](cli-foundation/README.md), [`cli-run/`](cli-run/README.md), [`cli-session-resume/`](cli-session-resume/README.md), [`cli-session-administration/`](cli-session-administration/README.md): native Rust CLI.
- [`remote-control/`](remote-control/README.md): outbound Desktop connection and the Remote Server relay.
- [`mobile-client/`](mobile-client/README.md): paired mobile client.

Product and review surfaces:

- [`official-website/`](official-website/README.md): public product site at `suncode.ai`.
- [`design-system/`](design-system/README.md): React/Vite design specification and review tooling (not production runtime).

Engineering tooling:

- [`harbor-benchmark-adapter/`](harbor-benchmark-adapter/README.md): isolated Harbor / Terminal-Bench adapter for the native CLI.

## Consolidated delivery coverage

Completed dated requirements have been summarized into these current feature records. The dates identify the removed delivery packages; this table is a traceability index, not a second requirements archive.

| Former delivery | Current feature record |
| --- | --- |
| Web remote-control design surface (2026-10-04) | [`design-system`](design-system/README.md) |
| Provider-neutral Computer Use (2026-10-06) | [`computer-use`](computer-use/README.md) |
| Remote transport/core ownership (2026-10-06) | [`remote-control`](remote-control/README.md) |
| Local and user Skills (2026-10-06) | [`agent-phase-1`](agent-phase-1/README.md) |
| CLI-only configuration overrides (2026-10-07) | [`cli-foundation`](cli-foundation/README.md) |
| Public product website (2026-10-07) | [`official-website`](official-website/README.md) |
| Harbor / Terminal-Bench adapter (2026-10-07) | [`harbor-benchmark-adapter`](harbor-benchmark-adapter/README.md) |
| Desktop read-only system Git (2026-10-12) | [`avalonia-desktop-phase-1`](avalonia-desktop-phase-1/README.md) |
| Project language-server catalog and jdtls, vtsls, basedpyright, rust-analyzer, Roslyn (2026-10-17) | [`language-servers`](language-servers/README.md) |
| OS-backed provider secret storage (2026-10-18) | [`providers-and-models`](providers-and-models/README.md) |
| Follow-up queue and turn steering (2026-10-21) | [`agent-phase-1`](agent-phase-1/README.md), [`avalonia-desktop-phase-1`](avalonia-desktop-phase-1/README.md) |
| Computer accessibility tree (2026-10-23) | [`computer-use`](computer-use/README.md); Windows/Linux conformance remains open |
| Native desktop menus, Dock menu, and Windows Jump List (2026-10-23) | [`avalonia-desktop-phase-1`](avalonia-desktop-phase-1/README.md); installed shell click-through remains open |

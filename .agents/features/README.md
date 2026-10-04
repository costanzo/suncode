# Features

This directory describes stable capabilities that are actually implemented. Delivery plans and progress notes are intentionally not retained here; current behavior is summarized in these feature records and verified by code, focused tests, specifications, and contracts.

Keep each feature note short and factual:

- what the product can do now
- what component owns the behavior
- what the user can rely on

Do not duplicate architecture, protocol, or migration history here.

## Current capabilities

Agent and SDK:

- [`agent-phase-1/`](agent-phase-1/README.md): embedded Rust agent loop, providers, policy, approvals, recovery, and SDK behavior.
- [`rust-sdk/`](rust-sdk/README.md): async-first Rust facade, blocking adapter, typed event streams, atomic session watch, explicit shutdown, and host capability ceilings.
- [`rust-core-phase-1/`](rust-core-phase-1/README.md): audited filesystem, search, Git, process, artifact, checkpoint, and WebFetch operations.
- [`persistence-phase-1/`](persistence-phase-1/README.md): current SQLite ownership, normalized storage model, and native SDK boundary.
- [`providers-and-models/`](providers-and-models/README.md): provider catalog, custom models, endpoint settings, and the OpenAI Responses wire format.
- [`context-compaction/`](context-compaction/README.md): automatic and manual context compaction.
- [`built-in-subagents/`](built-in-subagents/README.md): fixed specialist catalog, restricted delegation, linked child sessions, and desktop inspection.
- [`mcp-servers/`](mcp-servers/README.md): local and remote MCP servers with background project loading.
- [`language-servers/`](language-servers/README.md): local stdio language server definitions and runtime.
- [`browser-use/`](browser-use/README.md): bundled Playwright Browser Use worker and the embedded CEF preview (partial).
- [`computer-use/`](computer-use/README.md): first-party Computer Use (partial).
- [`network/`](network/README.md): HTTPS certificate verification, proxy settings, and the development proxy tool.
- [`diagnostic-logging/`](diagnostic-logging/README.md): persisted, rotating, redacted diagnostics.

Clients:

- [`avalonia-desktop-phase-1/`](avalonia-desktop-phase-1/README.md): implemented .NET 10 Avalonia workflows and client boundary.
- [`provider-transfer-status/`](provider-transfer-status/README.md): transient provider HTTP body byte rates in the Workspace footer.
- [`session-attention-notifications/`](session-attention-notifications/README.md): background system notifications and activation.
- [`cli-foundation/`](cli-foundation/README.md), [`cli-run/`](cli-run/README.md), [`cli-session-resume/`](cli-session-resume/README.md), [`cli-session-administration/`](cli-session-administration/README.md): native Rust CLI.
- [`remote-control/`](remote-control/README.md): outbound Desktop connection and the Remote Server relay.
- [`mobile-client/`](mobile-client/README.md): paired mobile client.
- [`design-system/`](design-system/README.md): React/Vite design specification and review tooling (not production runtime).

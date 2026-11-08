# Website Claims Guide

Reviewed: 2026-11-04. Scope: all 31 feature records, the remaining completed requirement package, and targeted source/contract checks. This is an editorial map to the owning feature records, not a release certification or a replacement specification. No authenticated provider sweep, installed-platform matrix, or Desktop–Relay–Mobile end-to-end run was performed in this review.

## Product description

SunCode is a general-purpose coding agent with a native desktop interface and an embedded Rust agent. It helps developers understand, change, review, and maintain local projects, with visible tool activity, scoped approvals, and conflict-aware undo for checkpointed file changes. A native CLI supports one-shot automation, and a paired mobile companion can control desktop sessions through a relay.

"Local" describes project execution and stored session state. Model requests still send selected context to configured providers; optional remote control and network tools also communicate outside the machine. Do not imply offline inference, zero data transmission, universal reversibility, or an OS sandbox.

## Capabilities suitable for product copy

These statements describe implemented behavior. Distribution availability must be established separately. Keep the qualification with the claim, including in translated copy and comparison tables.

| Topic and owning records | Supported wording | Qualification |
| --- | --- | --- |
| [Agent workflow](agent-phase-1/README.md), [context compaction](context-compaction/README.md) | Work in persistent project sessions, follow streamed activity, queue follow-ups, steer a running turn, and continue long conversations with automatic context compaction. | Queued input is transient until admitted to history. Compaction is a bounded summary, not unlimited or lossless memory. |
| [Desktop](avalonia-desktop-phase-1/README.md) | Browse projects and session history, inspect files and changes, manage approvals, and use English or Simplified Chinese in light or dark themes. | The file viewer is read-only. The changed-file panel currently merges paths across turns; do not call it an exact per-turn diff. |
| [Operations and undo](rust-core-phase-1/README.md) | Search and edit project files, run commands, retrieve web text, and restore checkpointed file changes with conflict checks. | Shell, Terminal, MCP, browser, and Computer Use effects are outside filesystem undo. Child-session checkpoints are not included in parent-turn undo. The dispatcher is not OS isolation. |
| [Specialists](built-in-subagents/README.md) | Let the primary agent delegate to six built-in specialists and inspect their linked child sessions. | Fixed roles and restricted tools; no custom agents, nested delegation, or promise of parallel autonomous teams. |
| [Providers and models](providers-and-models/README.md) | Select configured models, manage provider credentials, and change existing provider endpoints. | Catalog presence is not live integration certification. Capabilities vary by model. No client API creates/removes arbitrary catalog entries. |
| [MCP](mcp-servers/README.md) | Connect local stdio and remote Streamable HTTP MCP tool servers. | Tools only, with approval/policy limits; no resources, prompts, OAuth, or sandboxing guarantee. |
| [Language servers](language-servers/README.md) | Give the agent diagnostics, definitions, references, hover, and symbols through five supported language-server integrations. | Servers and language toolchains require local installation and project configuration. No arbitrary server types, formatting, completion, or code actions. |
| [Document reading](office-document-read/README.md) | Inspect, read, and search PDF, Word, presentation, and spreadsheet content with bounded extraction tools. | DOCX/PPTX/XLSX/ODS and targeted PDF cases have fixtures. DOC/PPT/XLS adapters still lack representative binary fixtures. No document editing, OCR, visual fidelity, or spreadsheet recalculation promise. |
| [Markdown](markdown-rendering/README.md) | Read rendered Markdown, mathematical formulas, and Mermaid diagrams in desktop conversations. | Supported syntax follows pinned libraries. Unsupported content falls back to source; Windows ARM64 math uses source fallback and installed macOS/Linux conformance remains open. |
| [Shells and Terminal](shell-settings/README.md) | Choose Agent and interactive Terminal shells independently and work in local terminal tabs. | Interactive terminals run with user authority, outside Agent approvals and undo. PowerShell setup and broader platform conformance have explicit gaps. |
| [CLI foundation](cli-foundation/README.md), [run](cli-run/README.md), [resume](cli-session-resume/README.md), [session administration](cli-session-administration/README.md) | Run a one-shot turn, continue a saved session, list/archive sessions, and consume JSONL output from a native Rust CLI. | No interactive chat or approval/question continuation. A pending decision returns status 4. CLI and Desktop cannot own the same data directory concurrently. Browser/Computer Use are disabled in CLI. |
| [SDK](rust-sdk/README.md), [persistence](persistence-phase-1/README.md) | Embed the Rust agent through its async facade, with C and C# bindings used by the desktop. Session state is stored locally in Rust-owned SQLite. | Python and TypeScript bindings are placeholders. Do not imply published packages or a live remote agent service. |
| [Network](network/README.md), [logging](diagnostic-logging/README.md), [transfer status](provider-transfer-status/README.md) | Configure trust/proxy settings and inspect redacted diagnostics and provider transfer activity. | Coverage is client-specific: CEF ignores SunCode proxy settings and updates retain strict trust. Byte rates are provider body traffic, not whole-device bandwidth. Provider API keys use the OS credential store exclusively. Other configured secrets can remain plaintext. |

## Implemented surfaces requiring prominent availability limits

| Surface | Current boundary | Evidence needed for a stronger release claim |
| --- | --- | --- |
| [Browser Use and preview](browser-use/README.md) | Optional CEF-enabled desktop builds only; default builds and baseline installers exclude CEF. Preview is partial, and browser screenshots do not reach the model as images. | Matching CEF packaging, platform smoke tests, and the feature's missing interaction/control capabilities. |
| [Computer Use](computer-use/README.md) | Partial, off by default, primary-display control with approval, takeover, and emergency-stop paths. | Durable approval batch semantics, dedicated approval UX, and installed capture/input/accessibility conformance. Wayland and multi-display control are not implemented. |
| [Remote control](remote-control/README.md), [mobile](mobile-client/README.md) | Paired companion implementation with Android-focused tests and iOS compilation; desktop and relay must remain available. | Full cross-component conformance, real-device scanning, desktop token refresh, and deployment verification. Relay state is in memory; restart requires re-pairing. No mobile image sending, background delivery, or push notifications. |
| [Notifications](session-attention-notifications/README.md) | Background desktop attention notifications and activation are implemented. | Installed click/display testing on signed macOS, Windows, GNOME, and KDE. Do not advertise mobile push. |
| [Windows installer](windows-installer/README.md) | x64 package and isolated install/uninstall lifecycle verified. | Signing, clean-machine behavior, ARM64, and trusted updater enrollment. |
| [macOS/Linux packaging](unix-desktop-packaging/README.md) | Packaging source exists, with partial user-reported publish results. | Native reruns after fixes, installed lifecycle/runtime verification, and signing/notarization. No verified all-platform download promise. |
| [Component updates](desktop-component-updates/README.md) | Client and signed catalog checking exist; checked-in catalogs reference sample archive values. | Real archives, immutable identities, platform signing, trusted installation receipts, and installed update conformance. Do not use catalog URLs as working downloads. |

The [design system](design-system/README.md), [Harbor adapter](harbor-benchmark-adapter/README.md), and developer proxy tool are engineering/review tools, not additional released clients. Benchmark integration provides no performance score or ranking. The [website](official-website/README.md) is a product information site, not the deferred Web agent client.

## Review findings and disposition

| Finding | Disposition |
| --- | --- |
| The completed Office package marked all steps done although legacy binary conformance was never demonstrated. | Consolidated scope, parsing boundaries, evidence, and remaining gaps into the Office feature; removed the eight dated delivery files. Preserved the requirement template and lifecycle policy. |
| Office and Markdown features were absent from the main index. | Both are indexed. All 31 feature directories are now reachable from the index. |
| Desktop notes called mobile and CLI session workflows deferred, described obsolete free-form LSP settings, and said update checking was disabled. | Corrected to current owning records and source. Packaging and shell summaries now retain their actual verification distinctions. |
| Browser notes simultaneously claimed CEF CDP support and listed it as unimplemented; network notes still described a retired worker proxy path. | Corrected to host-owned CEF, optional builds, and the absence of SunCode proxy propagation. Aligned the stale architecture paragraph. |
| Runtime ownership and ABI details had drifted: events, MCP, LSP, Computer Use, and ABI 25. | Updated to current crate ownership and ABI 27; removed the unnecessary ABI number from the general agent story. |
| Agent notes omitted document tools and implied all hosts ignored environment credentials. | Added document tools and the CLI-only ephemeral override distinction. |
| Provider copy treated `store: false` as a provider retention guarantee. | Replaced with the actual request behavior and provider-policy limit. Authenticated compatibility of seeded Responses routes remains unverified; exact model/vendor marketing requires fresh verification. |
| Remote notes claimed immutable system TLS defaults and referred to a Web client. | Corrected to shared configured trust and the receiving client. Remote control verification gaps remain open. |
| Desktop changed-file notes said per-turn paths were unused. | Source consumes and merges them; corrected the diagnosis while retaining the unresolved session-versus-turn presentation discrepancy. |
| Existing website hero promises every meaningful action is reversible; its capability copy groups desktop Git under Rust dispatch. | Resolved in the 2026-11-05 website update: copy limits undo to checkpointed mutations, and Git review is separate. The `/features` catalog maps all 31 records with scope and availability. This source change does not establish deployment. |

## Source checks and verification

The targeted checks used `agent/crates/documents/src/`, `agent/crates/tools/src/documents.rs`, the tools crate's `builtin/` and `definitions/` modules, `agent/crates/{browser,computer,mcp,lsp,events,remote}/src/`, provider seed SQL under `agent/crates/database/src/sqlite/data/`, `sdks/rust/src/types.rs`, `apps/cli/src/args.rs`, the desktop project/changed-path projection, and `website/app/page.tsx`. Feature records retain their own test and contract references; historical test claims were not all rerun.

`cargo test -p suncode-documents --offline --locked` passed all 13 tests on 2026-11-04, plus doc tests (zero cases). This confirms the current generated fixtures, not the missing legacy binary coverage. Documentation validation checks feature-index coverage, local Markdown links, removed-package references, and `git diff --check`. No runtime contracts or UI were changed.

Before publishing download buttons or platform/provider badges, verify the actual artifacts and integrations being offered. Until then, use implementation-scoped wording and link source/build instructions where appropriate.

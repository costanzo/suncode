# Architecture

## Current state

The immutable Rust-owned catalog contains jdtls, vtsls, basedpyright, and rust-analyzer. Shared lifecycle, document synchronization, runtime status, bounded stderr reporting, and five semantic tools are language-neutral. No Roslyn production adapter exists yet.

## Proposed design

Add a fixed `roslyn-language-server` catalog kind. Its adapter will launch the configured executable with `--stdio`, `--telemetryLevel off`, a SunCode-managed extension log directory, automatic project loading, and the selected source-generator policy. It will supply the C# language id and typed workspace settings for solution selection, restore, and diagnostic scopes.

## Boundaries and dependencies

Rust remains authoritative for executable and .NET detection, launch construction, project-pattern matching, configuration mapping, persistence, and runtime reconciliation. The design-system implementation remains review tooling only and does not become an Avalonia runtime dependency.

SunCode will not load arbitrary Roslyn extension assemblies or use the shared daemon. Each project retains its own process and workspace state, matching the existing catalog lifecycle.

## Data and control flow

Settings loads the catalog, opens the Roslyn detail by stable kind, edits a typed draft, and eventually submits one optimistic project mutation. Automatic project loading uses the server-recommended fallback limit or a custom positive limit. Automatic solution selection follows Roslyn's single-root-solution and recursive project-discovery rules; custom selection maps to `dotnet.defaultSolution`.

Compiler and analyzer scopes map independently to `openFiles`, `fullSolution`, or `none`. Automatic restore maps to `dotnet.projects.enableAutomaticRestore`. Source-generator execution maps to the supported `Balanced` or `Automatic` launch policy.

## Security and failure handling

The language server must run without a shell in the project root using the filtered environment. Repository files cannot select the executable. Telemetry is disabled by SunCode. Extension loading, debugger launch, daemon mode, and external pipe transport are unavailable. Server requests that edit files or execute arbitrary commands remain refused by the shared LSP client.

Restore, analyzers, and source generators run with the user's operating-system authority and may access package caches or the network. Their effects are not covered by filesystem undo. Startup stderr remains bounded and redacted through the shared LSP runtime.

## Compatibility and migration

The production persistence version and optional typed fields will be chosen after design approval. Existing version 1-5 records must remain readable. Supporting project-relative glob patterns may require a catalog-specific matcher without changing existing literal-marker behavior.

## Risks and rollback

The .NET tool is currently prerelease and its command-line options may evolve. SDK selection, MSBuild evaluation, NuGet restore, analyzers, and source generators vary across projects. The design keeps detection visible and expensive analysis choices explicit. Until production implementation is approved, rollback is limited to removing the design-only catalog fixture and route.

## Open questions

- Confirm whether project auto-load policy belongs in launch arguments or initialization options for the pinned server version.
- Confirm the production representation for project-relative root patterns.

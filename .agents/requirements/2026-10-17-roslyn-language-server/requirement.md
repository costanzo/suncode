# Requirement

## Background

SunCode currently provides project-scoped Java, JavaScript/TypeScript, Python, and Rust semantic analysis through fixed catalog entries. C# projects need the same five bounded read-only semantic tools through the official `roslyn-language-server` .NET tool.

## Goals

- Add Roslyn Language Server as the proposed fifth built-in language-server type.
- Design a focused project configuration surface before production implementation.
- Detect the executable and the .NET SDK/runtime selected from the project root.
- Expose stable C# project-loading and diagnostic choices without recreating Roslyn's editor configuration.

## Non-goals

- Bundling or installing .NET, Roslyn Language Server, SDK workloads, NuGet packages, or project dependencies.
- Supporting Razor as a separate model-facing language in this delivery.
- Exposing completion, rename, formatting, code actions, extensions, daemon/pipe transport, telemetry, log level, arbitrary launch arguments, or raw initialization JSON.
- Implementing Rust, SDK, persistence, or Avalonia behavior before design approval.

## Requirements

- The catalog identifies Roslyn Language Server as C# semantic analysis with independent installation detection.
- Project configuration includes enablement, executable, root patterns, project auto-load limit, automatic or custom solution selection, automatic restore, source-generator execution policy, compiler and analyzer diagnostic scopes, write-only environment changes, and timeouts.
- The detail page shows detected .NET SDK and runtime versions as read-only project facts.
- SunCode owns `--stdio`, `--telemetryLevel off`, log-directory construction, the C# language id, launch construction, settings mapping, validation, and defaults.
- Automatic loading remains enabled because SunCode does not issue editor-specific solution-open commands. Recommended mode uses Roslyn's 500-project fallback limit; custom mode accepts a positive bounded limit.
- A custom solution must be a project-relative `.sln` or `.slnx` path.
- Save and Reset remain disabled until the normalized draft differs from the loaded project configuration.
- Runtime failure retains configuration and exposes an actionable Retry.
- The UI states that restore, analyzers, and source generators run with the user's authority and are outside filesystem undo.

## Edge cases

- The global tool executable exists but its required .NET 10 runtime is unavailable.
- `global.json` selects an SDK different from the host default or selects an unavailable SDK.
- A repository contains multiple solutions or more projects than the discovery limit.
- A custom solution is renamed, removed, or outside the opened project.
- NuGet restore requires network access, credentials, or private package sources.
- Source generators or analyzers fail, hang, or execute expensive project code.
- Full-solution diagnostics are expensive in a large repository.

## Acceptance criteria

- The design-system catalog shows five supported language servers including Roslyn Language Server.
- A direct route opens the Roslyn detail page.
- The page covers default, custom project limit, custom solution, dirty, disabled, indexing, ready, failed, and retry states.
- Light, dark, and constrained-width layouts remain usable.
- Production behavior remains unchanged until the design is approved.

## Open questions

- Confirm production persistence versioning, root-pattern matching, and exact Roslyn workspace-configuration keys during implementation.

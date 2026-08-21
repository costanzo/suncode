# Requirement

## Background

The Phase 1 desktop client is implemented with .NET 10 Avalonia. Avalonia provides a cross-platform native desktop UI without changing the Rust-owned runtime architecture.

## Goals

- Provide a buildable .NET 10 Avalonia desktop application.
- Preserve the existing desktop workflows: projects, sessions, conversation streaming, approvals, checkpoint undo, Git review, settings, credentials, models, theme, and runtime diagnostics.
- Keep all durable state, provider access, project operations, policy, and recovery in Rust.
- Reuse the existing method-oriented C ABI and hand-written JSON DTO contract.
- Keep the desktop implementation limited to Avalonia and the Rust SDK facade.

## Non-goals

- Change runtime behavior, persistence, provider integrations, or SDK DTO semantics.
- Add CLI, TUI, Web, mobile, IDE, hosted, or collaborative clients.
- Introduce direct C# access to SQLite, providers, Git, or project files.
- Generate language bindings from the contract.

## Requirements

- Target `net10.0` and use Avalonia Desktop.
- Load the embedded Rust runtime through P/Invoke and validate ABI version 1.
- Keep native calls off the UI thread and marshal subscription callbacks onto Avalonia's UI dispatcher.
- Close subscriptions before releasing the shared runtime handle.
- Provide dark and light themes using the existing semantic palette.
- Keep approvals and undo limitations explicit and keyboard-accessible.
- Build the Rust dynamic library as part of the desktop build and copy it beside the managed application.
- Match the project hub, project workbench, settings, dialogs, Git drawer, responsive geometry, shortcuts, and empty/loading/error states within toolkit rendering limits.

## Edge cases

- Runtime library missing or ABI mismatch.
- Data directory already locked by another process.
- Project is not a Git repository.
- Session has no messages, no checkpoint, or no configured model.
- Subscription callback arrives during session/window shutdown.
- Native SDK returns malformed JSON or a redacted SDK error.

## Acceptance criteria

- `dotnet build apps/desktop-avalonia/SunCode.Desktop.csproj` succeeds with .NET 10.
- Rust SDK tests continue to pass with both static and dynamic library outputs.
- The Avalonia client exercises every desktop SDK method required by the Phase 1 workflows.
- Product, architecture, feature, contract, and decision documentation describe Avalonia as the only desktop client.

## Open questions

- None for this delivery.

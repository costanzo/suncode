# SunCode

SunCode is a general-purpose coding agent for developers working in software repositories. It helps inspect, change, review, and maintain a project while keeping machine access visible, scoped, approval-aware, and reversible.

Phase 1 is a local desktop application. A .NET 10 Avalonia client embeds one Rust agent SDK in-process. Rust owns the agent loop, model providers, policy, approvals, persistence, credentials, project operations, recovery, and undo. The desktop client owns presentation and transient interaction state.

## What You Can Do

- Open local projects and work in project-scoped sessions.
- Submit turns, follow streamed assistant and tool activity, queue follow-up input, cancel work, and resume earlier sessions.
- Use multiple configured model providers and models, with per-model reasoning-effort support where available.
- Inspect project files and search with bounded, ignore-aware tools.
- Make preconditioned file changes and review touched paths through turn-level checkpoints and conflict-aware undo.
- Review read-only Git status and structured diffs without requiring a Git executable.
- Run approved commands using explicit structured or platform-native shell execution.
- Fetch bounded textual web content through an approval-gated `webfetch` tool.
- Answer structured questions, track the current turn's todos, inspect provider traces, and manage read-only project dependencies.

Sensitive operations pass through Rust policy evaluation and an audited dispatcher. Approval, scope, checkpoint, cancellation, and recovery state are kept in the agent rather than reconstructed by the UI.

## Architecture

```text
.NET 10 Avalonia desktop
        |
        | P/Invoke over the hand-written C ABI
        v
Embedded Rust SDK (suncode-agent)
        |
        +-- suncode-llm       provider contracts and adapters
        +-- suncode-tool      audited filesystem, Git, process, and web operations
        +-- suncode-data      Diesel persistence and table operations
        `-- suncode-database  SQLite schema, manifests, and seed data
```

The agent uses one local SQLite database under the user data directory. The current schema is initialized transactionally and rejects incompatible databases rather than silently converting them. Streaming events are live notifications; normalized session tables are the durable source of truth.

The native SDK is embedded and method-oriented. There is no client-facing HTTP server, loopback endpoint, standalone agent service, or production TypeScript runtime. Future language SDKs are planned as native bindings over the same Rust implementation.

## Requirements

- .NET SDK 10
- Rust stable and Cargo
- A platform supported by Avalonia and the Rust toolchain

## Build And Run

From the repository root:

```sh
dotnet build apps/desktop-avalonia/SunCode.Desktop.csproj
dotnet run --project apps/desktop-avalonia/SunCode.Desktop.csproj
```

The desktop build invokes Cargo for `suncode-agent` and copies the resulting native library beside the managed executable.

To run the desktop tests:

```sh
dotnet test apps/desktop-avalonia/tests/SunCode.Desktop.Tests.csproj
```

To run the Rust workspace tests independently:

```sh
cargo test --manifest-path agent/Cargo.toml --workspace --all-targets
```

For a release publish on Apple Silicon macOS:

```sh
dotnet publish apps/desktop-avalonia/SunCode.Desktop.csproj \
  -c Release -r osx-arm64 --self-contained false
open apps/desktop-avalonia/bin/Release/net10.0/osx-arm64/publish/SunCode.app
```

## Repository Layout

| Path | Purpose |
| --- | --- |
| `apps/desktop-avalonia/` | .NET 10 Avalonia desktop client and colocated tests |
| `agent/` | Rust workspace for the embedded agent and its crates |
| `contracts/` | Hand-written SDK, persistence, and SQLite contracts |
| `design-system/` | Layered static design-system review pages and resource catalog |
| `sdks/` | Planned native TypeScript and Python binding surfaces |
| `.agents/` | Product, architecture, decisions, features, and current specifications |

## Documentation

- [Product overview](PRODUCT.md)
- [Architecture](.agents/ARCHITECTURE.md)
- [Implemented features](.agents/features/README.md)
- [SDK contract](contracts/agent-sdk/README.md)
- [Persistence contract](contracts/persistence.md)
- [SQLite schema](contracts/sqlite-schema.md)
- [Design-system review](design-system/README.md)

## Current Scope

The Avalonia desktop application and embedded Rust agent are the Phase 1 production surface. CLI, TUI, Web, mobile, IDE plugins, hosted execution, multi-user collaboration, and executable third-party extensions are deferred. The project does not claim to be an OS sandbox: the Rust boundary provides ownership, auditing, and policy enforcement inside the host process.

## License

SunCode is distributed under the license in [LICENSE](LICENSE).

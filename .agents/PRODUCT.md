# SunCode Product Overview

SunCode is a local-first coding agent. It runs on the developer's machine, works on projects already checked out there, and keeps conversation history, settings, and credentials in local storage owned by the user.

It is comparable in purpose to Claude Code, Codex, and OpenCode.

## Product thesis

Three commitments differentiate SunCode. Every architectural tradeoff should be resolved in favor of these, and a design that weakens one needs an explicit decision record.

1. **Local-first, no service dependency.** Project files, session history, and settings live on the user's machine. SunCode contacts the network only to reach the model provider the user configured. There is no SunCode account, no telemetry service, and no hosted control plane. The product remains fully functional for a user who never signs in to anything except their model provider.
2. **Reviewable authority.** Every machine-affecting action the agent takes passes through one narrow, audited path with a stated scope. Users can see what the agent is permitted to do, what it did, and undo its filesystem changes. Approval is a real decision with real granularity, not a modal that trains users to click through.
3. **One embedded runtime core, several native bindings.** One Rust SDK implementation owns session state and machine behavior inside its host process. Phase 1 embeds it in the .NET Avalonia client; future TypeScript and Python packages wrap the same implementation rather than connecting to a service or duplicating runtime logic.

## Users and primary jobs

The user is a software developer working in local repositories. The jobs SunCode must make efficient:

1. Point the agent at a project and start a session.
2. Give instructions with relevant files as context.
3. Follow messages, tool activity, and errors as they stream.
4. Approve or refuse a sensitive operation, understanding its scope.
5. Review the resulting diff, and undo it if it is wrong.
6. Interrupt a running turn and redirect it.
7. Resume earlier sessions from the local database through an SDK host.
8. Run the agent non-interactively in a script or CI job under a pre-authorized policy.

## Scope

The committed Phase 1 production surface is the Avalonia desktop application and it is the reference consumer of the Rust SDK facade. The superseded Qt client remains in the repository solely as the visual, interaction, and asset parity reference.

CLI, TUI, web, mobile, and IDE-plugin surfaces are future directions. They are deferred and must not shape current Phase 1 client implementation.

## Non-goals

- Cloud-hosted execution, multi-tenancy, and remote project provisioning. The runtime protocol should not permanently foreclose hosting, but no current design work targets it.
- Being a general-purpose editor, debugger, terminal emulator, or Git client.
- A plugin marketplace or a hosted extension registry.
- Team collaboration, session sharing, and multi-user access control.

## Constraints

- Rust is the only supported Phase 1 runtime implementation; Node.js and Bun are not production runtime dependencies.
- The production desktop application uses .NET 10 and Avalonia; Qt and Electron are prohibited production dependencies. The retained Qt source is a parity fixture, not a shipped dependency.
- The trusted OS layer is Rust.
- Protocol contracts are written as documentation and hand-implemented in each language. Contract-driven code generation is not used.

## Current status

The architecture in `ARCHITECTURE.md` is approved. The runtime core, Avalonia client, SQLite schema, and key SDK facade paths exist and have focused verification. Built-in provider support covers six providers with two static models each after focused verification. Describe a component as working only when source and focused verification exist.

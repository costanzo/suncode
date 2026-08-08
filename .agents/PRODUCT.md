# Suncode Product Overview

Suncode is a local-first coding agent. It runs on the developer's machine, works on projects already checked out there, and keeps conversation history, settings, and credentials in local storage owned by the user.

It is comparable in purpose to Claude Code, Codex, and OpenCode.

## Product thesis

Three commitments differentiate Suncode. Every architectural tradeoff should be resolved in favor of these, and a design that weakens one needs an explicit decision record.

1. **Local-first, no service dependency.** Project files, session history, and settings live on the user's machine. Suncode contacts the network only to reach the model provider the user configured. There is no Suncode account, no telemetry service, and no hosted control plane. The product remains fully functional for a user who never signs in to anything except their model provider.
2. **Reviewable authority.** Every machine-affecting action the agent takes passes through one narrow, audited path with a stated scope. Users can see what the agent is permitted to do, what it did, and undo its filesystem changes. Approval is a real decision with real granularity, not a modal that trains users to click through.
3. **One runtime core, several adapters.** A single local Rust runtime core holds session state, so clients are live views of the same work rather than separate applications with separate histories. Phase 1 ships the Qt desktop client through the SDK facade; other surfaces are deferred.

## Users and primary jobs

The user is a software developer working in local repositories. The jobs Suncode must make efficient:

1. Point the agent at a project and start a session.
2. Give instructions with relevant files as context.
3. Follow messages, tool activity, and errors as they stream.
4. Approve or refuse a sensitive operation, understanding its scope.
5. Review the resulting diff, and undo it if it is wrong.
6. Interrupt a running turn and redirect it.
7. Resume earlier sessions, and continue them from a different surface.
8. Run the agent non-interactively in a script or CI job under a pre-authorized policy.

## Scope

The committed Phase 1 surface is the Qt desktop application. It is the only client implemented in the current milestone and is the reference consumer of the Rust SDK facade.

CLI, TUI, web, mobile, and IDE-plugin surfaces are future directions. They are deferred and must not shape current Phase 1 client implementation.

## Non-goals

- Cloud-hosted execution, multi-tenancy, and remote project provisioning. The runtime protocol should not permanently foreclose hosting, but no current design work targets it.
- Being a general-purpose editor, debugger, terminal emulator, or Git client.
- A plugin marketplace or a hosted extension registry.
- Team collaboration, session sharing, and multi-user access control.

## Constraints

- Rust is the only supported Phase 1 runtime implementation; Node.js and Bun are not production runtime dependencies.
- The desktop application must use Qt; Electron is prohibited.
- The trusted OS layer is Rust.
- Protocol contracts are written as documentation and hand-implemented in each language. Contract-driven code generation is not used.

## Current status

The architecture in `ARCHITECTURE.md` is approved. The runtime core, Qt client, SQLite schema, and key SDK facade paths exist and have focused verification. Describe a component as working only when source and focused verification exist.

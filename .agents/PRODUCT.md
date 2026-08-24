# SunCode Product Overview

SunCode is a general-purpose coding agent. It helps developers understand, change, review, and maintain software projects through an agentic workflow with visible tool activity and user-controlled machine access.

It is comparable in purpose to Claude Code, Codex, and OpenCode.

## Product thesis

Three commitments differentiate SunCode. Every architectural tradeoff should be resolved in favor of these, and a design that weakens one needs an explicit decision record.

1. **Broad coding utility.** SunCode is designed for coding work across languages, frameworks, and project types rather than for one narrow development workflow. The current desktop release works with projects the user opens and keeps its application state in the embedded agent.
2. **Reviewable authority.** Every machine-affecting action the agent takes passes through one narrow, audited path with a stated scope. Users can see what the agent is permitted to do, what it did, and undo its filesystem changes. Approval is a real decision with real granularity, not a modal that trains users to click through.
3. **One embedded agent core, several native bindings.** One Rust SDK implementation owns session state and machine behavior inside its host process. Phase 1 embeds it in the .NET Avalonia client; future TypeScript and Python packages wrap the same implementation rather than connecting to a service or duplicating agent logic.

## Users and primary jobs

The user is a software developer working in code repositories. The jobs SunCode must make efficient:

1. Point the agent at a project and start a session.
2. Give instructions with relevant files as context.
3. Follow messages, tool activity, and errors as they stream.
4. Approve or refuse a sensitive operation, understanding its scope.
5. Review the resulting diff, and undo it if it is wrong.
6. Interrupt a running turn and redirect it.
7. Resume earlier sessions from the local database through an SDK host.
8. Run the agent non-interactively in a script or CI job under a pre-authorized policy.

## Scope

The committed Phase 1 production surface is the Avalonia desktop application and it is the reference consumer of the Rust SDK facade.

CLI, TUI, web, mobile, and IDE-plugin surfaces are future directions. They are deferred and must not shape current Phase 1 client implementation.

## Non-goals

- Cloud-hosted execution, multi-tenancy, and remote project provisioning. The agent protocol should not permanently foreclose hosting, but no current design work targets it.
- Being a general-purpose editor, debugger, terminal emulator, or Git client.
- A plugin marketplace or a hosted extension registry.
- Team collaboration, session sharing, and multi-user access control.

## Constraints

- Rust is the only supported Phase 1 agent implementation; Node.js and Bun are not production runtime dependencies.
- The production desktop application uses .NET 10 and Avalonia; other desktop UI toolkits and Electron are not supported production dependencies.
- The trusted OS layer is Rust.
- Protocol contracts are written as documentation and hand-implemented in each language. Contract-driven code generation is not used.

## Current status

The architecture in `ARCHITECTURE.md` is approved. The agent core, Avalonia client, SQLite schema, and key SDK facade paths exist and have focused verification. Built-in provider support covers six providers with two static models each after focused verification. Describe a component as working only when source and focused verification exist.

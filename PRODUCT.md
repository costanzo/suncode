# Product

<!-- impeccable:product-schema 1 -->

## Platform

adaptive

## Users

Software developers working in local repositories. They use Suncode to start a session against a project, provide instructions with relevant context, follow streamed messages and tool activity, decide whether sensitive operations may run, review resulting changes, undo filesystem changes when needed, and resume earlier sessions.

## Product Purpose

Suncode is a local-first coding agent that runs on the developer's machine and works with projects already checked out there. It keeps conversation history, settings, and credentials in local storage owned by the user. Success means developers can delegate useful repository work while retaining clear visibility and control over what the agent is allowed to do and what it changed.

## Positioning

Suncode has no Suncode service dependency: the product remains local-first and contacts the network only for the model provider configured by the user. A single Rust runtime core owns sessions and machine-affecting behavior across clients. Every such action passes through one narrow, audited authority path with explicit scope, approval where required, and turn-level filesystem undo.

## Operating Context

The user opens a local project in the Qt desktop application, creates or resumes a session, submits turns, and watches ordered conversation and activity events as they stream. The runtime may inspect or mutate project files, run approved processes, and produce artifacts. Sensitive actions can suspend a turn until the user allows or denies them. The user can inspect files touched by a turn and restore checkpointed filesystem changes, subject to expiry and conflict status.

## Capabilities and Constraints

- Phase 1 ships the Qt 6 desktop client as the only implemented client surface.
- Rust owns the runtime core, provider integration, agent loop, policy, approvals, persistence, credentials, client API, recovery, and operations.
- The built-in provider is DeepSeek, with the stable Suncode model identity `deepseek-v4-flash`.
- The Qt client consumes the Rust SDK facade and does not access SQLite, model providers, or project files directly.
- Machine-affecting operations use an audited internal Rust dispatcher; this is an auditability boundary, not an OS sandbox.
- Credentials are kept in the OS credential store and must not enter SQLite, protocol responses, events, audit rows, or logs.
- The product is local-first. Cloud execution, multi-tenancy, hosted identity, collaboration, CLI/TUI/Web clients, mobile clients, IDE plugins, and executable third-party extensions are deferred or out of scope.
- The desktop application must use Qt; Electron is prohibited. Node.js and Bun are not Phase 1 production runtime dependencies.
- Project, session, and turn are the product's domain terms; workspace and task are retired terms.

## Product Principles

1. Keep the product useful without a Suncode account or hosted control plane.
2. Make authority legible: users should understand scope, risk, approval, and outcome.
3. Keep one authoritative runtime core so clients remain live views of the same sessions.
4. Preserve reversibility and honest recovery status for machine-affecting work.
5. Keep protocol and ownership boundaries explicit, hand-implemented, and verifiable.

## Evidence on Hand

The repository contains an implemented Qt desktop vertical slice under `apps/desktop-qt/`, a Rust runtime under `runtime/`, hand-written protocol contracts and shared vectors under `contracts/`, and SDK documentation under `sdks/`. Existing product and architecture records are maintained under `.agents/`. No external testimonials, customer claims, benchmarks, pricing, or other proof assets are established; future work must not fabricate them.

## Accessibility & Inclusion

No product-specific accessibility standard or user need has been confirmed yet. Treat accessibility as an open product requirement and preserve Qt's native accessibility affordances in future client work.

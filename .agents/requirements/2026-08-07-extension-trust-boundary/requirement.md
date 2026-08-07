# Requirement

## Background

Phase 1 deliberately trusts the TypeScript runtime and built-in provider adapters. Third-party plugins, MCP servers, and provider adapters are deferred because an in-process worker or unsandboxed child process does not provide the promised security boundary.

## Goals

- Define a safe extension boundary before executable third-party code is enabled.
- Require independent child processes and Rust-mediated OS sandboxing.
- Preserve extension identity, least privilege, secret safety, auditability, and failure isolation.

## Non-goals

- Enabling plugins, MCP servers, or third-party providers in Phase 1.
- Treating Node worker threads as a security boundary.
- Designing cloud tenancy or hosted extension infrastructure.

## Requirements

- Extension discovery may inspect and validate manifests but must not execute project-declared code.
- Every executable extension runs in an independent child process; the host process choice is not a security substitute for OS sandboxing.
- Rust materializes the sandbox and reports observed enforcement per requested capability.
- Extension requests include an unforgeable host-issued extension identity and are re-authorized for project, session, capability, scope, lifetime, and origin.
- Extensions receive no SQLite handle, master key, Rust transport handle, unrestricted environment, arbitrary client socket, or direct project authority.
- Large values use opaque artifact references; secret values use scoped handles over a separate resolution path and are never placed in extension protocol payloads.
- Crash, timeout, protocol violation, capability escalation, or sandbox setup failure disables the extension and fails its outstanding calls closed.
- Built-in provider adapters remain in the trusted runtime; third-party provider adapters follow this boundary.

## Edge cases

- A child process starts but sandbox setup is weaker than requested.
- An extension requests a capability not present in its manifest or grant.
- The host restarts after a non-idempotent extension call.
- An extension attempts to impersonate another extension or invoke a core operation directly.
- An extension emits oversized, malformed, or secret-bearing output.

## Acceptance criteria

1. The runtime and core contracts define extension identity, handshake, capability declaration, cancellation, limits, artifacts, errors, and shutdown.
2. Platform tests prove which sandbox guarantees are enforced on Windows, macOS, and Linux; unsupported guarantees fail closed.
3. No worker-thread implementation is accepted as the isolation mechanism.
4. Extension-originated requests are re-authorized and audited at both boundaries.
5. Secret and artifact test vectors prove values do not enter protocol logs or payloads.
6. Crash/restart tests show outstanding non-idempotent calls are not replayed automatically.
7. This requirement is approved before any executable third-party extension is added.

## Open questions

- Which extension classes are needed first: plugins, MCP, or third-party providers?
- Which sandbox capabilities must be fail-closed on each supported OS?
- Which IPC framing and artifact transport are shared with the core protocol?

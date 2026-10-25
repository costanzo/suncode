# Technical Specifications

This directory records current implementation facts and contracts.

Keep each spec short and current. Prefer one file per stable boundary:

- runtime behavior
- storage schema
- client protocol
- security rules
- operation semantics

The approved target architecture and ownership boundaries remain in [`../ARCHITECTURE.md`](../ARCHITECTURE.md). Product behavior belongs in `../features/`; this directory records technical facts that need more precision than a feature summary.

Current focused specifications include [`agent-phase-1.md`](agent-phase-1.md) and [`rust-core-phase-1.md`](rust-core-phase-1.md) for agent behavior and boundaries, [`built-in-subagents.md`](built-in-subagents.md) for the immutable specialist catalog and child-session lifecycle, and [`desktop-notifications.md`](desktop-notifications.md) for the attention-notification boundary and activation IPC. Public client and storage contracts remain in `contracts/`.

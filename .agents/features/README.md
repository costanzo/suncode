# Features

This directory describes stable capabilities that are actually implemented. Delivery plans and progress notes are intentionally not retained here; current behavior is summarized in these feature records and verified by code, focused tests, specifications, and contracts.

Keep each feature note short and factual:

- what the product can do now
- what component owns the behavior
- what the user can rely on

Do not duplicate architecture, protocol, or migration history here.

## Current capabilities

- [`agent-phase-1/`](agent-phase-1/README.md): embedded Rust agent loop, providers, policy, approvals, recovery, and SDK behavior.
- [`rust-core-phase-1/`](rust-core-phase-1/README.md): audited filesystem, search, Git, process, artifact, checkpoint, and WebFetch operations.
- [`persistence-phase-1/`](persistence-phase-1/README.md): current SQLite ownership, normalized storage model, and native SDK boundary.
- [`avalonia-desktop-phase-1/`](avalonia-desktop-phase-1/README.md): implemented .NET 10 Avalonia workflows and client boundary.

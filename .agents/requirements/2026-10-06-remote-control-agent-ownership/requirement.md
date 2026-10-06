# Requirement

## Background

Remote control currently lives in the Rust SDK facade even though it is an agent capability. This couples transport, request dispatch, event forwarding, and lifecycle to one client binding.

## Goals

- Move remote-control transport into an independent Rust crate under `agent/crates`.
- Make agent core own remote command authorization, projections, event subscriptions, and lifecycle.
- Keep the existing Desktop, Remote Server, Mobile, C ABI, and managed SDK contracts compatible.

## Non-goals

- Changing the Remote Server or Mobile wire protocol.
- Adding hosted execution, new remote permissions, or new remote operations.

## Requirements

- The remote crate must not depend on `sdks/rust`.
- Remote requests must continue through the agent's existing policy, approval, session, and persistence paths.
- Non-desktop hosts must be able to disable remote control through host capabilities.
- Remote credentials and configuration remain in the existing SQLite configuration table.

## Acceptance criteria

- `sdks/rust` exposes the existing remote methods without owning the remote worker implementation.
- Agent startup, shutdown, configuration replacement, reconnect, and event upload retain current behavior.
- Focused Rust tests and workspace checks pass.

# Progress

- Status: Complete
- Last updated: 2026-09-19

## Completed

- Identified 81 public facade methods and the subset that currently calls `block_on`.
- Selected an async-facade plus `Deref`-based blocking-wrapper architecture.
- Added runtime-free `AsyncAgentSdk` startup and ownership.
- Converted Browser, MCP, LSP, turn, approval, question, and network-reconciliation operations to native async methods.
- Added `blocking::AgentSdk`, retained root `AgentSdk` compatibility, and reused synchronous methods through `Deref`.
- Preserved C ABI 10 and C#/Avalonia behavior through the blocking adapter.
- Added a current-thread Tokio test proving async startup works inside an existing runtime.
- Updated the SDK contract, architecture, feature/specification records, package documentation, and decision index.

## In progress

- None.

## Blocked

- None.

## Log

### 2026-09-19

- Requirement initialized.
- Implementation and verification completed.

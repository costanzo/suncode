# Changes

## Source

- Replaced the runtime-owning semantic facade with runtime-free `AsyncAgentSdk`.
- Converted runtime-dependent Browser, MCP, LSP, turn, approval, question, and settings methods to native async functions.
- Added `facade/blocking.rs` with runtime creation, awaited-method adapters, and `Deref` reuse for synchronous methods.
- Kept the root `AgentSdk` name as blocking compatibility while exporting `AsyncAgentSdk` and `blocking` explicitly.

## Contracts and generated artifacts

- Rust API addition/refactor; C ABI remains version 10.
- Updated the hand-written SDK contract and Rust/C package documentation.

## Configuration and persistence

- No changes.

## Tests

- Added async startup coverage on an existing current-thread Tokio runtime.
- Existing Rust SDK tests continue exercising the blocking compatibility surface.
- C binding and desktop suites verify native compatibility.

## Documentation

- Added this delivery package.
- Updated architecture, durable feature/specification records, and the accepted decision index.

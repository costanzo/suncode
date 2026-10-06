# Architecture

## Current state

`sdks/rust/src/facade/remote.rs` owns pairing, encryption, SSE, request dispatch, event upload, and the worker lifecycle. It calls the SDK facade for every agent operation.

## Proposed design

`suncode-remote` owns protocol transport, pairing, encryption, reconnect, and event forwarding. It calls a narrow `RemoteHost` trait. `suncode-agent` owns remote command dispatch and projections through its `Agent` methods; the SDK supplies only the host adapter for network settings and event-subscription bridging while the existing SDK lifecycle attaches and stops the transport controller.

The SDK facade keeps only typed client methods and delegates to `Agent`.

## Boundaries and dependencies

`suncode-remote` depends on shared Rust contracts and data/configuration primitives, never on the SDK. Agent core depends on `suncode-remote` for the remote host contract and dispatch implementation; the SDK depends on both and adapts the desktop binding.

## Compatibility and migration

The existing HTTP/SSE protocol, configuration keys, C ABI, and C# methods remain unchanged.

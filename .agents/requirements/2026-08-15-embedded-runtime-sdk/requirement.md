# Requirement

## Background

The Qt desktop client currently embeds the Rust runtime static library, but its C++ adapter still sends method, REST-like path, and JSON body strings through a generic `request_json` entry point. The Rust facade reconstructs an authenticated Axum request and dispatches it through the same router used by a standalone loopback HTTP/SSE server. This preserves transport compatibility at the cost of making an embedded SDK behave like a server.

SunCode now commits to an embedded SDK model. The Rust runtime is a library loaded into its host process and cannot be launched as a standalone client-facing server. Qt is the first host. Future TypeScript and Python packages will be native bindings over the same Rust SDK, not HTTP clients and not alternate runtime implementations.

## Goals

- Replace REST-like SDK dispatch with named, typed Rust SDK methods.
- Expose method-oriented C ABI entry points for Qt and future native bindings.
- Remove all client-facing HTTP/SSE server behavior, loopback binding, authentication token, endpoint discovery, and standalone runtime binary support.
- Preserve runtime ownership of SQLite, credentials, providers, policy, operations, recovery, events, and identifiers.
- Keep provider outbound HTTPS behavior unchanged.
- Preserve current Qt product behavior while changing its transport boundary.
- Define a native binding foundation suitable for TypeScript N-API and Python PyO3 packages.

## Non-goals

- Sharing one live runtime instance across independent OS processes.
- Introducing replacement IPC such as Unix sockets, named pipes, WebSocket, or JSON-RPC.
- Implementing the full TypeScript or Python packages in this delivery.
- Removing JSON from provider wire protocols, SQLite JSON columns, flexible SDK event payloads, or internal operation payloads.
- Changing provider behavior, policy decisions, database schema, or QML presentation.

## Requirements

- The Rust runtime must build as a library without a standalone binary entry point.
- Opening an SDK instance must acquire the runtime lock, open and migrate SQLite, initialize providers and operations, and perform recovery in-process.
- The SDK must not bind a listening socket, publish an endpoint, accept inbound HTTP, construct synthetic HTTP requests, or depend on HTTP status codes.
- SDK operations must be named methods with explicit parameters and typed Rust results or errors.
- The C ABI must use opaque handles, explicit ownership/free functions, and named operation functions. Rust-owned structs and strings must not cross the ABI by layout.
- The C ABI must expose an ABI version and retain add-only compatibility within a major ABI version.
- Qt must call named SDK methods and must not construct REST paths.
- Long-running SDK calls must remain off the Qt UI thread.
- Session events must support retained replay followed by live delivery without a replay-to-live loss window.
- SDK errors must use domain error codes rather than HTTP status codes.
- Runtime credentials and provider credentials must never be returned in SDK DTOs, events, or errors.
- Future TypeScript and Python bindings must wrap the Rust SDK in-process and must not open SQLite or implement runtime behavior independently.

## Edge cases

- A second host process attempting to open the same runtime data directory fails with a typed `runtime_already_active` error.
- Multiple client wrappers inside one process may share one runtime handle, but each owns its subscription and presentation state.
- Closing a subscription is idempotent and waits for callback delivery to stop before releasing user data.
- Closing the runtime waits for or cancels SDK-owned asynchronous work according to the existing cancellation contract.
- Callback consumers may be Qt, Node.js, or Python and therefore callbacks must not assume a language runtime thread.
- Event receiver lag must trigger replay recovery or a typed resync requirement; it must not be silently ignored.

## Acceptance criteria

- No production client-facing Axum router, listener, HTTP authentication, SSE endpoint, discovery endpoint record, or runtime server binary remains.
- `suncode_agent_sdk_request_json` is removed from Rust and Qt.
- Qt project/session/settings/credential/turn/approval/checkpoint flows use named SDK methods.
- Existing session snapshot, event ordering, approval, cancellation, checkpoint, credential, and model behavior passes focused verification.
- The Rust dependency tree no longer includes dependencies used only by the removed inbound server.
- The SDK contract documents methods, inputs, outputs, errors, ownership, threading, and event ordering without HTTP terminology.
- Architecture, feature, specification, decision, and SDK packaging documents describe the embedded model consistently.

## Open questions

- Whether future language bindings should expose synchronous convenience methods in addition to async-first APIs.
- Whether stable leaf DTOs should eventually use dedicated C structs instead of method-specific JSON payloads.

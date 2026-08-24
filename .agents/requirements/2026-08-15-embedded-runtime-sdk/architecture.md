# Architecture

## Current state

Qt links `suncode-agent` as a static library and obtains one shared opaque runtime handle. Request/response calls still cross a generic C ABI as method, REST-like path, and JSON body strings. Rust creates a synthetic authenticated Axum request and dispatches it through an in-process router. A separate binary can bind the same router to loopback HTTP and SSE.

## Proposed design

The runtime becomes an embedded library with three conceptual layers:

```text
Qt C++ adapter       future N-API binding       future PyO3 binding
       |                       |                         |
       `---------------- named native bindings ---------'
                               |
                         Rust SDK facade
                               |
                      typed runtime services
                               |
           agent, policy, persistence, providers, operations
```

The initial implementation may keep the facade and service in the existing `suncode-agent` crate while establishing typed boundaries. Separate binding crates are introduced only when they contain buildable packaging code.

## Boundaries and dependencies

- Runtime services own business validation and return typed domain results and `SdkError` values.
- The Rust SDK owns lifecycle, Tokio execution, method dispatch, event subscriptions, and language-neutral DTO serialization.
- The C ABI owns only pointer validation, UTF-8 conversion, result allocation, and callback lifetime.
- Qt owns QML-facing presentation state and Qt-thread delivery.
- Future N-API and PyO3 packages depend on the Rust SDK facade and contain no runtime logic.
- Provider adapters may use outbound HTTPS. No module accepts inbound network requests.
- Operations remain behind policy and the audited dispatcher; embedding does not grant clients direct operation access.

## Data and control flow

1. A host opens the SDK with default or explicit configuration.
2. Rust acquires the data-directory lock, opens SQLite, initializes runtime services, and performs recovery.
3. A host calls a named method such as `create_session(project_id, title, model)`.
4. The binding converts host strings and values into a typed Rust input.
5. The runtime service validates scope and executes against Rust-owned state.
6. The binding returns a method-specific DTO or a typed SDK error.
7. A session subscription registers for live events before completing replay from its sequence cursor, then delivers ordered events through the host callback.

Complex evolving DTOs and event payloads may remain UTF-8 JSON at the C boundary. JSON is a payload representation, not a routing or transport protocol. Rust and high-level language wrappers expose method-specific types.

## Security and failure handling

The SDK caller is in the same process and is therefore inside the runtime trust boundary. HTTP authentication is removed, but project/session ownership, policy evaluation, approval, path validation, credential redaction, and operation auditing remain mandatory.

Only one process may own a runtime data directory. The runtime lock remains but endpoint/token discovery is removed. A second process receives `runtime_already_active`. There is no supported cross-process attach behavior.

The SDK defines stable error codes independent of HTTP. Errors contain a bounded message and redacted structured details. Panics must not unwind across the C ABI.

## Compatibility and migration

Migration is incremental but the completed tree has no generic request router:

1. Extract typed service methods from Axum handlers.
2. Add named Rust SDK and C ABI methods with focused parity tests.
3. Migrate Qt request sites to named functions.
4. Remove `request_json`, Axum routing, SSE, listener startup, discovery publication, server configuration, the runtime binary, and HTTP-only dependencies.
5. Replace the client-runtime HTTP contract with a runtime SDK contract and update shared vectors.

The SQLite schema, stable model IDs, event names, content sequences, idempotency rules, and Qt presentation contract remain compatible.

## Risks and rollback

- Moving handler logic can change validation or response shapes. Focused method parity tests cover every Qt-used operation before the router is deleted.
- Native callback misuse can cause use-after-free. Subscription close must cancel and join before releasing host data, and callback scheduling is tested.
- Replay/live handoff can lose events. Registration and replay ordering are redesigned and tested under concurrent emission.
- Removing the server eliminates cross-process compatibility. This is an intentional product constraint, not a temporary limitation.
- Native TypeScript/Python distribution is platform-specific. ABI versioning and opaque handles prevent language packages from depending on Rust layouts.

## Open questions

- The first delivery keeps method payloads as JSON at the C ABI where DTOs are complex; later deliveries may promote stable DTOs to dedicated C structs.

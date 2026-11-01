# Contracts

These are hand-written contracts for the Rust agent, native clients, and their supporting protocols. Each document states its implementation status; defined future protocols must not be described as implemented. Contracts are normative documents, not code-generation inputs.

## Active contracts

- [`agent-sdk/README.md`](agent-sdk/README.md): embedded SDK lifecycle, methods, DTOs, errors, events, authority, and binding rules.
- [`remote-control/README.md`](remote-control/README.md): Mobile HTTP and Session SSE contracts, plus the private Desktop device boundary.
- [`persistence.md`](persistence.md): ownership, retention, recovery, configuration, and secret handling.
- [`sqlite-schema.md`](sqlite-schema.md): the current 15-table SQLite schema and projection rules.

## Distribution protocols

- [`updates/README.md`](updates/README.md) and [`updates/updates.openapi.yaml`](updates/updates.openapi.yaml): public, signed Desktop/Agent SDK update catalog (OpenAPI 3.2.1), implemented C# client planning/caching/integrity/activation boundaries, and remaining publisher/installer release verification. Production delivery is unconfigured.

Contract behavior is verified by focused Rust and Avalonia tests in the owning implementation packages. There is no generated schema or fixture directory; changes are made directly to the hand-written contract and its tests.

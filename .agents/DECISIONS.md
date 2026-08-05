# Decision Index

## ADR-20260804-foundational-architecture

- Date: 2026-08-04
- Status: Accepted
- Context: Suncode needs a secure, cross-platform foundation for a local coding-agent product.
- Decision: Use a contract-first polyglot monorepo; a TypeScript runtime supervises one Rust child; JSON-RPC 2.0 uses newline-delimited stdio; Rust owns SQLite, secret handling, sessions, operations, and permission enforcement; clients communicate only with the local runtime.
- Consequences: Contracts are canonical, generated artifacts must be deterministic, Rust remains the trusted machine boundary, and proposed product behavior must not bypass the runtime/core layering.
- Details: `ARCHITECTURE.md`

## ADR-20260805-agent-knowledge-layout

- Date: 2026-08-05
- Status: Accepted
- Context: Early project knowledge was stored under tool-specific paths alongside transient brainstorming state.
- Decision: Store durable contributor and agent context in `.agents/`, organized into features, dated requirements, technical specs, and this decision index. Keep local tool state ignored and outside `.agents`.
- Consequences: Legacy tool-specific paths and directives are obsolete and must not be recreated.
- Related requirement: `requirements/2026-08-05-agent-knowledge-layout/`
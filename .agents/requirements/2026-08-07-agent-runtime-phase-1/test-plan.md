# Test Plan

## Scope

The current delivery validates the runtime design and requirements only. Later implementation must test deterministic orchestration, provider compatibility, security boundaries, persistence/replay, extensions, and client integration.

## Unit tests for later implementation

- Turn-level state transitions, iteration limits, cancellation, retries, and budget exhaustion.
- Tool-call child machines: several calls from one assistant message, mixed outcomes across siblings (one approved, one denied, one timed out), and turn aggregation only when all children are terminal.
- Approval ordering: no execution path exists that precedes authorization.
- Stall detection for repeated near-identical tool calls within budget.
- Frozen snapshot enforcement: a tool call against a replaced or removed definition fails as stale.
- Scheduler serialization, queue policy, duplicate idempotency keys, and independent-session concurrency.
- Input admission and promotion: `queue`, `steer` only at safe boundaries, `cancel-and-replace`, drain after restart, wake coalescing.
- Policy profile resolution, and typed denial rather than a blocking prompt in non-interactive mode.
- Provider request/response normalization, capability filtering, usage, rate-limit, and error classification.
- Context precedence, provenance, token budgeting, truncation, compaction, and secret redaction.
- Tool schema validation, risk classification, output bounds, timeout, cancellation, and unknown completion.
- Policy evaluation for grants, scope, expiration, revocation, and approval requirements.
- Manifest validation and deterministic discovery for skills, plugins, and MCP servers.
- Stream separation: audit appends are never rewritten by compaction, projections rebuild from session content alone, sync state is discardable without loss.
- Content append and projection updates are transactional; replay, snapshot fallback, and projection rebuild are correct.
- Configuration layering precedence, and that a project file cannot activate a security-relevant setting on its own.
- Checkpoint capture, restore, hash-mismatch conflict reporting, and retention expiry.

## Integration and conformance tests for later implementation

- Shared test vectors: both the runtime and core implementations agree on accept/reject and extracted fields for every vector. This replaces generated-type drift checks and is mandatory for every contract change.
- Provider fixture matrix for streaming text, multimodal parts, tool calls, refusal, malformed output, context limits, rate limits, and disconnects.
- Rust-backed read, search, process, write, artifact, and approval-required operation fixtures.
- MCP fixture servers for tools, resources, prompts, schema changes, auth failure, timeouts, and server restart.
- Plugin worker fixtures for valid contribution, undeclared capability, crash, hang, incompatible version, and quarantine.
- Skill fixtures for precedence, conflicting instructions, invalid manifest, incompatible version, and bounded resources.
- Multi-client event ordering, approval resolution, reconnect/replay, and stale mutation rejection.

## Security tests for later implementation

- Provider credentials never appear in logs, events, diagnostics, or client payloads.
- Model, skill, plugin, MCP, repository, and tool text cannot broaden policy or bypass approval.
- Plugins and MCP servers cannot access SQLite, Rust transport, master keys, undeclared environment variables, or arbitrary sockets.
- The core rejects out-of-project paths and mismatched capability assertions even when runtime preflight incorrectly allows them.
- Cross-project and cross-session request/event isolation.
- A committed project file cannot widen authority; opening a hostile repository does not change what the agent may do.
- Secret values never appear in protocol message bodies, including in diagnostic request transcripts.
- Policy profiles cannot authorize a capability the user has not declared, and no mode disables enforcement or audit.
- Non-idempotent provider/tool calls are not blindly retried after uncertain completion.
- Redaction of prompts, file contents, command output, secrets, and telemetry attributes.

## Fault-injection tests for later implementation

- Runtime restart during every turn state and every tool-call child state.
- Core exit between a completed write and its recorded result, then reconciliation by hash across all three outcomes: completed, not started, and changed by something else.
- Core child exit, protocol corruption, heartbeat timeout on the control channel under bulk load, and bounded restart backoff.
- Cancellation delivered on the control channel while a large result is streaming on stdio.
- Provider outage, rate limit, credential revocation, partial stream, and malformed event.
- SQLite busy, migration interruption, projection corruption, and audit-log corruption.
- Artifact inventory mismatch on startup: orphaned bytes and dangling references.
- Client version skew against a resident runtime, including drain refusal past the grace period.
- Extension worker crash/hang and MCP server network loss.
- Client disconnect/reconnect with retained and unavailable event sequences.

## Performance tests for later implementation

- Time to first token and end-to-end turn latency by provider.
- Context assembly and compaction latency at configured history limits.
- Concurrent independent sessions under provider and Rust resource limits.
- Event throughput, SQLite append latency, replay latency, and projection rebuild time.
- Plugin/MCP call overhead and bounded output memory.

## Behavioral evaluation for later implementation

The tests above verify that the harness is correct. They do not verify that the agent is useful — that depends on system instructions, tool descriptions, and edit reliability, none of which any check above measures. Those are also the things most likely to regress silently, since a prompt change breaks nothing that compiles.

So a fixed task suite is part of the deliverable:

- A set of repository tasks with deterministic pass criteria, run against a pinned model.
- Tracked per run: pass rate, token cost, turn count, and tool-call failure rate.
- Treated as a regression gate for changes to instructions, tool descriptions, tool schemas, and compaction.
- Stood up as soon as one tool call works end to end, not after the tool catalog is complete.

Provider non-determinism means results are a distribution, not a single value. The suite needs enough tasks that a real regression is distinguishable from run-to-run variance, and the pass-rate threshold that gates a release is an open decision.

## Regression checks

- Run `git diff --check` for this documentation delivery.
- Run repository documentation/link validation when available.
- For implementation, run the documented full verification command on Windows, macOS, and Linux.

## Commands and results

- Pending until the documentation changes are complete.

## Residual risks

- No executable runtime exists to validate the proposed state machine or extension boundaries.
- Provider semantics, MCP security, plugin isolation, and secret storage need separate implementation designs.
- Exact budgets and performance targets are intentionally open.


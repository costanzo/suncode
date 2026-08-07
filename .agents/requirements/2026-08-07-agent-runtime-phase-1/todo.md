# Todo

## Product and architecture decisions

- [x] Reconcile the stale `ADR-20260804-foundational-architecture` ownership summary with `../../ARCHITECTURE.md`. Resolved by `ADR-20260807-runtime-owns-durable-state`.
- [x] Adopt the two-level turn and tool-call state machine and correct approval ordering.
- [x] Separate durable state into audit, session content, and client sync streams.
- [x] Consolidate `opencode-amendments.md` into the normative documents.
- [ ] Approve the Phase 1 runtime scope and the sequential tool-call scheduling constraint.
- [ ] Select the first provider and its authentication method: API key or OAuth subscription login. Blocks credential storage, first-run experience, and cost presentation.
- [ ] Decide canonical content/message schema and multimodal scope.
- [ ] Define default token, time, cost, iteration, tool, and output budgets.
- [ ] Define the initial built-in tool catalog and risk classes.

## Non-interactive execution

- [ ] Define the policy profile format and how a profile is declared and activated.
- [ ] Define non-interactive output format and exit-code taxonomy.
- [ ] Decide what the default profile authorizes, if anything.

## Reversibility

- [ ] Define checkpoint capture granularity, retention, and expiry visibility.
- [ ] Define restore conflict presentation when a file changed outside the agent.

## Context and sessions

- [ ] Specify instruction precedence and trust classes.
- [ ] Define the committed project configuration file: format, discovery, and how its declarations are activated without granting authority.
- [ ] Define compaction triggers, summary structure, per-stream retention, export, and deletion.
- [ ] Define draft ownership and background-task behavior.
- [ ] Define unknown-completion reconciliation by hash comparison.
- [ ] Decide whether the audit log needs tamper-evidence or whether file permissions suffice.

## Extensions

- [ ] Define skill manifest and discovery precedence.
- [ ] Choose plugin worker isolation and trust/provenance model.
- [ ] Define plugin lifecycle, pinning, upgrade, rollback, and quarantine.
- [ ] Choose initial MCP transports, authentication, network policy, and server configuration.
- [ ] Define MCP prompt/resource trust treatment and namespace rules.

## Security and operations

- [ ] Define provider secret classes, storage, rotation, and invalidation.
- [ ] Define approval defaults, persistent grants, revocation, and audit retention.
- [ ] Define safe Markdown, attachment, clipboard, and external-resource policies.
- [ ] Define rate limits, concurrency limits, circuit breakers, and shutdown grace period.
- [ ] Define runtime telemetry schema, sampling, redaction, and support diagnostics.

## Contracts and implementation

- [ ] Write the client-runtime and runtime-core contract documents. No code generation; each language implements its own types.
- [ ] Build the shared test-vector suite that both implementations must agree on. This is the only drift protection.
- [ ] Implement the runtime kernel only after contracts and persistence design are approved.
- [ ] Stand up the behavioral evaluation suite as soon as one tool call works end to end.
- [ ] Integrate the runtime with the CLI/TUI surface first, then Qt desktop.


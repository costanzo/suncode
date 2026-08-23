# Architecture

## Current state

The runtime advertised `shell` with a required `script` and second-based timeout. The execution boundary already translated shell commands into platform-specific program arguments.

## Proposed design

Advertise `bash` with OpenCode's `command`, millisecond `timeout`, and `workdir` schema. Keep translation and audited execution unchanged after converting the model request to `program`, `args`, `cwd`, and `timeout_ms`.

## Boundaries and dependencies

- `suncode-llm` receives the tool definition from core and serializes it unchanged.
- Core owns the model-facing schema and compatibility translation.
- Operations continue to receive only normalized internal process parameters.
- Avalonia displays both new bash calls and historical shell aliases as shell commands.

## Data and control flow

`bash(command, timeout, workdir)` -> core validation -> platform shell translation -> audited `process/run`

Recoverable validation failures are serialized as a `tool` message correlated by `tool_call_id`, then the agent continues its model loop. Failed results are retained in the session context so later turns do not lose the assistant/tool exchange.

Historical `shell/script` and `shell`-field calls enter the same path through compatibility lookup.

## Security and failure handling

Timeout bounds and empty-command checks remain fail-closed before process creation. Recoverable argument errors are fed back to the model; approval, project scope, filtered environment, cancellation, output bounds, and audit behavior remain enforced. Repeated equivalent calls and non-recoverable failures still stop the turn.

## Compatibility and migration

Only the model-facing advertised schema changes. Persisted tool uses retain their original request JSON. Compatibility aliases are not included in the new tool definition.

## Risks and rollback

The main risk is clients or tests assuming `shell` is an advertised tool. The old aliases remain accepted, and reverting the definition plus translation mode restores the prior contract.

## Open questions

- Whether future protocol documentation should expose a versioned tool schema vector.

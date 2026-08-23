# Architecture

## Current state

The core translates model tool calls into process/run requests and executes them in an operations spawn_blocking task. The operations runner captures bounded output in reader threads and calls Child::kill() on timeout.

## Proposed design

The core passes a CancellationToken into each authorized operation call. The operations boundary exposes an optional atomic cancellation flag so the process runner can poll it without depending on Tokio. The core watches the token while awaiting the blocking operation and sets the flag when cancellation is requested.

Each output stream has its own reader and capture state. Capture state continuously drains the pipe, keeps a 64 KiB tail preview, and switches to a temporary file after the preview limit is exceeded. Artifact creation streams the temporary files into a content-addressed artifact while hashing, avoiding an unbounded in-memory buffer.

Unix commands run in a dedicated process group and are terminated with a negative-PID signal. Windows commands are terminated with taskkill /T, with the existing hidden-console flag preserved.

## Boundaries and dependencies

- suncode-runtime owns cancellation-token observation and async scheduling.
- suncode-tool owns process creation, output capture, process-tree termination, and artifacts.
- suncode-tool adds the platform libc dependency only for Unix process-group signals.
- Avalonia and provider code remain unchanged.

## Data and control flow

turn cancellation -> core select -> atomic flag -> process poll -> process-group/tree termination -> output EOF -> result/artifact.

## Security and failure handling

Working-directory validation, filtered environment handling, policy authorization, and Windows hidden-console behavior are retained. Child termination is best-effort but the operation waits for the direct child and output pipes before returning. Temporary capture files are removed when the result is dropped.

## Compatibility and migration

The advertised tools keep process and shell; persisted bash aliases continue through the same translation. Internal timeout_ms remains an operations detail and is no longer advertised in model schemas.

## Risks and rollback

The main residual risk is platform-specific process termination behavior, especially Windows taskkill availability. Reverting the operations change restores the prior bounded capture behavior but also restores its large-output and cancellation defects.

## Open questions

- Whether a future API should expose a structured streaming tool-output event rather than only the final result.

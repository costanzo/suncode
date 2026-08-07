# Test Plan

## Scope

The current delivery validates requirements and architecture documentation only. Later implementation must test core operation correctness, security enforcement, the runtime-core contract, recovery, and cross-platform behavior.

Coverage below is scoped to the Phase 1 catalog. Deferred capabilities carry their tests in their own packages.

## Unit tests for later implementation

- Path canonicalization, project-boundary checks, symlink/junction handling, case sensitivity, and reserved path handling. This is the highest-value area, since it is both the security foundation and where platforms diverge most.
- Capability assertion validation, grant scope matching, denial reasons, and approval-required responses.
- Bounded file reads, binary detection, encoding handling, metadata, hashes, and line ranges.
- Write, edit, patch, atomicity, precondition failures, idempotency, and conflict detection.
- Glob and search limits for directories, files, bytes, matches, previews, hidden files, binary files, and ignore rules.
- Artifact fallback when a result exceeds inline limits, and pagination cursor correctness.
- Process spawning, environment filtering, working directory validation, timeout, cancellation, output bounding, and process-tree cleanup.
- Sandbox profile selection, filesystem views, temp directories, network egress policy, credential exposure, and cleanup.
- Artifact creation, metadata, read authorization, retention, sweep execution, and refusal to delete a pinned artifact.
- Checkpoint capture, extension as new paths are touched, hash verification, restore, and conflict reporting on external modification.
- Journal write-before-mutate ordering, duplicate key resolution, and terminal-outcome recording.

## Integration and conformance tests for later implementation

- Shared test vectors accepted and rejected identically by the core and runtime implementations, with matching extracted fields. This is the only drift protection, so uncovered fields are treated as gaps.
- Runtime tool adapters cannot execute privileged project operations without the core; dependency and import checks enforce it.
- Sandbox profile outcomes are reported consistently across Windows, macOS, and Linux, including where a platform cannot enforce a requested class.
- Runtime approval decisions are rejected by the core when the capability assertion scope does not match the canonical operation.
- Core restart during read, write, search, process, artifact, and checkpoint operations.
- Multi-session concurrent read and search with serialized or rejected conflicting writes.
- Client-rendered diff and model-context diff derive from the same core snapshot.
- A language server launched as a sandboxed process: the runtime holds the conversation, and any code action or formatting result is applied through the core mutation path with normal authority checks.
- Control-channel independence: cancellation and heartbeats are delivered while a large result streams on stdio.

## Security tests for later implementation

- Out-of-project paths, symlink escapes, broad roots, ambiguous globs, and path traversal are denied.
- Plugins, MCP servers, and clients cannot access core transport, raw artifact paths, or direct project filesystem and process authority.
- A path returned by a previous search or read is not treated as authority for a later operation; every operation re-canonicalizes.
- Process tools cannot inherit undeclared secrets or unrestricted environment variables.
- A secret value never appears in a protocol message body, a log, an operation result, or a diagnostic transcript. Resolution happens off the request path.
- A rotated or revoked secret reference fails cleanly rather than injecting a stale credential.
- Subprocess network access follows the requested and approved sandbox egress class.
- Logs and diagnostics redact secrets and avoid full file contents or command output by default.
- Large outputs are accessible only through opaque artifact references with permission checks.
- Restore cannot be used to write outside the project or to overwrite a file modified since capture.

## Fault-injection tests for later implementation

- Core child crash, protocol corruption, heartbeat timeout, and restart backoff.
- **The reconciliation matrix, which is the highest-risk area of this layer.** Kill the core after a write reaches the OS but before the response is delivered, then verify all three resolutions: hash matches post-image (completed), matches pre-image (never happened, retry safe), matches neither (conflict, never retried).
- Kill the core between journal write and mutation, and between mutation and journal completion.
- Duplicate request arriving for a key whose journal entry has no terminal outcome.
- Files changed externally during read, search, patch, and checkpoint restore.
- Command hangs, ignores termination, spawns descendants, or floods output.
- Artifact inventory divergence: orphaned bytes and dangling references detected and resolved at startup.
- Project directory renamed, deleted, or unmounted mid-session.
- Sweep requested for an artifact held by an in-flight operation.

## Performance tests for later implementation

- Directory walk and search latency across representative project sizes, since Phase 1 searches the filesystem directly with no index.
- Large-file read and artifact streaming memory usage.
- Process output streaming under bounded limits.
- Checkpoint capture cost for a turn touching many files.
- Control-channel latency while stdio carries a large result.

## Regression checks

- Run `git diff --check` for this documentation delivery.
- Run repository documentation/link validation when available.
- For implementation, run the documented full verification command on Windows, macOS, and Linux.

## Commands and results

- Pending until the documentation changes are complete.

## Residual risks

- No executable OS layer exists to validate these contracts.
- Exact sandbox mechanisms, journal durability, artifact retention, and default policy remain open.
- With no generated types, contract conformance depends entirely on vector coverage. A field no vector exercises can diverge silently between the two implementations.
- Searching without an index may prove too slow on large projects, which would pull the content-search-index package forward. The performance tests above are what would reveal it.


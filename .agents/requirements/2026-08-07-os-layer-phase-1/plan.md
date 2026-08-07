# Implementation Plan

This is a proposed later implementation sequence. No OS-layer implementation is part of the current requirements delivery.

Path canonicalization comes first because it is the security foundation every other operation depends on, and because it is where cross-platform behavior diverges most.

## Decisions first

1. [ ] Approve the OS-layer scope, the narrow-surface rule, and the runtime/core ownership matrix.
2. [ ] Fix the final Phase 1 method list and the default approval policy per operation class.
3. [ ] Write the runtime-to-core contract document: operation lifecycle, error classes, capability assertions, artifacts, two-channel framing.
4. [ ] Build the first shared test vectors, including malformed and hostile inputs.
5. [ ] Define cross-platform path canonicalization behavior and its conformance vectors.

## Foundation

6. [ ] Implement the RPC server: framing on both channels, validation, handshake, cancellation, progress.
7. [ ] Implement path canonicalization and project-boundary enforcement.
8. [ ] Implement capability assertion validation and OS policy enforcement.
9. [ ] Implement the operation journal, duplicate resolution, and recovery reporting.

## Operations

10. [ ] Implement file read, metadata, and hash.
11. [ ] Implement write, edit, and patch with preconditions and conflict detection.
12. [ ] Implement glob and directory walk with ignore policy and limits.
13. [ ] Implement content search with bounded previews and artifact fallback.
14. [ ] Implement artifact storage, read, and sweep execution.
15. [ ] Implement sandbox profile resolution and materialization per platform.
16. [ ] Implement process execution, output streaming, cancellation, and process-tree cleanup.
17. [ ] Implement secret reference resolution and environment injection.
18. [ ] Implement checkpoint capture, hash verification, and restore.

## Integration

19. [ ] Implement runtime-side tool adapters and policy preflight against these operations.
20. [ ] Add cross-platform conformance, security, and fault-injection tests, with fault injection covering the write-then-crash reconciliation paths.
21. [ ] Add performance tests for search, large reads, and process streaming.
22. [ ] Promote stable behavior into features/specs and record durable decisions.

## Deferred packages

Each needs its own requirement record and an explicit trigger, and none may re-enter this package: watchers, file catalog, content-search index, diff engine, VCS operations, PTY and interactive processes, toolchain inspection.

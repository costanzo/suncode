# Historical Runtime-to-Core Contract

Status: Superseded by `ADR-20260808-rust-unified-runtime`. This document remains only as migration history for the retired TypeScript/Rust split. The Phase 1 product uses one Rust runtime process and calls audited operations in-process.

The retired runtime-to-core transport was newline-delimited JSON-RPC 2.0 over private child-process stdio. It is no longer a production boundary.

## Envelope

Requests use JSON-RPC 2.0 with a string request ID. Notifications omit `id`. Every mutating operation includes `project_id`, `session_id`, `turn_id`, `tool_call_id`, and `idempotency_key` where applicable. The core never receives provider credentials or raw secret values.

Errors use:

```json
{
  "code": "capability_denied",
  "message": "stable human-readable summary",
  "retryable": false,
  "details": {}
}
```

`message` is diagnostic text and is not a machine contract. Clients and the runtime branch on `code`.

## Retired methods

| Method | Direction | Purpose |
| --- | --- | --- |
| `core/hello` | runtime -> core | Negotiate protocol and core capabilities |
| `core/health` | runtime -> core | Read health and child status |
| `project/open` | runtime -> core | Canonicalize, validate, and select a local project directory |
| `project/inspect` | runtime -> core | Read bounded project metadata |
| `fs/read` | runtime -> core | Read a canonical in-project file |
| `fs/metadata` | runtime -> core | Read bounded file metadata and hash |
| `search/glob` | runtime -> core | Recursively list bounded in-project glob matches |
| `search/find` | runtime -> core | Find bounded text matches with line previews |
| `fs/write` | runtime -> core | Write a bounded in-project file when its pre-image matches |
| `fs/edit` | runtime -> core | Apply preconditioned text replacements |
| `fs/patch` | runtime -> core | Apply a preconditioned unified text patch |
| `fs/move` | runtime -> core | Move one regular in-project file |
| `fs/delete` | runtime -> core | Delete one regular in-project file |
| `artifact/read` | runtime -> core | Read a bounded opaque artifact |
| `artifact/sweep` | runtime -> core | Delete runtime-marked artifact IDs |
| `process/run` | runtime -> core | Run one bounded non-interactive process |
| `process/start` | runtime -> core | Start a cancellable non-interactive process |
| `process/status` | runtime -> core | Read a managed process status |
| `checkpoint/restore` | runtime -> core | Restore one checkpoint after verifying the current post-image |
| `capability/check` | runtime -> core | Authoritatively evaluate an asserted operation |
| `capability/execute` | runtime -> core | Execute an already authorized operation |
| `operation/cancel` | runtime -> core | Cooperatively cancel an operation |
| `operation/reconcile` | runtime -> core | Resolve unknown completion after restart |
| `operation/status` | runtime -> core | Read a journaled operation status |
| `core/recovery` | runtime -> core | Report pending operations and managed artifacts |

The retired implementation included every method in the table. Paths were canonicalized and kept project-relative in results. Mutations required pre-images, capture opaque checkpoints, and used an optional core-private journal keyed by the supplied idempotency key. Text edit and patch rejected stale context; move and delete captured enough pre-image state for reverse restore. Process execution was argv-based, non-interactive, bounded, environment-filtered, and cancellable for managed starts. Large reads and process output used artifact IDs.

The current embedded operations contract retains structured argv semantics for `process/run`: callers provide a program and string argument array, and the operation never invokes a shell implicitly. The current agent exposes platform shell scripts separately and resolves them to Windows PowerShell on Windows or POSIX `/bin/sh` on macOS/Linux before entering the audited process operation.

## Capability assertion

An assertion names one operation and one canonical scope:

```json
{
  "operation": "fs.read",
  "project_id": "project-1",
  "resource": {"path": "src/index.ts"},
  "scope": {"kind": "project", "project_id": "project-1"},
  "grant_id": "grant-1",
  "expires_at": "2026-08-07T00:10:00Z"
}
```

Rust re-canonicalizes paths and independently checks scope, expiry, operation class, and limits. A runtime decision never replaces core enforcement.

## Bounded results

Read results include `truncated`, `bytes`, and an optional opaque `continuation`. Large output must use an artifact reference rather than an unbounded JSON field. File contents are never written to logs.

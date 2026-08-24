# Historical Runtime-to-Core Contract

Status: Superseded by `ADR-20260808-rust-unified-runtime`. This document remains only as migration history for the retired TypeScript/Rust split. The Phase 1 product uses one Rust agent process and calls audited operations in-process.

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
| `fs/move` | runtime -> core | Move one regular in-project file |
| `fs/delete` | runtime -> core | Delete one regular in-project file |
| `checkpoint/restore` | runtime -> core | Restore one checkpoint after verifying the current post-image |

Persisted historical tool rows remain readable as data, but removed operation names are no longer executable.

The current embedded operations contract exposes canonical `tool/*` methods for the seven model tools and a typed checkpoint restore method. `bash` resolves its command to Windows PowerShell on Windows or POSIX `/bin/sh` on macOS/Linux before entering the private audited process runner.

## Bounded results

Read results include `truncated`, `bytes`, and an optional opaque `continuation`. Large output must use an artifact reference rather than an unbounded JSON field. File contents are never written to logs.

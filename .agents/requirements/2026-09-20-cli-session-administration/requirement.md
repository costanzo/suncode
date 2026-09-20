# Requirement

## Background

The CLI can create sessions through one-shot `run` but cannot inspect or archive them. This delivery intentionally excludes resume rather than exposing a snapshot-only placeholder; its later one-shot turn semantics are defined by `requirements/2026-09-20-cli-session-resume/`.

## Goals

- List primary sessions for a project.
- Archive one primary session by ID.
- Preserve SDK ownership of project identity, user scope, persistence, and lifecycle validation.

## Non-goals

- Session resume or interactive chat.
- Reopening, renaming, pinning, deleting, or inspecting child sessions.
- Direct SQLite access or client-side session filtering authority.

## Requirements

- Add `suncode session list [PATH]`, defaulting to the current directory.
- Add `suncode session archive SESSION_ID`.
- Use only `AsyncAgentSdk::open_project`, `list_sessions`, and `archive_session`.
- Emit human-readable text or versioned JSONL results through the existing output contract.
- Include active and archived primary sessions in list output.
- Do not expose `session resume` in this delivery.
- Consume explicit SDK shutdown before reporting success.

## Edge cases

- A missing project/session follows stable SDK error handling.
- Child sessions remain read-only and cannot be archived directly.
- Re-archiving an archived session follows the SDK's idempotent lifecycle behavior.

## Acceptance criteria

- Argument and child-process integration tests cover list, archive, JSONL shape, and unavailable resume grammar.
- Required CLI and broader regression checks pass.

## Open questions

- None.

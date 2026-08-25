# Architecture

## Current state

The Rust agent builds provider input from a host environment system message, optional dependency context, and persisted session messages. File reads execute through the audited operations package and return normalized tool results.

## Proposed design

Core reads `<project>/AGENTS.md` directly through a bounded internal instruction loader before each provider request. It emits a system message without exposing the canonical absolute project path. This keeps guidance fresh while avoiding durable duplication in session history.

After a successful project `read`, core canonicalizes the target and walks parent directories up to, but excluding, the project root. Unseen nested `AGENTS.md` files are attached nearest-first in a structured `repository_instructions` field on that read result, matching OpenCode's system-reminder behavior while preserving SunCode's provider-neutral JSON tool result contract.

## Boundaries and dependencies

- Instruction discovery is Rust-owned and does not bypass the operations dispatcher for model-requested file access.
- Automatic instruction reads are project-scoped metadata reads, never dependency or parent-directory reads.
- The continuation stores only relative paths already attached during the current turn.

## Data and control flow

1. Build the compacted conversation context.
2. Load the root `AGENTS.md` and append it after host environment context.
3. Execute model-requested reads normally.
4. On read success, discover nested instructions, add them to the normalized result, and remember their relative paths.
5. Persist and return the resulting tool JSON through the existing tool-use path.

## Security and failure handling

Canonical path checks prevent symlink escape. Size, file-count, and total-byte limits bound prompt impact. Invalid or unavailable instruction files are ignored rather than making ordinary agent work unavailable.

## Compatibility and migration

This is a new project contract and adds no compatibility behavior or database migration.

## Risks and rollback

Repository instruction content can direct model behavior but cannot grant authority; policy, approvals, project scope, and audited operations remain enforced independently.

## Open questions

- None.

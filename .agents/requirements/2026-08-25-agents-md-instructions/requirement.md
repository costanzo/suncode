# Requirement

**Status: Complete**

## Background

SunCode did not automatically load repository guidance. OpenCode loads project instructions into the system context and discovers more specific instruction files when reading files below nested directories.

## Goals

- Automatically load a project-root `AGENTS.md` into every provider request.
- Discover more specific `AGENTS.md` files when the agent reads a file in a nested directory.
- Apply narrower directory instructions before broader directory instructions and avoid duplicate attachment within one turn.
- Bound instruction context and keep all discovery inside the opened project.

## Non-goals

- Loading `CLAUDE.md`, deprecated `CONTEXT.md`, remote URLs, or global OpenCode configuration.
- Reading instruction files outside the opened project or from registered dependencies.
- Adding configuration or compatibility aliases.

## Requirements

1. A non-empty project-root `AGENTS.md` is a system message on each model call and reflects file edits made during the session.
2. A successful `read` of a project file walks from that file's directory toward the project root and attaches previously unseen nested `AGENTS.md` files nearest-first.
3. The root file is not duplicated in read results, and directly reading an `AGENTS.md` file does not self-attach it.
4. Instruction files must resolve inside the canonical project root and be regular UTF-8 files no larger than 32 KiB.
5. One read attaches at most 16 instruction files and 64 KiB of nested instruction content.
6. Missing, unreadable, oversized, invalid UTF-8, or out-of-scope files are ignored without failing the turn.

## Edge cases

- A symlinked `AGENTS.md` resolving outside the project is ignored.
- Dependency reads do not import dependency-owned instructions.
- Context compaction may summarize historical read results, while the root instruction is always reloaded for the next provider request.

## Acceptance criteria

- Root instruction system injection and nested read attachment tests pass.
- Existing agent tool, recovery, SDK, and desktop tests remain green.
- Formatting, clippy, build, and diff checks pass.

## Open questions

- None.

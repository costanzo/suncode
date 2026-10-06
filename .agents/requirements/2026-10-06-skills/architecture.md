# Architecture

## Current state

The Rust agent builds a structured system prompt and a model-facing built-in tool catalog. Project instructions are loaded by `agent/crates/core`, and clients consume the Rust SDK. No Skill catalog exists.

## Proposed design

`agent/crates/skills` owns Skill discovery, YAML frontmatter parsing, validation, precedence, collision diagnostics, bounded document loading, and metadata rendering. `agent/crates/core` creates a per-turn catalog snapshot from the opened project root, passes visible metadata to the system prompt, and implements the `skill` tool by calling the crate API.

## Boundaries and dependencies

`suncode-skills` depends only on serde, serde_json, serde_yaml, and the Rust standard library. It does not depend on core, data, SQLite, providers, policy, SDK, or clients. Core depends on `suncode-skills`; the tools crate owns only the static model schema.

## Data and control flow

At turn preparation, core discovers Skills and adds escaped name/description/location metadata to the primary system prompt. A model `skill` call is read-only and uses the same project root to build a catalog, validates the requested name, loads bounded Markdown, and returns relative resource names. The tool result enters the existing session call/message projections.

## Security and failure handling

Project discovery is canonicalized and restricted to the project root. File size and resource count are bounded. Skill text is untrusted and carries an explicit warning in tool output. Skill loading has read-only risk and never bypasses ordinary tool policy. Child agents retain their existing immutable allowlists and never receive `skill`.

## Compatibility and migration

No SQLite migration is required. Existing sessions remain valid because Skill metadata is turn-time context and loaded content is ordinary tool history. The crate is a new workspace member.

## Risks and rollback

The main risk is prompt growth from large catalogs; metadata is bounded by file validation and can later gain a catalog count limit. Rollback removes the crate dependency and tool registration without changing persisted schema.

## Open questions

Client Skill listing and explicit invocation will be specified separately when interactive composer contracts are ready.

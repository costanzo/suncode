# Requirement

## Background

SunCode needs reusable, project-local coding instructions comparable to OpenCode and pi Agent Skills. The feature must remain Rust-owned and must not introduce a TypeScript, Node.js, plugin, or remote execution dependency.

## Goals

- Discover local `SKILL.md` files from project and user locations.
- Make concise Skill metadata available to the primary model.
- Let the primary model load a Skill document on demand through an audited read-only tool.
- Keep Skill filesystem access in a dedicated `suncode-skills` crate under `agent/crates`.
- Preserve SunCode authority, project boundaries, and existing child-agent tool allowlists.

## Non-goals

- Remote Skill downloads, package registries, executable plugins, or marketplace support.
- Persisting Skill documents in SQLite.
- Giving specialist child agents an independent Skill catalog in this delivery.
- Implementing interactive slash commands before the interactive CLI/composer contract exists.

## Requirements

- Supported locations are `<project>/.suncode/skills`, `<project>/.agents/skills`, `~/.config/suncode/skills`, and `~/.config/suncode/.agents/skills`.
- Project native locations have precedence over project compatibility locations, then user native, then user compatibility.
- Every Skill is a directory containing `SKILL.md` with YAML frontmatter and Markdown content.
- `name` must match `^[a-z0-9]+(-[a-z0-9]+)*$`, and `description` is required and bounded to 1024 characters.
- Invalid files produce diagnostics and do not enter the catalog. Duplicate names keep the first deterministic winner and produce a collision diagnostic.
- `disable-model-invocation: true` hides a Skill from model metadata and rejects model tool loading.
- Skill content is untrusted guidance and cannot grant authority, change policy, approve actions, or request secrets.
- The `skill` model tool is read-only, bounded, and returns body content plus a bounded relative resource listing.

## Edge cases

- Missing directories are empty sources.
- Broken symlinks, invalid UTF-8, invalid YAML, missing frontmatter, path escape, and oversized files produce warnings.
- Skill resources are metadata only in the first delivery; execution remains subject to ordinary audited tools.

## Acceptance criteria

- `suncode-skills` has focused tests for parsing, discovery precedence, collisions, escaping, and bounded resources.
- Primary turns advertise available Skills and can load an allowed Skill through `skill`.
- Child agents do not advertise or receive the Skill tool.
- Existing agent, SDK, and CLI tests remain green.

## Open questions

- Whether the Avalonia composer should expose a Skill selector or slash command after the interactive input contract is finalized.
- Whether `.claude/skills` and `.opencode/skills` compatibility should be opt-in in a later delivery.

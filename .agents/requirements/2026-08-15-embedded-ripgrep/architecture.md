# Architecture

## Current state

`agent/crates/operations/src/search.rs` recursively reads regular files and calls `str::find`. It neither uses ripgrep libraries nor implements its advertised regular-expression semantics.

## Proposed design

Use ripgrep's `ignore` crate for project traversal, `globset` for include filtering, `grep-regex` for the default Rust-regex matcher, and `grep-searcher` for bounded line-oriented file search.

## Boundaries and dependencies

The implementation remains inside the audited Rust operations crate. The runtime core continues to normalize provider-facing tool arguments and dispatch `tool/grep`/`search/find`; it does not gain filesystem or matcher dependencies.

## Data and control flow

1. Validate query, include pattern, and result bound.
2. Compile the regular expression and glob matcher.
3. Walk project files using ripgrep ignore semantics without following links.
4. Sort accepted project-relative paths for deterministic results.
5. Search each bounded file and convert matcher events into the existing JSON response.

## Security and failure handling

Traversal begins only at the canonical project root and does not follow symbolic links. Invalid regex/glob input is rejected. File read/search failures are skipped consistently with the existing best-effort search behavior. No shell or external executable is used.

## Compatibility and migration

The method names and response schema remain stable. Query syntax changes from literal-only matching to Rust regular expressions, matching the existing tool description. Literal punctuation must be escaped when it has regex meaning.

## Risks and rollback

Ignore defaults can omit files previously searched. Tests make this intentional behavior explicit. Rollback consists of removing the four dependencies and restoring the prior `find` implementation.

## Open questions

- PCRE2 remains deferred.

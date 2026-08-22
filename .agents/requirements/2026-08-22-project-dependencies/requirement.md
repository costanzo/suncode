# Requirement

## Background

An opened project may depend on source in other local folders. Users need to browse and let the agent inspect that code without switching the active project or granting the related folders mutation authority.

## Goals

- Register and remove related source folders per project.
- Browse the main project and dependencies from one left-side Explorer.
- Let the agent read and search dependency code through stable aliases.
- Keep dependencies strictly read-only.

## Non-goals

- Editing dependency files, running processes in them, or including them in Git/checkpoint/undo scope.
- Filesystem indexing, watching, file preview, or a general-purpose editor.
- Remote dependencies, package resolution, or dependency graph inference.

## Requirements

- The project window has a dedicated Explorer gutter that switches the existing left sidebar in place and can collapse when selected again.
- Explorer displays the current project tree and a `Dependencies` group of registered folder trees.
- Directory children load lazily through the Rust SDK; Avalonia does not read the filesystem directly.
- Users can add one folder with the native folder picker and remove a registration without deleting files.
- Rust canonicalizes roots and rejects self, ancestor, descendant, duplicate, and overlapping registrations.
- Client and model DTOs do not expose dependency absolute roots.
- The model addresses dependency content through `dependency:<dependencyId>/<relativePath>` and can use only read, glob, and grep.
- Search/read results preserve the dependency alias.

## Edge cases

- Missing, inaccessible, non-directory, escaped, or removed roots return stable SDK/operation errors.
- Symlinks are omitted from Explorer directory listings and are not followed.
- Directory listings are bounded and report truncation.
- An existing otherwise-current 13-table database receives the new empty table transactionally; incompatible databases remain unchanged and rejected.

## Acceptance criteria

- Adding a valid non-overlapping folder makes it appear under `Dependencies` and persists across launch.
- Main and dependency directory roots expand and refresh correctly.
- Removing a dependency removes only its registration.
- Agent read/glob/grep can inspect registered dependency content and returns aliased paths.
- Dependency aliases are rejected by write, edit, process, Git, checkpoint, and other operations.
- Rust workspace tests, Avalonia build, contract validation, and diff checks pass.

## Open questions

- None for this delivery.

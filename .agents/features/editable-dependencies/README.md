# Editable project dependencies

Status: Implemented and focused-tested on Windows.

## Behavior

- A project retains its main root and registered dependency roots. Existing registrations participate in the same session policy as main-root operations. Reads and searches are allowed by the interactive default; mutations and processes require approval unless Full Control applies.
- Model context advertises canonical dependency paths. Built-in file tools and bash workdir accept real paths and legacy `dependency:<id>/...` aliases. Relative paths and an omitted workdir use the main root. Shell command text uses ordinary filesystem paths and is passed intact to the captured host shell.
- Root selection occurs before the existing audited operation dispatcher. Canonical containment, hash preconditions, mutation serialization, output bounds, cancellation, and command safety checks still apply. Unexecuted operations re-resolve current registrations after approval, so removal revokes their dependency scope.
- Model-visible dependency file results and semantic locations use real paths. Private checkpoint metadata and retained turn-change keys use `dependency:<id>/<relative>` to distinguish same-named files across repositories. Main-root keys remain relative, and SDK turn_diff uses existing stored versions.
- Root instructions prefer AGENTS.override.md, AGENTS.md, then CLAUDE.md, with explicit directory-tree scope. Successful reads discover bounded nested instructions nearest-first. Dependency instruction identities are absolute, avoiding collisions with main-root or other dependency instructions.
- Existing SDK registration, Explorer selection, SQLite schema, and native ABI remain compatible. The desktop file viewer continues to be read-only. Source Control and LSP lifecycle remain main-project scoped.

## Verification

- Agent core: 75 tests passed after excluding the existing single-instance-lock test. The unfiltered run passed 75 and failed that one assertion because Windows returned ErrorKind::Uncategorized instead of AlreadyExists. The lock implementation was not changed.
- Tool operations: all 75 library tests passed, including dependency mutation preconditions, qualified checkpoint identities, real/alias workdirs, default cwd, and unchanged shell command text.
- Core regression cases exercise ordinary approvals, Full Control, same-name changes in separate roots, retained diff content, real shell execution in a dependency, removal while awaiting approval, and scoped/deduplicated instructions.
- Rust SDK registration/browsing regression passed. Locked C SDK cargo check passed.
- Native macOS/Linux execution and production UI runtime conformance have not been exercised for this change. No UI source or layout changes are part of this delivery.

## Limits

File-change review covers built-in file mutations. The current SDK has no public file undo operation. Processes report os_isolation=false and network=not_enforced; their cwd and policy checks do not constitute an OS sandbox. Independent directory permission profiles and multi-repository Git/LSP management remain deferred.

# Architecture

## Current state

The unified `configuration` table already supports session scope. Policy currently evaluates only tool risk and non-interactive mode, while approval decisions support `allow_once` and `deny`.

## Proposed design

Use a boolean session configuration key named `full_control`. The database approval transaction handles `allow_session`, storing the grant only when it successfully resolves a pending approval. Agent policy reads the session grant before evaluating each tool call.

Avalonia reads effective settings through the existing SDK settings method and writes `full_control=false` through the existing setting method. It never accesses SQLite directly.

## Boundaries and dependencies

- SQLite owns the durable grant and atomic approval transition.
- Runtime core owns policy interpretation.
- The SDK exposes the existing typed settings boundary plus the expanded approval decision.
- Avalonia owns only presentation and user interaction.

## Data and control flow

`Allow for session` -> `resolve_approval(allow_session)` -> one SQLite transaction updates approval and session configuration -> continuation resumes -> later policy checks allow known risky tools.

`Turn off` -> session setting `full_control=false` -> later policy checks return to normal approval behavior.

## Security and failure handling

Full Control skips interactive approval for known project-write, process, and destructive-write tool risks. It does not skip schema translation, canonical path checks, dependency restrictions, cancellation, operation auditing, checkpoints, or unknown-tool denial. A failed approval resolution rolls back the grant.

## Compatibility and migration

No schema migration is required because `configuration` already accepts session-scoped keys. Existing approvals and sessions default to Full Control off.

## Risks and rollback

The elevated state increases the consequence of an incorrect model action. Persistent warning UI and a direct off action mitigate accidental long-lived grants. Removing the configuration row or treating it as false restores prior policy behavior.

## Open questions

- None.

# Architecture

## Current state

One-shot `run` creates a new project session and already owns prompt input, typed events, cancellation, lag recovery, tail draining, and terminal response adaptation. Session list/archive use typed SDK methods.

## Proposed design

Extract the one-turn driver so both new-session `run` and existing-session `session resume` use the same event and signal behavior. Resume calls the SDK's primary-session reopen method, atomically watches the session, verifies there is no durable suspension, then submits one turn.

Add one narrow Rust SDK query for the current pending approval by validated session. Pending question state already exists in `SessionSnapshot`. The CLI uses these only to fail closed; it does not resolve them.

## Boundaries and dependencies

The CLI owns grammar and output adaptation only. SDK/core continue owning session history, model context, user scope, lifecycle validation, approval/question persistence, providers, policy, and operations. No C ABI or desktop changes are required.

## Data and control flow

1. Read one prompt source.
2. Reopen and atomically watch the supplied primary session.
3. Reject pending approval/question state with exit 4.
4. Submit a new turn with a fresh idempotency key.
5. Consume typed events, resynchronize after lag, cancel on interrupt, and drain ready tail events.
6. Emit `session.resume.result`, then consume SDK shutdown.

## Security and failure handling

The CLI never reads SQLite or bypasses a durable suspension. Credentials and provider access remain core-owned. Child sessions remain read-only. Browser/Computer host capabilities stay disabled.

## Compatibility and migration

The command grammar and Rust SDK pending-approval query are additive. No schema, C ABI, managed SDK, or existing CLI result changes.

## Risks and rollback

The main risk is divergence between run and resume turn-driving behavior; sharing one implementation prevents it. Removing the command and additive query requires no data migration.

## Open questions

- None.

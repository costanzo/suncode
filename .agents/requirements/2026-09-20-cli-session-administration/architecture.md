# Architecture

## Current state

`apps/cli` embeds `AsyncAgentSdk` and implements administrative commands plus one-shot `run`.

## Proposed design

Add a nested `session` command family with `list` and `archive`. Route both through the existing administrative dispatcher and `CommandReport` output adapter.

## Boundaries and dependencies

The CLI does not read SQLite, infer ownership, or mutate session rows. Project canonicalization, user scoping, child-session restrictions, and lifecycle writes remain SDK/core-owned. No new dependency is required.

## Data and control flow

- List: open/select project through the SDK, call `list_sessions`, render returned records and UI states.
- Archive: call `archive_session` with the supplied stable ID and render the returned session record.

## Security and failure handling

The SDK verifies that projects and sessions belong to the configured user. Child lifecycle changes fail closed. No credentials or provider calls are involved.

## Compatibility and migration

The grammar and JSONL result types are additive. Persistence, C ABI, desktop behavior, and existing CLI commands are unchanged.

## Risks and rollback

The main UX risk was exposing an underspecified resume command in this administrative slice. It remained unexposed here and was later delivered as a separate one-shot-turn command. The additive list/archive variants can be removed without data migration.

## Open questions

- None.

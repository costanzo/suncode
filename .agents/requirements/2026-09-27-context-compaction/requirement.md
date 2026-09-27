# Requirement

## Background

The previous in-memory compaction was discarded after a turn and did not account for fixed request context.

## Goals

- Reuse a durable summary and retained tail across turns without changing transcript or audit history.
- Preserve complete tool-call/result groups and account for system, tools, and output budget.
- Recover once from provider context overflow.

## Non-goals

- New public SDK methods, a new SQLite table, or a general migration system.
- A provider-independent tokenizer or compaction model selector.

## Requirements

- Store each successful compaction checkpoint with exact rowid boundaries.
- Prefer a bounded structured model summary; retain the local summary on failure.
- Keep the model-facing summary within the retained-context budget.
- Keep raw retained messages out of the live compaction event payload.

## Edge cases

- Events inserted in the same millisecond as a checkpoint must replay correctly.
- Repeated compactions select the latest checkpoint.
- An overflow retry must run at most once per turn.

## Acceptance criteria

- A later turn loads the summary, retained tail, and all post-checkpoint messages.
- Focused core/data tests and diff checks pass.

## Open questions

- Future work may add provider-native token counting and a dedicated summary model policy.

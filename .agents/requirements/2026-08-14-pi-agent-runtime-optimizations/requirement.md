# Requirement

## Background
PI Agent has useful harness behavior around queued user messages, batched tool execution, and context-window-aware compaction. Suncode should selectively adopt those ideas without changing its Phase 1 Rust runtime ownership, approval model, SQLite event stream, checkpoint behavior, or Qt-only client scope.

## Goals
- Accept a user submission while a session turn is running by queueing it for the active turn, similar to PI's steering/follow-up drain points.
- Preflight a model-returned tool-call batch before execution and execute read-only batches concurrently when no approval is required.
- Replace fixed character-only compaction with model-window-aware token estimation using reserve and recent-tail token settings.

## Non-goals
- Introduce PI's lane/tree session model.
- Add durable queue records or change SQLite schema in this slice.
- Change Suncode's approval, audit, operation dispatcher, checkpoint, undo, provider credential, or Qt boundary model.
- Enable parallel filesystem writes, process execution, or approval-gated operations.

## Requirements
- Running-turn submissions return an explicit queued response and emit a `turn.queued` event.
- Queued messages are injected as `message.user` records at safe drain points before the next provider call or before turn completion.
- Tool calls in one assistant message are validated and policy-checked before an eligible allowed batch executes.
- Only all-read-only batches may execute concurrently; all other batches execute through the existing sequential path.
- Compaction thresholds use the active model's input window when known, capped by Suncode's existing 64k turn token budget.

## Edge cases
- Duplicate queued idempotency keys return the existing queued item.
- Queued messages are discarded when the active turn fails, is cancelled, or approval is denied.
- Approval-required calls still suspend before the risky call executes.
- Read-only batch failures still fail the turn, matching existing operation failure behavior.

## Acceptance criteria
- Focused Rust tests cover queued message injection, read-only batch preflight/execution, approval behavior, and compaction token thresholds.
- `git diff --check` passes.

## Open questions
- Whether queued messages should become durable SQLite records if Suncode later supports crash-resumable running turns.
- Whether Qt should expose an active-turn "send follow-up" affordance instead of only backend/API support.

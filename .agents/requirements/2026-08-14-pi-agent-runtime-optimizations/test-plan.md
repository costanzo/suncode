# Test Plan

## Scope
Runtime agent behavior only: running-turn queueing, read-only tool batch execution, approval preservation, and context compaction thresholds.

## Unit tests
- `cargo test -p suncode-agent agent::tests::queued_submit_is_injected_before_completion`
- `cargo test -p suncode-agent agent::tests::read_only_tool_batch_is_preflighted_before_execution`
- `cargo test -p suncode-agent agent::tests::write_waits_for_approval_and_captures_checkpoint`
- `cargo test -p suncode-agent context::tests::compacts_when_estimated_tokens_exceed_model_window_reserve`

## Integration and conformance tests
- `cargo test -p suncode-agent`

## Regression checks
- `git diff --check`

## Manual checks
None planned for this backend-focused slice. Qt receives only a status text update for queued responses.

## Commands and results
- `cargo test -p suncode-agent agent::tests::queued_submit_is_injected_before_completion` passed.
- `cargo test -p suncode-agent agent::tests::read_only_tool_batch_is_preflighted_before_execution` passed.
- `cargo test -p suncode-agent context::tests::compacts_when_estimated_tokens_exceed_model_window_reserve` passed.
- `cargo test -p suncode-agent agent::tests::write_waits_for_approval_and_captures_checkpoint` passed.
- `cargo test -p suncode-agent` passed with 23 tests.
- `git diff --check` passed.

## Residual risks
The queue is in-memory and not crash-resumable; this is intentional for the current selective adoption.

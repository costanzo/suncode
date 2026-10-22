# Test Plan

## Scope

Verify transient session queue behavior across Rust agent, SDK/native boundary, and Avalonia.

## Unit tests

- Queue FIFO, idempotency, complete payload retention, promote/retry/remove/restore/take-back.
- Steering is consumed at the next provider-call boundary and is not submitted as a new turn.
- Follow-ups are admitted only after terminal turn completion and each receives a distinct turn.
- Failed validation and turn termination preserve or safely downgrade queued items.

## Integration and conformance tests

- Rust SDK operations and C/C# serialization preserve queue DTO/event fields.
- Agent event/snapshot behavior reconciles queue mutations after lag.
- Attachments survive queue operations and revalidate model vision support at admission.

## Regression checks

- Existing submit, cancel, approval/question continuation, shutdown, and session-watch tests.
- Desktop composer, input mode, turn status, and session switching.

## Manual checks

- Queue multiple text/image messages while streaming; promote one to steering.
- Confirm current model/tool action continues, steering appears at the next request, then follow-ups run FIFO.
- Collapse queue while streaming and verify composer/output remain visible.
- Expand/collapse long text; take back, edit, remove/undo, and check focus and full payload.
- Verify light/dark themes, narrow window, keyboard access, and screen-reader names.

## Commands and results

## Residual risks

- Pending queue state is intentionally in-memory and is discarded when the application process shuts down.

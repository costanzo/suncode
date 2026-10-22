# Todo

## Design

- [x] Confirm the queue supports multiple messages and default follow-up semantics.
- [x] Confirm steering waits for the active turn's next model request and never cancels current work.
- [x] Confirm queue row mode names and fallback behavior against implementation.

## Implementation

- [x] Add Rust queue row snapshots and mutation methods.
- [x] Separate after-turn follow-up admission from in-turn steering consumption.
- [x] Preserve image references and full text through queue actions.
- [x] Add SDK, C ABI, and C# bindings.
- [x] Add Avalonia queue list, row actions, collapse, disclosures, and undo.

## Verification

- [x] Add focused agent tests for queue behavior.
- [x] Check the additive C ABI and managed binding compile contracts.
- [x] Build Rust SDK/ABI and desktop projects; run focused queue tests.
- [ ] Verify themes, narrow layouts, keyboard behavior, and accessibility labels in a running desktop client.
- [x] Run `git diff --check`.

## Closeout

- [x] Update feature and SDK contract descriptions.
- [x] Record stable behavior in the agent feature and SDK contract docs.

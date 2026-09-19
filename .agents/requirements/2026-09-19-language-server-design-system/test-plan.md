# Test Plan

## Scope

Validate the design-system Language Server Settings specimen and ensure no production Avalonia or Rust source is changed by this delivery.

## Unit tests

- Not applicable; the specimen uses local interactive state.

## Integration and conformance tests

- Build the Vite review application.
- Exercise navigation, add/edit, enable/disable, retry, and delete interactions in the browser.

## Regression checks

- Verify the existing Settings destinations remain navigable.
- Verify dark and light themes.
- Verify the narrow Settings composition.

## Manual checks

- Inspect hierarchy, wrapping, status semantics, focus order, authority copy, editor-window overflow, and empty/failure states.

## Commands and results

- Recorded in the completion response after execution.

## Residual risks

- Production behavior is intentionally unimplemented and therefore not covered by this design delivery.

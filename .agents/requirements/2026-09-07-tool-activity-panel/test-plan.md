# Test Plan

## Scope

Design-system route and interaction verification only.

## Unit tests

## Integration and conformance tests

## Regression checks

## Manual checks

- Tool tree expansion and detail selection.
- Standalone no-turn, active-turn, and completed-turn primary states.
- Running live-output state and tail-follow affordance.
- Turn hover user-message preview.
- Inline active-tool jump affordance.
- Compact-width stacking and focus visibility.

## Commands and results

- `npm run build`: passed, 189 modules transformed.
- `git diff --check`: passed.
- Browser console warning/error check: passed with no entries.
- Responsive inspection at 1280px and 620px: passed without horizontal overflow.

## Residual risks

- Production Avalonia projection, event buffering, focus transfer, and drawer restoration remain intentionally unimplemented pending design approval.

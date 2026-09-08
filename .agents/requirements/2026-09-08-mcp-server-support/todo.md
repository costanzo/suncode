# Todo

## Design

- [x] Draft the requirement.
- [x] Draft the architecture.
- [x] Add the design-system prototype.
- [x] Confirm global definitions with project-scoped connections, initial transports, tools-only scope, and authority policy.
- [x] Approve the architecture.
- [x] Update and verify the separate-window prototype.

## Implementation

- [x] Run a Rust MCP SDK compatibility spike.
- [x] Implement persistence and runtime lifecycle.
- [x] Implement tool catalog, policy, audit, and result normalization.
- [x] Implement SDK bindings and Avalonia UI.
- [x] Add focused tests at the persistence, adapter, runtime, SDK, ABI, and desktop boundaries.

## Verification

- [x] Run focused Rust, binding, and desktop checks.
- [x] Run required broader automated checks.
- [x] Verify design-system dark/light, keyboard dismissal, backdrop absence, overflow, and represented runtime states.
- [ ] Exercise real third-party stdio and Streamable HTTP servers across supported desktop platforms.
- [ ] Complete a manual native-window pass at the 520 x 520 minimum on every supported desktop platform.

## Closeout

- [x] Update features and specifications.
- [x] Record accepted decisions.
- [ ] Remove the delivery package when its active decision trail has been consolidated.

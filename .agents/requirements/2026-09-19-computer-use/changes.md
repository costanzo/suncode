# Changes

## Source

- Extend the pinned Enigo source with public display/capture contracts and platform backends.
- Add `agent/crates/computer/` for action validation and serial desktop execution.
- Add an Anthropic Messages adapter and provider-neutral client-toolset contracts.
- Integrate ComputerManager, policy, approvals, auditing, cancellation, recovery, and screenshot context.
- Add Rust/C/C# SDK management methods.
- Add design-system and Avalonia Computer Use Settings and approval surfaces.

## Contracts and generated artifacts

- Update the hand-written agent SDK contract and C ABI version.
- Document Computer Use provider and action-result contracts.
- No contract generation is introduced.

## Configuration and persistence

- Add bounded global Computer Use enablement and execution-limit settings.
- Add write-only Linux portal restore-token storage if required by the final backend.
- Add native Computer Use model capability metadata.
- Persist redacted action outcome metadata only; image bytes and sensitive typed text remain transient.

## Tests

- Add Enigo capture/input coordinate conformance tests.
- Add `suncode-computer` action, batch, cleanup, and image tests.
- Add Anthropic Messages wire tests.
- Add core policy, approval, cancellation, recovery, and redaction tests.
- Add SDK parity and Avalonia Settings tests.
- Add real-platform smoke tests for supported desktop targets.

## Documentation

- Record the accepted first-party Computer Use architecture decision.
- Update product, architecture, specifications, SDK, persistence, security, design, and release records when behavior becomes implemented.

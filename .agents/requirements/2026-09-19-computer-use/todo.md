# Todo

## Design

- [x] Confirm the requirement.
- [x] Review the architecture.
- [x] Add the Computer Use Settings specimen before production UI work.
- [ ] Add/verify the Computer Use approval specimen.
- [ ] Verify permission, unsupported, active, interrupted, and error states in both themes.

## Enigo

- [x] Add public display/capture/frame contracts.
- [x] Add mock capture tests and coordinate transforms.
- [ ] Complete macOS capture permission status and production conformance.
- [ ] Implement Windows capture and DPI-consistent coordinates.
- [ ] Implement X11 capture in the root coordinate space.
- [ ] Unify Wayland RemoteDesktop and ScreenCast through one portal session.
- [ ] Replace the XDG absolute-pointer workaround and expose capability truth.

## SunCode implementation

- [x] Add `suncode-computer` and all 17 actions.
- [ ] Complete serial batch approval continuation; emergency stop, cancellation, and held-input cleanup are implemented.
- [ ] Complete image resizing and context pruning; zoom, PNG encoding, and frame generations are implemented.
- [x] Add Anthropic Messages and native client-toolset support.
- [x] Add ComputerManager, policy, approval, audit, persistence redaction, and recovery foundations.
- [x] Add Rust/C/C# SDK enablement, runtime-info, and emergency-stop APIs.
- [ ] Complete Avalonia permission onboarding and approval UI; Settings enablement/runtime/emergency-stop UI is implemented.

## Verification

- [x] Run focused Enigo and Rust checks/tests that do not generate real input.
- [x] Build the SDK and Avalonia desktop client.
- [ ] Run macOS, Windows, X11, and Wayland conformance smoke tests.
- [x] Run `git diff --check` in the SunCode repository.

## Closeout

- [ ] Update features and specifications.
- [x] Record the architectural decision.
- [ ] Complete contracts, security, design, packaging, and release documentation; core SDK/persistence/design documents are updated.

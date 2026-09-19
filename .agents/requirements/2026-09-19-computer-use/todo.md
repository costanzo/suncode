# Todo

## Design

- [x] Confirm the requirement.
- [x] Review the architecture.
- [ ] Add the Computer Use Settings and approval specimens before production UI work.
- [ ] Verify permission, unsupported, active, interrupted, and error states in both themes.

## Enigo

- [ ] Add public display/capture/frame contracts.
- [ ] Add mock capture tests and coordinate transforms.
- [ ] Implement macOS capture and permission status.
- [ ] Implement Windows capture and DPI-consistent coordinates.
- [ ] Implement X11 capture in the root coordinate space.
- [ ] Unify Wayland RemoteDesktop and ScreenCast through one portal session.
- [ ] Replace the XDG absolute-pointer workaround and expose capability truth.

## SunCode implementation

- [ ] Add `suncode-computer` and all 17 actions.
- [ ] Add serial batch execution, cancellation, emergency stop, and held-input cleanup.
- [ ] Add image resizing, zoom, frame generations, and context pruning.
- [ ] Add Anthropic Messages and native client-toolset support.
- [ ] Add ComputerManager, policy, approval, audit, persistence redaction, and recovery.
- [ ] Add Rust/C/C# SDK management APIs.
- [ ] Add Avalonia Settings, permission onboarding, approval, and emergency-stop UI.

## Verification

- [ ] Run focused Enigo and Rust tests.
- [ ] Run SDK and Avalonia tests.
- [ ] Run macOS, Windows, X11, and Wayland conformance smoke tests.
- [ ] Run `git diff --check` in each changed repository.

## Closeout

- [ ] Update features and specifications.
- [ ] Record the architectural decision.
- [ ] Update contracts, security, design, packaging, and release documentation.

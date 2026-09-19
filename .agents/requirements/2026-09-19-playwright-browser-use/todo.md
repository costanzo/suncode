# Todo

## Design

- [x] Confirm the requirement.
- [x] Review the architecture.
- [x] Add Browser use to the design-system Settings specimen.
- [ ] Verify all runtime and failure states in both themes.

## Implementation

- [x] Freeze target runtime versions and hashes.
- [x] Add the browser worker protocol and JavaScript worker.
- [x] Add `suncode-browser` and core BrowserManager.
- [x] Add browser model tools and initial fail-closed policy integration.
- [x] Add SDK and Avalonia management surfaces.
- [x] Add packaging integration and document signing order.

## Verification

- [x] Run focused Rust, worker, SDK, design-system, and Avalonia tests.
- [ ] Run offline packaged-runtime smoke tests on all supported targets.
- [ ] Run required broader checks and `git diff --check`.

## Closeout

- [ ] Update features and specifications.
- [x] Record the bundled runtime decision.

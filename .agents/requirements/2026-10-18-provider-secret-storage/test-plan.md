# Test Plan

## Scope

Design-system UI plus Rust storage, SDK, CLI, and Avalonia integration. Installed keychain permission behavior remains platform release verification.

## Unit tests

Rust tests cover secret redaction, storage-mode exclusivity, masking, mode switching, and missing keychain entries. CLI tests cover storage selection and stable credential exit codes.

## Integration and conformance tests

Run Vite build and validate routes, in-page provider state, draft cancellation, masked replacement, and confirmations in the browser.

## Regression checks

Provider endpoint controls and existing Settings navigation remain accessible.

## Manual checks

Both themes; default and constrained sizes; native radio keyboard selection; shared dialog focus and Escape; empty, configured, plaintext, loading, access failures, missing entry, save/remove failure; successful changes survive provider navigation.

## Commands and results

- `npm run build`: passed; bundle-size advisory remains.
- Rust focused tests: passed.
- CLI tests: passed.
- C# and Avalonia builds: passed.
- `git diff --check`: passed.

## Residual risks

Native store permissions, signing, headless Linux Secret Service availability, and installed-platform durability remain release verification.

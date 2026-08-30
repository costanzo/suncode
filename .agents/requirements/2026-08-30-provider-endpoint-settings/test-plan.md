# Test Plan

- Rust SDK accepts and normalizes a valid endpoint, preserves provider metadata and credentials, and updates `list_models` immediately.
- Rust SDK rejects malformed, non-HTTP, hostless, and credential-bearing endpoints without changing the route.
- C ABI exposes the named method and returns the normalized result envelope.
- Avalonia tests cover the new native binding and provider URL save path where practical.
- Design-system production build succeeds and the Settings layout detector reports no unexplained findings.
- `git diff --check` succeeds.

## Result

- Passed `cargo test --workspace`.
- Passed 49 Avalonia tests.
- Passed the design-system production build and Settings layout detector.
- Passed `git diff --check`.

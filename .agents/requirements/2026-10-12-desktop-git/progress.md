# Progress

- Status: Completed; platform-specific verification remains a CI/release follow-up
- Last updated: 2026-10-12

## Completed

- Desktop system Git service, process bounds, physical project paths, NUL-safe status/numstat, structured diffs, and client DTOs.
- Availability-gated controls/footer, guarded shortcut, saved drawer handling, cancellation and stale-result rejection.
- Dedicated Rust/SDK Git API and git2/libgit2 dependencies removed; native ABI advanced to 18.
- Architecture, design specimens, feature/spec/contract documentation, and superseding decision updated.
- Rust operations (35), SDK (34), and C binding (4) tests passed; CLI/C checks passed. Design-system build passed.

## Verification

- Full desktop regression: 316 passed. Final diff inspection and whitespace check passed.
- Design-system Source Control route was inspected in the in-app browser across the unavailable, loading, repository error, binary, and reading-limit states.
- Native macOS Git integration was exercised against real repositories.

## Residual verification

- Windows/Linux Git discovery, process behavior, and native desktop rendering require platform CI or live platform verification.
- The native executable could not be bound by the available macOS UI inspection tool in this environment.

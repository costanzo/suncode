# Changes

## Source

- Move the typed facade out of `agent/crates/core` into `sdks/rust`.
- Move C ABI exports into `sdks/c`.
- Add an ABI-version regression test and C SDK package documentation.
- Expose only the harness primitives required by the facade.
- Split the Rust facade into a minimal crate root, centralized SDK DTOs in `types.rs`, and capability-focused implementation modules under `src/facade/`.

## Contracts and generated artifacts

- Preserve `contracts/agent-sdk/README.md` method and ABI details.

## Tests

- Move facade tests with the facade and keep a focused C ABI test in `sdks/c`.
- Keep the existing 13 Rust facade behavior tests intact after the internal module split.

## Documentation

- Update repository architecture, SDK layout, and Avalonia build documentation.
- Document the Rust SDK source organization in `sdks/rust/README.md`.

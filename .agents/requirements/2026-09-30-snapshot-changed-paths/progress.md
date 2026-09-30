# Progress

- Status: Testing
- Last updated: 2026-09-30

## Completed

- Rust data field, loader, and turn index; C# SDK record parameter; desktop projection; contract and spec updates.
- Focused tests in Rust data, Rust SDK facade, C# SDK models, and desktop projection.

## In progress

- Manual check: reopen a session whose turn wrote files and confirm the Review panel lists them.

## Blocked

- None.

## Log

### 2026-09-30

- Requirement initialized. Chosen approach: expose per-turn checkpointed paths in the Rust snapshot contract.
- Verification: `cargo test --manifest-path agent/Cargo.toml --workspace --all-targets --no-fail-fast` passes except two pre-existing failures unrelated to this change: `suncode-data` `seeded_model_catalog_matches_current_provider_limits_and_capabilities` (catalog id `deepseek-flash` vs expected `deepseek-v4-flash-vision-exp`) and `suncode-mcp` `default_environment_includes_platform_baseline_and_working_directory` (host `LANG` differs from expected `C.UTF-8`). Rust SDK 35/35, C ABI 4/4, desktop 190/190.

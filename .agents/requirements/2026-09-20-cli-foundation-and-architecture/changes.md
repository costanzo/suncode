# Changes

## Source

- Expand the provider adapter constraint to `openai` and `anthropic`.
- Narrowly rebuild the immediately previous provider table while preserving rows.
- Replace the stale C ABI version literal with the authoritative constant.

## Contracts and generated artifacts

- Add `contracts/cli.md` as the approved CLI behavior and ownership contract.
- Update the SQLite schema contract for Anthropic adapter compatibility.
- No generated artifacts.

## Configuration and persistence

- Approve the bounded `SUNCODE_` CLI environment registry.
- Keep provider credentials SQLite-only.
- Preserve the current unversioned schema model and use one narrow compatibility rebuild.

## Tests

- Add previous-provider-constraint preservation coverage.
- Run agent, Rust SDK, C, and Avalonia suites plus formatting and Clippy.

## Documentation

- Remove CLI from deferred scope while marking implementation as not started.
- Update product, architecture, repository guidance, specifications, and decisions.

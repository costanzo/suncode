# Progress

- Status: Complete with contract gaps documented
- Last updated: 2026-09-03

## Completed

- Static comparison of design-system and Avalonia surfaces.
- Confirmation, composer, conversation state, review state, geometry, settings presentation, and tool detail alignment implemented.
- Desktop build, focused tests, design-system build, and diff checks pass.

## In progress

## Completed contract extensions

- Certificate trust source/path is persisted and applied by Rust Provider and WebFetch clients.
- Bash operations emit bounded live `tool.output` events and Avalonia renders a scrolling live-output panel.
- `context.compacted` is projected into `session_call` as an internal Provider Trace exchange.
- `retry_last_turn` is exposed through Rust SDK/C ABI and invoked by the Review panel.

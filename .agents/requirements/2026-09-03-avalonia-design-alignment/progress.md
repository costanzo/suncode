# Progress

- Status: Complete with contract gaps documented
- Last updated: 2026-09-03

## Completed

- Static comparison of design-system and Avalonia surfaces.
- Confirmation, composer, conversation state, review state, geometry, settings presentation, and tool detail alignment implemented.
- Desktop build, focused tests, design-system build, and diff checks pass.

## In progress

## Residual gaps

- Certificate trust source/path is presentation-only until the Rust SDK, persistence, and HTTP trust configuration expose a matching contract.
- Rust currently emits completed tool results rather than live tool-output events, so the tool detail modal cannot stream output while a command is running.
- Provider trace storage exposes model exchanges but not `context.compacted` as a trace call; the conversation surface shows the event, while the dedicated trace row awaits a contract extension.
- “Retry turn” currently reloads the selected session because no SDK retry-last-turn operation exists.

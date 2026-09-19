# Progress

- Status: Complete
- Last updated: 2026-09-19

## Completed

- Reviewed turn, event, Browser, MCP, LSP, Computer Use, blocking adapter, native close, and managed lifetime ownership.
- Added core shutdown state and process-manager drains.
- Added consuming async and blocking SDK shutdown methods.
- Routed native handle close through explicit shutdown.
- Added focused stream closure and data-directory reopen tests.
- Confirmed the implementation compiles.
- Added event-hub close-all coverage and verified desktop behavior remains unchanged.
- Updated architecture, features, specification, contract, SDK documentation, and the decision index.

## In progress

- None.

## Blocked

- Broad SDK/C tests currently fail before shutdown behavior because the current repository baseline reports ABI 13 while its C test expects 10, and its seeded Anthropic provider adapter violates the current SQLite `adapter_type IN ('openai')` schema constraint.

## Log

### 2026-09-19

- Requirement initialized.
- Repository HEAD changed during the delivery to `f076350af0008f00aad8d75ddeb4e96ac715b447` (`pin windows and x11 computer capture`); preserved those changes and adapted shutdown to include Computer Use emergency stop.
- Focused shutdown compilation, event tests, Clippy, formatting, diff checks, and all 107 Avalonia tests passed. Full Rust SDK and C unit execution remained blocked by the unrelated baseline inconsistencies listed above.

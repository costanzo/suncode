# Architecture

## Current state

`list_provider_exchanges` returns a flat list of `session_call` records. `provider_exchange` returns one call without its correlated `session_message` or `session_tool_use` rows.

## Proposed design

Extend the existing read-only provider trace contract. The list result adds ordered turn summaries alongside call summaries. The detail result preserves the call shape and adds normalized correlated messages and tool uses. Avalonia groups call summaries under their owning turns and loads detail only when a call is selected or expanded. It then projects provider-input user/assistant/thinking messages, the assistant response, and correlated tool uses as parallel third-level content nodes without adding another SDK method.

## Boundaries and dependencies

SQLite remains owned by `suncode-db`. Runtime core composes DB DTOs into SDK responses. Avalonia consumes only the C ABI JSON contract. `suncode-llm` reports cache usage when an OpenAI-compatible endpoint supplies it.

## Data and control flow

Opening or refreshing the drawer reads all turn and call summaries for the selected session. Selecting a call reads that call plus correlated messages and tools. Live provider lifecycle events trigger a summary refresh; durable detail always comes from SQLite.

## Security and failure handling

Only normalized provider data is returned. Credentials, authorization headers, and provider-private raw wire payloads remain excluded. Missing provider usage is represented as unavailable rather than zero.

## Compatibility and migration

The existing C ABI methods remain. Their JSON responses gain additive fields. No database migration is required because all relationships already exist in the current fresh schema.

## Risks and rollback

The main risk is a dense inspector becoming hard to scan at small drawer heights. The UI uses a stable tree width, compact metrics, scrollable detail, explicit empty states, and the existing resizable bottom drawer.

## Open questions

- None.

# Changes

## Source

- Added database trace read models and queries for turns, call-correlated messages, and call-correlated tool uses.
- Extended runtime provider trace summary and detail DTOs without adding a new C ABI method.
- Retained provider-reported cache-read and cache-write token usage in normalized OpenAI-compatible call records.
- Rebuilt the Avalonia trace drawer as a turn/call tree with a call inspector for messages, tools, request/response data, timing, and usage metrics.

## Contracts and generated artifacts

- Updated the hand-written runtime SDK contract and shared vectors with additive turn, message, tool-use, and cache usage fields.
- No generated artifacts were introduced.

## Configuration and persistence

- No schema change. Existing `session_turn`, `session_call`, `session_message`, and `session_tool_use` relationships are queried directly.

## Tests

- Added database coverage for turns without calls and call-correlated messages/tools.
- Added OpenAI-compatible cache-token parsing coverage.
- Extended runtime SDK coverage for trace summary and detail responses.

## Documentation

- Updated the runtime and Avalonia feature records and the Phase 1 runtime specification.

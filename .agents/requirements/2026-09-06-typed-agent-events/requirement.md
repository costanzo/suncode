# Requirement

## Background

Agent event emission currently accepts unrelated string event names and anonymous JSON payload values, so Rust does not verify that an event name has the expected payload shape.

## Goals

- Represent known agent event names with a Rust enum.
- Provide documented Rust payload structures for event data.
- Preserve the existing SDK event JSON envelope and event names.

## Non-goals

- Changing the C ABI callback format.
- Generating C# contracts.
- Changing persisted normalized table formats.

## Acceptance criteria

- `Agent::emit` accepts typed event names and payloads.
- Core lifecycle events use concrete payload structures.
- Existing SDK serialization remains compatible.
- Focused Rust tests and workspace checks pass.

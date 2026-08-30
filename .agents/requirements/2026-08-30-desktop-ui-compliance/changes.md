# Changes

## Source

- Added message-owned image attachment submission, capability gating, thumbnails on sent messages, and safe image validation.
- Rebuilt Review hierarchy, compact navigation restoration, session status lamps, Settings provider overview, Markdown styling, focus return, and accessible names/status feedback.
- Added bounded off-thread thumbnail work, image payload limits, message bitmap disposal, and a recycling conversation container.

## Contracts and generated artifacts

- Updated the hand-written SDK and persistence contracts plus executable design specimens; no generated protocol artifacts were added.

## Configuration and persistence

- `session_message` uses `image_ref` content parts to associate an existing `session_image` row without a schema change.

## Tests

- Added multimodal wire-shape and session-state tests; existing Rust and .NET suites cover the broader regressions.

## Documentation

- Added and completed this delivery package, updated durable feature/specification files, and marked the placeholder-only requirement as superseded in part.

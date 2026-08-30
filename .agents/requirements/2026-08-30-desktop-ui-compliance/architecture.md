# Architecture

## Current state

Avalonia owns presentation and transient interaction state. Rust owns persisted session images, messages, turns, provider capabilities, validation, and the SDK facade. The current submit method accepts text only, while session images are separately persisted and restored into the composer.

## Proposed design

The UI continues to upload images through the Rust-owned image service, but holds returned image identifiers as pending composer attachments. Turn submission sends text plus those identifiers. Rust validates the identifiers against the selected session and model capability, associates them with the admitted user message, and returns them in normalized conversation snapshots. Submitted images leave the composer and render above their owning user message.

Review and responsive changes remain presentation projections over existing normalized DTOs. Where a missing projection is needed, Rust exposes it through the existing SDK snapshot/event boundary rather than allowing Avalonia to inspect storage or the project directly.

## Boundaries and dependencies

- `design-system/` remains review-only.
- Avalonia never reads SQLite, project files, Git, or provider APIs directly.
- Rust validates durable attachment ownership and provider capability.
- Contract changes remain hand-written in documentation, Rust, and C# with focused tests.

## Data and control flow

1. Avalonia uploads an image to Rust and receives its session-owned ID and thumbnail.
2. Avalonia submits text plus up to three pending image IDs.
3. Rust validates and admits the turn atomically, persists message attachment ownership, and constructs provider-neutral image content only for capable models.
4. Conversation snapshots return attachment metadata on the user message.
5. Avalonia removes submitted IDs from the composer and renders them on the message.

## Security and failure handling

- Paths and raw bytes do not enter provider requests unless the selected image is validated and the provider capability allows it.
- Credentials and raw native envelopes remain excluded from UI and logs.
- Failed submission preserves pending attachments and explanatory status.
- Oversized, malformed, foreign-session, missing, or unsupported images fail closed.

## Compatibility and migration

Existing session images without a message association remain pending composer references. Existing text-only callers continue through a compatibility wrapper that supplies no attachment IDs. Schema evolution follows the repository's current-schema initialization rules and focused compatibility checks.

## Risks and rollback

- Attachment changes cross persistence and provider boundaries; land them behind focused contract tests before UI behavior changes.
- Conversation virtualization can disturb scroll anchoring; preserve existing latest-message and user-scroll behavior.
- Shared focus and token styles affect every window; verify all top-level surfaces in both themes.

## Open questions

- None.

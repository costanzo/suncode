# Requirement

## Background

Provider API keys currently live as plaintext in SQLite. The user requests a fresh-system design with per-provider system credential storage or plaintext storage, and requires design-system review and confirmation before production development.

## Goals

- Store each protected API key directly in the operating system credential store.
- Default new credentials to protected storage; allow an explicit plaintext choice per provider.
- Show only a saved mask and location, plus accurate access and failure states.
- Keep Rust ownership through a dedicated secrets crate.

## Non-goals

Legacy migration, import, ECC, shared master keys, production changes before UI approval, and extending this delivery to other secret categories.

## Requirements

The Settings provider detail must distinguish saved state from pending edits, support key replacement and storage changes, confirm plaintext Save and key removal, cancel pending changes, retain saved metadata after failure, and avoid automatic fallback. Each platform needs accurate store naming. Review tooling must expose empty, saved, loading, locked, denied, unavailable, missing, save failure, and removal failure states without OS side effects.

## Edge cases

Short keys use a fixed mask. A missing system entry requires replacement input. Keychain access failure must not be reported as an unconfigured provider. A mode change that needs an inaccessible old key requires restored access or replacement input. In-flight credential controls are disabled.

## Acceptance criteria

Interactive review route, consistent themes and constrained geometry, keyboard-operable controls and shared confirmations, build and diff validation. Production work starts only after explicit UI confirmation.

## Open questions

UI approval and production implementation authorization were received. Installed-platform permission checks remain release verification.

# Architecture

## Current state

Rust data operations persist provider keys in SQLite. The LLM resolver returns an optional String and currently suppresses database errors. Production contracts remain unchanged by this design review.

## Proposed design

A future `suncode-secrets` crate under the agent owns platform credential-store access, secret wrappers, references, and typed errors. Data owns plaintext-mode values and non-secret credential metadata. Rust composition owns per-provider policy and resolution. Providers and native clients do not call platform credential stores directly.

## Boundaries and dependencies

This delivery changes only the executable React design specification and its documentation. Rust, SDKs, CLI, Avalonia, database schema, and accepted persistence decisions are deferred until UI approval.

## Data and control flow

The prototype holds each provider's mask and mode in Settings page state. Draft input stays inside the credential component and is cleared after Save, Cancel, or navigation. Fake platform and failure controls sit outside the product specimen. No localStorage or OS secret operations are used.

## Security and failure handling

No global key or key pair. Complete keys belong to the selected OS store or explicit plaintext mode. Failures preserve saved status and never downgrade protection. Production must distinguish absent values from locked, denied, unavailable, and failed operations, and handle partial updates across separate stores before declaring success.

## Compatibility and migration

Fresh system only, as requested. No legacy schema, plaintext import, or migration workflow.

## Risks and rollback

Platform signing, CLI access, system policy, and headless Linux behavior require implementation validation later. The review only simulates those outcomes.

## Open questions

Production API shape and platform dependencies will be finalized after UI approval.

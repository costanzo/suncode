# Architecture

## Current state

Rust owns provider credential metadata and mode changes. The LLM resolver returns `Result<Option<String>, BusinessError>` so access errors propagate to callers.

## Proposed design

The `suncode-secrets` crate under the agent owns platform credential-store access, secret wrappers, references, and typed errors. Data owns plaintext-mode values and non-secret credential metadata. Rust composition owns per-provider policy and resolution. Providers and native clients do not call platform credential stores directly.

## Boundaries and dependencies

The executable React design was approved before implementation. Rust owns platform access through `suncode-secrets`; data owns per-provider metadata and transactional mode changes; SDK, CLI, and Avalonia consume redacted DTOs. The system is fresh-only: incompatible old provider schemas are rejected rather than migrated.

## Data and control flow

The prototype holds each provider's mask and mode in Settings page state. Draft input stays inside the credential component and is cleared after Save, Cancel, or navigation. Fake platform and failure controls sit outside the product specimen. No localStorage or OS secret operations are used.

## Security and failure handling

No global key or key pair. Complete keys belong to the selected OS store or explicit plaintext mode. Failures preserve saved status and never downgrade protection. Production must distinguish absent values from locked, denied, unavailable, and failed operations, and handle partial updates across separate stores before declaring success.

## Compatibility and migration

Fresh system only, as requested. No legacy schema, plaintext import, or migration workflow.

## Risks and rollback

Platform signing, system policy, and headless Linux behavior require installed-platform release validation.

## Open questions

No open design questions remain. Installed-platform permission behavior requires release verification.

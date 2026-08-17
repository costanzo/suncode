# Requirement

## Goals

- Rename the conversation root table from `sessions` to singular `session`.
- Rename `audit_records` and `approval_requests` to singular `audit_record` and `approval_request`.
- Rename `checkpoint_manifests` and `checkpoints` to singular `checkpoint_manifest` and `checkpoint`.
- Replace split settings tables with `configuration` for global, project, and session scopes.
- Preserve global-to-project-to-session precedence and relational owner integrity.
- Require every persisted model provider to select a provider adapter known to `suncode-llm`.
- Use the OpenAI-compatible adapter as the current general-purpose option for custom endpoints.

## Non-goals

- Migrate an existing database.
- Add a second provider wire adapter in this delivery.
- Add provider/model catalog CRUD to the C ABI.

## Acceptance criteria

- The fresh schema has 13 application tables and uses singular names for `session`, `audit_record`, `approval_request`, `checkpoint_manifest`, and `checkpoint`.
- Configuration owner shape, uniqueness, JSON validity, and foreign keys are enforced.
- Custom providers with an unknown adapter are rejected.
- Seeded providers explicitly select `openai` compatibility.

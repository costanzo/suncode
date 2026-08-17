# Requirement

## Background

Project metadata lives in `projects`. Configuration needs one consistent structure across global, project, and session ownership. The first project-specific value is the default model ID.

## Goals

- Keep the existing `projects` table unchanged.
- Use one `configuration` key/value table for global, project, and session scopes.
- Enforce project ownership with a foreign key and cascade cleanup.
- Preserve the existing SDK settings methods while routing all three scopes through `configuration`.
- Use a project's `default_model` setting when a session or turn does not explicitly provide a model.

## Non-goals

- Add project CRUD beyond the existing `projects` behavior.
- Migrate or convert older databases.
- Add tenant, user, or hosted scopes.
- Add a client-specific project-setting API in the C ABI.

## Requirements

- `configuration` stores valid JSON plus `updated_at` and enforces one value per scope owner and key.
- Project and session configuration rows use foreign keys with `ON DELETE CASCADE`.
- Project values override global values, and session values override project values in effective reads.
- `default_model` must be a JSON string containing a non-empty advertised model ID when used for model selection.

## Acceptance criteria

- Fresh databases contain `configuration` and no legacy settings tables.
- All configuration scopes write to `configuration`.
- Effective configuration preserves global/project/session precedence.
- Sessions created without an explicit model use the project's configured default model.
- Database, runtime, formatting, and diff checks pass.

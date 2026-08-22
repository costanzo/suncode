# Requirement

## Background

The Avalonia and Rust logging modules were configured through `SUNCODE_LOG_*` environment variables. Durable application settings already belong to Rust-owned SQLite. The physical project identity table also remained plural while related current tables are singular.

## Goals

- Persist the shared logging policy as global `configuration` rows.
- Apply one policy to separate Avalonia and Rust log files.
- Expose the policy in the Avalonia Settings > Logging page.
- Rename the physical `projects` table to `project`.
- Preserve the existing SDK settings method and Rust-only SQLite ownership.

## Non-goals

- Store log records in SQLite.
- Add a schema migration runner or silently convert an existing database.
- Move the bootstrap data/database location into SQLite.

## Requirements

- Seed `log_level="INFO"`, `log_directory=""`, `log_max_bytes=10485760`, and `log_retention=5` in global configuration.
- Treat an empty log directory as `<data directory>/logs`.
- Accept logging settings only at global scope and validate their JSON types and ranges before persistence.
- Configure Rust after SQLite opens and reconfigure it immediately after a logging setting changes through the SDK.
- Configure Avalonia from the existing settings DTO after runtime startup.
- Let users edit all four values in Settings > Logging and save them through the existing SDK settings method.
- Use `project` in the schema manifest, foreign keys, indexes, and Store queries.

## Edge cases

- File-open failure keeps stderr diagnostics available.
- The logging settings cannot locate their own database, so data/database paths remain bootstrap inputs.
- A database containing the former `projects` table is incompatible under the current fresh-schema-only policy and is not mutated.

## Acceptance criteria

- A fresh database has `project`, not `projects`, and contains typed logging defaults.
- `SUNCODE_LOG_*` is absent from production source and current logging documentation.
- Rust database/runtime tests and the Avalonia build pass.
- Current persistence contracts explain the compatibility boundary.

## Open questions

- A future released-schema migration policy remains a separate architectural decision.

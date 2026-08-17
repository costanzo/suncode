# Architecture

`projects` remains the project identity and lifecycle table. One `configuration` table owns global, project, and session key/value overlays without adding nullable settings columns to domain tables.

```text
configuration
    |- global(key)
    |- project(project_id, key)
    `- session(session_id, key)
```

The database Store keeps the public effective-settings shape (`SettingRecord`) for SDK compatibility. It resolves settings in global, project, session order, with later scopes replacing earlier keys. CHECK constraints enforce the correct owner columns and partial unique indexes enforce per-scope key uniqueness.

Core resolves `default_model` from the project row when creating a session without an explicit model. The agent applies the same lookup for a turn submitted without a model, retaining the existing DeepSeek fallback when no project default is configured.

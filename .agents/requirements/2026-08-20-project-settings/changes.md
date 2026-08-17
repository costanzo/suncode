# Changes

- Added the unified `configuration` schema table and kept `projects` unchanged.
- Routed global, project, and session configuration through the new table while retaining the existing SDK DTO and methods.
- Added project default-model resolution for sessions and turns without an explicit model.
- Added Store and runtime tests for isolation, precedence, validation, and default selection.

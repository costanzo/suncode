# Changes

- Renamed the `sessions` table/schema resource and all foreign keys to `session`.
- Renamed `audit_records` and `approval_requests` table/schema resources to `audit_record` and `approval_request` without changing their structures.
- Renamed `checkpoint_manifests` and `checkpoints` table/schema resources to `checkpoint_manifest` and `checkpoint` without changing their structures.
- Replaced `project_setting` and `setting_records` with `configuration`.
- Changed effective configuration precedence to global, project, session.
- Added provider `adapter_type` to schema, seed data, DTOs, Store validation, and runtime registry construction.
- Updated Avalonia's global configuration call and current contracts.

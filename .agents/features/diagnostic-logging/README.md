# Diagnostic Logging

**Status:** Implemented and focused-tested

The Rust agent writes `agent.log` and the Avalonia client writes a separate `desktop.log`. Logging policy is global SQLite configuration. Environment variables do not set it.

| Key | Default | Rule |
| --- | --- | --- |
| `log_level` | `INFO` | `TRACE`, `DEBUG`, `INFO`, `WARN`, `ERROR`, or `OFF` |
| `log_directory` | `""` | Empty means `<data directory>/logs` |
| `log_max_bytes` | 10 MiB | Integer of at least 1024 |
| `log_retention` | 5 | Rotated backups, 0-100 |

- All four keys are global-only and validated on write. A successful SDK write reconfigures the Rust logger right away.
- Before SQLite is read, both loggers write to a default file. Every record is flushed. If logging itself fails, the logger falls back to stderr and the original failure is kept.
- Rotation happens when the next line would exceed `log_max_bytes`. Backups are named `agent.log.1` through `agent.log.<retention>`. A retention of 0 deletes the active file instead.
- Each line holds a timestamp, level, process and thread IDs, component, and a bounded single-line message.
- Error records include the operation name, safe session/turn correlation, error code, retryability, and provider request ID when one exists. API keys, authorization headers, prompts, model output, tool arguments and results, file contents, raw provider payloads, proxy credentials, and screenshot bytes are never logged.

The SDK boundaries where errors are logged are listed in [`agent-phase-1/`](../agent-phase-1/README.md). The logging contract is in [`contracts/persistence.md`](../../../contracts/persistence.md).

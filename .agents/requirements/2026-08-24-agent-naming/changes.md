# Changes

- Renamed the production Rust workspace, crate, native library, C ABI symbols, desktop binding namespace, current contract paths, and current feature/specification paths to `agent`.
- Renamed health/error identifiers and local lock/log/database surfaces to `agent` terminology.
- Bumped the native ABI from 2 to 3.
- Removed database fallback and legacy Keychain import behavior; the new project uses only `agent.sqlite3` and current SQLite credentials.
- Updated current product documentation and preserved the prior harness migration only as historical context.

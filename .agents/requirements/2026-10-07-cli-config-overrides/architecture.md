# Architecture

## Proposed design

`suncode-config` owns typed parsing and precedence. `suncode-data` remains the SQLite owner and supplies a small persisted-settings source to the resolver. SDK composition resolves one effective runtime configuration, injects it into the agent, and the agent snapshots turn limits at admission.

## Boundaries

- CLI selects `EnvironmentSource::Process`.
- Avalonia and default native hosts select `EnvironmentSource::Disabled`.
- Provider credential overlays are in-memory only.
- Agent core consumes typed configuration and does not parse environment variables.

## Security

Hard-deny command checks remain unconditional. Environment tool ceilings cannot expand specialist or host capability ceilings. Credentials are redacted from diagnostics and never copied into child process environments.

# Requirement

## Background

Language servers are currently configured as global free-form local processes. The generic command, argument, language ID, root marker, initialization JSON, environment, and timeout form requires users to understand both LSP and each server's launch contract. Global arguments also cannot safely express per-project state such as the unique jdtls data directory.

## Goals

- Present a SunCode-owned catalog of supported language-server types.
- Store enablement and effective configuration per project.
- Give each supported type a focused configuration contract and Settings page.
- Add jdtls as the first supported type.
- Give every project an independent jdtls process, index, and data directory.

## Non-goals

- Bundling or downloading jdtls.
- Adding completion, rename, formatting, code actions, or arbitrary LSP methods.
- Implementing the Rust, persistence, SDK, or Avalonia changes before the design-system review is accepted.

## Requirements

- The catalog is product-owned and users cannot create, rename, or delete built-in types.
- The jdtls configuration supports project enablement, executable and JDK detection or override, a managed or custom data directory, root markers, environment entries, and startup/request timeouts.
- Raw launch arguments, language IDs, and initialization JSON are built-in behavior rather than primary user configuration.
- Runtime status and failures remain scoped to the selected project and provide actionable recovery.
- The design-system specimen is reviewed before production implementation begins.

## Edge cases

- jdtls or a compatible JDK is not detected.
- No configured root marker matches the opened project.
- A custom data directory is already locked by another jdtls process.
- The executable, JDK, or root marker changes while the server is running.
- Existing global free-form definitions require an explicit compatibility decision.

## Acceptance criteria

- The design-system Settings page shows the project-scoped catalog and the jdtls detail flow.
- Managed data paths visibly resolve to a project-unique location.
- Enable, disable, detection, save/restart, indexing, ready, failed, and retry states are specified.
- Desktop and constrained-width layouts remain usable in both themes.

## Open questions

- Resolved: legacy free-form definitions remain persistence-compatible but are not exposed as a `Custom` type or launched.
- Resolved: executable and JDK detection supplies per-project defaults; no global user-configurable defaults are introduced.

# Requirement

## Background

SunCode currently supports Java semantic analysis through the project-scoped built-in jdtls catalog entry. JavaScript, JSX, TypeScript, and TSX projects need the same five bounded read-only semantic tools without returning to global free-form process configuration. The selected server is `@vtsls/language-server`, launched locally as vtsls.

## Goals

- Add vtsls as the second SunCode-owned language-server catalog type.
- Configure and run vtsls independently for each opened project.
- Detect the vtsls executable and a compatible Node.js runtime.
- Give users a focused TypeScript SDK policy without exposing raw initialization JSON or VS Code's full TypeScript settings surface.
- Complete design-system review before production implementation.

## Non-goals

- Bundling or installing Node.js, vtsls, TypeScript, package managers, or project dependencies.
- Exposing completion, rename, formatting, code actions, or vtsls commands to the model.
- Supporting arbitrary tsserver plugins or the full `typescript.*` and `javascript.*` preference catalog in this delivery.
- Adding a vtsls data directory; vtsls has no jdtls-style project workspace lock requirement.
- Implementing Rust, SDK, persistence, or Avalonia changes before design approval.

## Requirements

- The catalog identifies vtsls as JavaScript and TypeScript semantic analysis and reports host installation detection independently from jdtls.
- Project configuration includes enablement, vtsls executable, Node.js executable, root markers, TypeScript SDK policy, optional custom SDK directory, tsserver memory, write-only environment changes, and startup/request timeouts.
- TypeScript SDK policy offers workspace-preferred, bundled, and custom modes. Workspace-preferred maps to vtsls automatic workspace TypeScript selection and falls back to the bundled version.
- SunCode owns `--stdio`, language IDs, launch construction, initialization behavior, validation, and defaults.
- Node.js must be compatible with the installed vtsls package; the initial production design validates Node 18 or newer.
- Save and Reset remain disabled until the normalized draft differs from the loaded project configuration.
- Runtime failure displays an actionable reason and Retry without discarding unsaved configuration.

## Edge cases

- vtsls is absent or resolves to a broken npm shim.
- Node.js is absent, older than the package requirement, or differs from the runtime embedded in the executable shim.
- Workspace TypeScript is absent; workspace-preferred falls back to the vtsls bundled TypeScript version.
- A custom SDK directory does not contain `tsserverlibrary.js`.
- No root marker matches the opened project.
- Large monorepos exceed the configured tsserver memory limit.

## Acceptance criteria

- The design-system catalog shows both jdtls and vtsls.
- A direct review route opens the vtsls detail page.
- The detail page specifies default, custom SDK, advanced, dirty, failed, retry, indexing, and ready states.
- Light, dark, and constrained-width layouts remain usable.
- Production Rust, SDK binding, and Avalonia behavior match the approved design.

## Resolved decisions

- Production accepts the `vtsls` npm bin or a direct `vtsls.js` script and resolves common npm layouts before launch.
- Project configuration exposes a detected default or custom Node.js runtime and validates Node.js 18 or newer.

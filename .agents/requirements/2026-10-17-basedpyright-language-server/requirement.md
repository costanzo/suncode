# Requirement

## Background

SunCode supports Java through jdtls and JavaScript/TypeScript through vtsls. Python projects need the same five bounded read-only semantic tools through the SunCode-owned project catalog. The selected server is basedpyright, launched locally with `basedpyright-langserver --stdio`.

## Goals

- Add basedpyright as the proposed third built-in language-server type.
- Design a focused project configuration surface before production implementation.
- Detect the basedpyright executable and a project Python interpreter.
- Expose useful analysis choices without recreating basedpyright's full configuration file.

## Non-goals

- Bundling or installing Python, basedpyright, virtual environments, or project dependencies.
- Replacing committed `pyproject.toml` or `pyrightconfig.json` configuration.
- Exposing completion, rename, formatting, code actions, inlay hints, or arbitrary diagnostic overrides to the model.
- Implementing Rust, SDK, persistence, or Avalonia behavior before design approval.

## Requirements

- The catalog identifies basedpyright as Python semantic analysis with independent installation detection.
- Project configuration includes enablement, executable, target Python interpreter, root markers, diagnostic scope, type-checking mode, optional configuration location, write-only environment changes, and timeouts.
- SunCode owns `--stdio`, the Python language id, launch construction, settings mapping, validation, and defaults.
- The UI explains that committed basedpyright configuration takes precedence over language-server analysis settings.
- Save and Reset remain disabled until the normalized draft differs from the loaded project configuration.
- Runtime failure retains configuration and exposes an actionable Retry.

## Edge cases

- basedpyright is installed inside a project virtual environment rather than on the host PATH.
- The selected interpreter disappears or points outside the project.
- No Python root marker matches the opened project.
- A monorepo keeps its Python configuration below the opened project root.
- Workspace diagnostics are expensive for a large repository.

## Acceptance criteria

- The design-system catalog shows jdtls, vtsls, and basedpyright.
- A direct route opens the basedpyright detail page.
- The page covers default, custom configuration location, dirty, disabled, indexing, ready, failed, and retry behavior.
- Light, dark, and constrained-width layouts remain usable.
- Production implementation remains pending explicit design approval.

## Open questions

- Whether production interpreter detection should prefer `.venv`, the executable environment, or the host PATH when several interpreters exist.
- Whether type-checking mode should be omitted from runtime settings when a project configuration file is present.

# Architecture

## Current state

The core registry assembles eight model-facing schema modules. The agent separately maps model and historical aliases into the audited operations dispatcher.

## Proposed design

Assemble only six schemas in `tools::definitions`: `read`, `glob`, `grep`, `write`, `edit`, and `bash`. Remove the unused `apply_patch` and `process` schema modules while retaining their agent translation and operations dispatch paths.

## Boundaries and dependencies

The LLM request surface changes in runtime core. The operations package, policy mapping, persistence, SDK facade, and desktop client remain unchanged.

## Data and control flow

New turns receive the six schemas. Historical tool calls continue through the existing name mapping, validation, authorization, and dispatcher.

## Security and failure handling

The change narrows model authority without bypassing policy. Internal compatibility operations remain approval-gated and audited.

## Compatibility and migration

No database or protocol migration is needed. Persisted calls retain their original names and normalized results.

## Risks and rollback

The main risk is prompt/schema disagreement. An exact registry test and removal of the process-specific host instruction cover that boundary.

## Open questions

- None.

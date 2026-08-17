# Requirement

## Background

Repository and interface copy currently define SunCode by its device-centric deployment model. SunCode is instead a general-purpose coding agent. The current embedded desktop architecture remains an implementation fact, not the product category.

## Goals

- Define SunCode consistently as a general-purpose coding agent.
- Remove the retired positioning from interface copy, product knowledge, contributor guidance, requirements, and decisions.
- Preserve accurate Phase 1 facts about the embedded desktop runtime, SQLite persistence, and current deferred scope.

## Non-goals

- Add hosted execution, tenancy, remote identity, or new client surfaces.
- Change runtime behavior, protocols, persistence, or authority boundaries.
- Remove precise uses of `local` that describe files, processes, runtime state, or storage.

## Requirements

- Product descriptions must lead with broad coding utility rather than deployment topology.
- The About window must describe SunCode as a general-purpose coding agent.
- Architecture documents must distinguish product identity from the current desktop deployment.
- Historical records must use current terminology without changing the technical decisions they document.

## Edge cases

- Current Phase 1 limitations must not be presented as permanent product identity.
- Technical terms such as local folder and local runtime must remain when they are operationally meaningful.

## Acceptance criteria

- A repository-wide case-insensitive scan finds no retired positioning phrase.
- Product, architecture, contributor, decision, and UI copy describe the same product category.
- The Avalonia application builds and focused tests pass.
- `git diff --check` passes.

## Open questions

- None.

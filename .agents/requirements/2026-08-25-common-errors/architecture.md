# Architecture

## Proposed design

`suncode-common::BusinessError` owns the shared `code`, `message`, and JSON `details` contract. Provider retry metadata remains available as typed fields and is mirrored into `details` for the existing SDK/event shape.

## Boundaries and dependencies

LLM, data, core, and tools return `BusinessError`. Native database/provider/OS errors are converted at the crate that owns the adapter. `suncode-common` has no database or provider implementation dependencies; Diesel errors are converted inside `suncode-data`.

## Compatibility and migration

This is a new project boundary. The public error payload shape is preserved without a compatibility layer.

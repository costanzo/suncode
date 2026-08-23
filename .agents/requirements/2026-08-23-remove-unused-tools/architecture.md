# Architecture

## Current state

The provider receives seven schemas, while core still translates retired tool names and operations still dispatches patch, extra filesystem actions, capability wrappers, asynchronous process management, recovery helpers, and protocol-era aliases without production callers.

## Proposed design

Use one canonical name per model tool and one canonical `tool/*` operation method. Retain only operations reached by the agent or typed SDK. Keep `process::run` as the private implementation behind `tool/bash`; remove its structured and asynchronous public operation entries.

## Boundaries and dependencies

Core continues to own model schemas, translation, validation, policy, and approval. The operations crate continues to own machine effects. The typed SDK retains Git inspection and checkpoint restore. No client gains a generic operation API.

## Data and control flow

`registered model name` -> exact core mapping -> policy -> canonical `tool/*` method -> audited operation module.

## Security and failure handling

Unknown and retired names fail closed before execution. Removing compatibility aliases narrows authority. Existing write checkpoints, process cancellation flags, output bounds, network constraints, and audit behavior remain intact.

## Compatibility and migration

This intentionally removes development-era execution compatibility. Persisted tool records are data and remain readable, but replay of retired calls is unsupported. No SQLite or C ABI shape changes.

## Risks and rollback

The risk is deleting an operation reached through an indirect string. Repository-wide call-site scans and focused tests cover every retained dispatcher method. Reverting the source deletion restores the old development-only surface without data conversion.

## Open questions

- None.

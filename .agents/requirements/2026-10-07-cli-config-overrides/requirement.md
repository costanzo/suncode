# Requirement

## Background

Terminal-Bench and other non-interactive CLI hosts need per-process policy, limits, and provider credentials without changing the Avalonia process behavior or persisting benchmark secrets.

## Goals

- Make `suncode-config` the single resolver for environment, persisted, and default configuration.
- Let CLI hosts opt into `SUNCODE_*` overrides while Avalonia and other hosts ignore them.
- Support non-interactive policy, tool ceilings, timeouts, tool budgets, isolated data paths, and in-memory provider credentials.

## Non-goals

- Changing the permanent hard-deny command rules.
- Making the Rust process boundary an OS sandbox.
- Adding Harbor integration in this delivery.

## Requirements

- Bootstrap paths are resolved before SQLite opens and cannot fall back to SQLite.
- Runtime settings resolve environment over persisted settings over defaults.
- Environment credentials are never persisted or exposed to child processes, logs, traces, or JSON output.
- Effective turn configuration is snapshotted at turn admission.
- Tool allowlists constrain both advertised tools and execution.

## Acceptance criteria

- CLI can run with isolated `SUNCODE_DATA_DIRECTORY` and CLI-only environment overrides.
- Avalonia's SDK host ignores the same process environment variables.
- Focused config, provider, policy, timeout, and tool-ceiling tests pass.

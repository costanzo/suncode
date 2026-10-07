# Requirement

## Background

Harbor can run installed custom agents inside Terminal-Bench sandboxes. SunCode's native CLI is the production agent host, so benchmark integration should wrap that binary rather than add a second agent loop.

## Goals

- Provide a Harbor `BaseInstalledAgent` adapter for the SunCode CLI.
- Pass task instructions through stdin and preserve CLI JSONL output.
- Write a valid ATIF-v1.7 trajectory in Harbor's agent log directory.

## Non-goals

- Adding Harbor or Python to SunCode production dependencies.
- Building or distributing the SunCode binary inside this repository's adapter.
- Implementing interactive chat, approval continuation, or trajectory loading.

## Acceptance criteria

- A Harbor custom-agent entry point is available at `benchmarks.harbor.suncode_agent:SunCodeAgent`.
- The adapter supports provider-qualified Harbor model names by passing the SunCode model ID.
- The adapter writes `trajectory.json` and the pure converter has focused fixture coverage.

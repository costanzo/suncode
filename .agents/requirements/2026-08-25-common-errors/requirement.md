# Requirement

## Background

The Rust crates defined separate provider, persistence, agent, SDK, and registration error types even though they cross the same module boundaries and expose the same code/message/details contract.

## Goals

- Provide one shared `suncode-common` Rust crate.
- Use `BusinessError` as the common business-level error across core, LLM, data, and tools APIs.
- Preserve existing error codes, messages, structured details, retryability, and provider request identifiers.

## Non-goals

- Exposing third-party Diesel, HTTP, Git, or OS error types to clients.
- Changing the SDK wire format or adding migration compatibility.

## Acceptance criteria

- No custom business error type remains in the Rust crates besides `suncode-common::BusinessError`.
- Workspace tests, formatting, and clippy pass.

# Test Plan

## Scope

Remote pairing/configuration, encryption, reconnect, request dispatch delegation, event forwarding, and shutdown.

## Unit tests

- Remote payload and URL validation.
- Controller status and configuration transitions.
- Agent host dispatch and event subscription delegation.

## Regression checks

- Rust SDK and C binding compilation.
- Existing remote-control tests.
- `git diff --check`.

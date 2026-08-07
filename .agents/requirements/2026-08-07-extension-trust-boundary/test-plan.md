# Test Plan

## Scope

Future extension boundary only; no executable extension behavior exists in Phase 1.

## Unit tests

Manifest validation, capability declarations, identity binding, and fail-closed policy outcomes.

## Integration and conformance tests

Cross-process handshake, cancellation, bounded output, artifact references, sandbox enforcement, and secret-handle delivery.

## Regression checks

Ensure Phase 1 cannot start project-declared extension code and cannot route extension requests around policy.

## Manual checks

Inspect platform-reported sandbox guarantees and quarantine behavior after child failure.

## Commands and results

Not run; this is documentation only.

## Residual risks

Platform sandbox primitives and supported OS matrix are not selected.

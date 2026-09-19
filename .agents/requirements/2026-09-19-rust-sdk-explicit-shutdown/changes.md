# Changes

## Source

- Add core shutdown state, cancellation, manager draining, bounded turn quiescence, and event-hub closure.
- Add consuming async and blocking Rust SDK shutdown methods.
- Route native handle close through blocking shutdown.
- Reject new turn and continuation admission after shutdown begins.

## Contracts and generated artifacts

- Document explicit shutdown ordering and drop fallback behavior.
- Preserve the current C close symbol, signature, and ABI version.

## Configuration and persistence

- No schema or configuration changes.

## Tests

- Add same-data-directory reopen and stream-closure tests.
- Run focused compilation and available regression suites.

## Documentation

- Add this delivery package and update architecture, SDK contract, feature/specification records, and the decision index.

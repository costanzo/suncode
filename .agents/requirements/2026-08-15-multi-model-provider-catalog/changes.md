# Changes

## Source

- Added a per-request wire-model argument to the provider contract and model routes.
- Registered two static models for each built-in provider.

## Contracts and generated artifacts

- The provider-neutral completion contract now carries a per-request wire-model argument.

## Configuration and persistence

- Provider endpoint/model environment overrides were removed.
- No SQLite schema change is planned; sessions continue to store stable model IDs.

## Tests

- Added catalog, route, API, agent, and Qt verification.

## Documentation

- Initialized this multi-model provider delivery record and updated durable project knowledge.

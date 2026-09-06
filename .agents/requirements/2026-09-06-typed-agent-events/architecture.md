# Architecture

`agent::events` owns `EventType`, `EventPayload`, and concrete payload DTOs. Each `EventPayload` variant determines its `EventType`; `Agent::emit` accepts only that combined typed event and converts it to the existing data projection boundary. The data crate and native SDK continue using the stable string/JSON representation because that is the persistence and cross-language serialization boundary.

There is no arbitrary core `Json` payload or unknown event-name variant. Adding a new agent event requires adding its named payload structure, enum variant, and stable dotted name in one place.

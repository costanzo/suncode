# Architecture

Presentation-only changes remain in Avalonia views, controls, and view models. Existing Rust SDK contracts are reused; no new client-side database or provider access is introduced. Certificate-source UI is currently documented as a design requirement but has no persisted SDK setting, so the client exposes the state locally until a contract is approved.

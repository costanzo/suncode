# Architecture

## Current state

Avalonia XAML and C# view models compose the desktop UI and adapt the Rust SDK C ABI into presentation state. The Rust runtime is built as a dynamic library beside the managed executable.

## Proposed design

`apps/desktop-avalonia/` is a .NET 10 Avalonia application. XAML owns layout, semantic styles, and native controls. C# view models own transient presentation state. `RuntimeSdk` is a narrow P/Invoke adapter over ABI version 1 and returns parsed SDK DTOs. `DesktopViewModel` projects DTOs and ordered events into observable UI collections.

The Rust crate produces `rlib`, `staticlib`, and `cdylib`. The desktop build invokes Cargo and copies the platform dynamic library into the managed output directory.

## Boundaries and dependencies

- Avalonia depends on Avalonia and the native SDK contract only.
- C# never opens SQLite, contacts model providers, invokes Git, or reads project files.
- Rust remains the only owner of durable state and machine-affecting operations.
- DTOs remain hand-written and are parsed with `System.Text.Json`.

## Data and control flow

1. The application opens one reference-counted native runtime handle shared by every hub, project, and settings view model in the process.
2. Each view model loads health, models, settings, credentials, and projects asynchronously while retaining independent presentation and subscription state.
3. Each project opens in its own window; selecting a project loads sessions and runtime-owned Git status without changing another project window.
4. Selecting a session loads its snapshot, usage, checkpoints, then starts an ordered native subscription.
5. Native callbacks are copied to managed strings and queued to Avalonia's UI dispatcher.
6. Mutating commands call named SDK methods and refresh affected projections.

## Security and failure handling

Native strings are copied and freed through the SDK allocator. API keys are sent only to `set_credential`, never retained in view-model state. Errors are parsed from SDK envelopes and shown as bounded status text. Subscription and runtime handles use deterministic disposal.

## Compatibility and migration

The C ABI version and method payloads are unchanged. Avalonia is the sole production client and consumes the existing SDK surface.

## Risks and rollback

Native library discovery and platform packaging are the main integration risks. The project-local build copies the library beside the executable, while P/Invoke keeps a stable logical library name.

## Open questions

- Signing, notarization, installer generation, and release-channel packaging remain release-engineering work outside this source migration.

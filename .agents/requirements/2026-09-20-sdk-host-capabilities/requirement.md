# Requirement

## Background

The approved CLI must not initialize or advertise Browser Use or Computer Use in its first release, even when the shared database contains settings enabled by the desktop host. SDK startup currently derives those capabilities only from persisted settings and local runtime availability, so a terminal host cannot express a stricter capability ceiling without mutating shared configuration.

## Goals

- Add typed Rust SDK startup options containing host capability ceilings.
- Preserve existing desktop, C, C#, and default Rust startup behavior.
- Allow the future CLI to disable Browser and Computer Use without changing persisted settings.
- Prevent disabled host capabilities from initializing backends, advertising model tools, requesting OS permissions, or accepting management operations.
- Keep capability ceilings separate from policy authority and persisted user preferences.

## Non-goals

- Implementing the CLI executable.
- Adding policy grants or non-interactive policy profiles.
- Changing the C ABI or managed API.
- Adding per-session capability mutation.
- Disabling MCP, LSP, built-in coding tools, or providers.

## Requirements

- Public `SdkOpenOptions` contains typed `SdkHostCapabilities`.
- Default capabilities retain Browser and Computer Use support.
- Async and blocking Rust facades accept options with and without trusted provider composition.
- C continues using default startup and requires no ABI change.
- Core receives an immutable host capability ceiling at composition time.
- A disabled Browser capability exposes no browser tools, starts no worker, and rejects management operations with `browser_host_unavailable`.
- A disabled Computer capability exposes no native computer toolset, initializes no Enigo backend, requests no OS permission, and rejects management operations with `computer_host_unavailable`.
- Runtime info may report the persisted desired enablement but must report host unavailability and no active backend/runtime.
- Updating settings through either named or generic SDK methods cannot bypass the host ceiling or mutate the persisted enablement.
- Explicit shutdown remains safe for a capability-disabled host.

## Edge cases

- Persisted Browser and Computer settings are both true when the CLI opens.
- A host reads runtime info for an unavailable capability.
- A host attempts to disable a persisted capability it does not own.
- Default C/Avalonia startup must remain fully capable.
- Trusted provider composition and capability options are used together.

## Acceptance criteria

- Focused SDK tests prove settings remain unchanged and no Computer backend is available.
- Browser runtime reports unsupported for the disabled host.
- Named management operations fail with stable host-unavailable errors.
- Rust SDK, core, C binding, and Avalonia tests pass.
- Formatting, applicable Clippy, locked builds, and `git diff --check` pass.

## Open questions

- Additional future host ceilings should be added only when a real client cannot safely present the capability.

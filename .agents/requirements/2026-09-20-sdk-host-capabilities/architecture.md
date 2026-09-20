# Architecture

## Current state

`AsyncAgentSdk::open_default` composes core from persisted settings. Browser manager reads global Browser enablement, and SDK startup installs an Enigo Computer backend when Computer Use is enabled. A host cannot declare that it lacks UX or packaging for either capability.

## Proposed design

Add public `SdkOpenOptions { host_capabilities: SdkHostCapabilities }`. Both capability fields default to true so existing Rust, C, C#, and Avalonia startup remains unchanged. Add async and blocking `open_with_options` and `open_with_options_and_providers` methods; compatibility methods delegate with default options.

The SDK maps its public options to core-owned `AgentHostCapabilities` during composition and retains the SDK value in facade state for host-facing management checks. Browser and Computer managers receive immutable availability booleans.

When unavailable, managers retain the persisted desired setting only for diagnostics. They expose no model tools and perform no external initialization. Browser runtime information reports `Unsupported`; Computer runtime information reports no backend, unsupported permissions, user control, and a safe host-unavailable message. SDK mutation and permission methods reject before persistence, so the shared desktop preference is not changed.

## Boundaries and dependencies

- SDK owns the public startup option contract.
- Core owns enforcement at tool catalog, execution, runtime, and backend boundaries.
- Persisted settings remain user preferences, not host capability declarations.
- Policy still evaluates only calls that survive the capability ceiling; a capability ceiling cannot grant authority.
- C and C# remain on default capabilities.

## Data and control flow

1. Host constructs `SdkOpenOptions`.
2. SDK loads persisted settings and passes immutable host capabilities into core composition.
3. Computer backend initialization occurs only when persisted enablement and host availability are both true.
4. Browser/Computer catalogs check host availability before persisted enablement or runtime probing.
5. Host-facing mutations check the SDK ceiling before writing settings.
6. Shutdown drains only resources that were actually initialized.

## Security and failure handling

The ceiling is fail-closed and immutable for the SDK lifetime. It does not relax policy, approval, project scope, audit, or undo behavior. Stable errors distinguish host unavailability from user-disabled or missing-runtime states. Disabled Computer capability avoids OS permission queries and input backend initialization.

## Compatibility and migration

The change is additive to Rust. Default startup behavior is unchanged. There is no persistence migration, C ABI bump, managed DTO change, or desktop UI change.

## Risks and rollback

Risks are persisting changes before rejecting, reporting persisted enablement as effective availability, and enforcing only at the SDK while model tools bypass it. SDK pre-write checks plus core manager enforcement address these. Rollback removes the options and restores persisted-setting-only composition without data changes.

## Open questions

- None for Browser and Computer Use.

# Requirement

## Background

Rust `watch_session` atomically returns a snapshot and typed stream, but C/C#/Avalonia still load a snapshot and establish a callback subscription separately. Starting callback delivery immediately inside the native watch call would race with managed snapshot application and could let live events mutate an incomplete or stale UI projection.

## Goals

- Expose atomic watch establishment through the C and C# bindings.
- Return the snapshot together with a dormant subscription handle.
- Begin callback delivery only after the managed host explicitly starts the handle.
- Migrate primary desktop session loading and resync to the atomic watch.
- Preserve current loading, error, stale-selection, and session-entry UX.

## Non-goals

- Visual or copy changes.
- Migrating read-only child-session inspection in this delivery.
- Replacing the existing compatibility snapshot and subscribe functions.
- Adding replay or cross-process subscriptions.

## Requirements

- The C ABI adds `watch_session` and `subscription_start`, advances its exact version, and retains existing functions.
- `watch_session` returns a snapshot JSON value and an opaque dormant subscription handle.
- Events queue while dormant and callbacks cannot run before `subscription_start` succeeds.
- Start is single-use and failure is explicit; close works before or after start.
- C# exposes a typed disposable watch with `Snapshot` and `Start`.
- Desktop selection applies the atomic snapshot and auxiliary data, installs the watch as the current subscription, then starts callbacks.
- Every stale, failed, or cancelled load disposes its dormant watch.
- Resync follows the same atomic load path.

## Edge cases

- Session selection changes while watch creation or projection is running.
- Auxiliary data loading fails after the dormant watch exists.
- More than the bounded event capacity arrives before callback start.
- Start is invoked twice or after disposal.
- A callback disposes its own subscription.
- Native thread creation fails.

## Acceptance criteria

- No callback can reach the desktop before the matching snapshot is applied and the subscription becomes current.
- Stale session loads cannot retain handles or deliver events.
- Lag while dormant triggers the existing `resync.required` flow after start.
- ABI, C#, desktop, and focused lifecycle tests pass.

## Open questions

- Child-session inspection remains snapshot-only and may adopt an equivalent read-only watch later.

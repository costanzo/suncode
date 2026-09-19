# Architecture

## Current state

The Rust agent owns providers, the agent loop, tools, policy, approvals, SQLite, SDK contracts, and machine operations. User images can reach vision-capable providers, but successful tool results are reconstructed primarily as text. The Claude provider currently uses an OpenAI-compatible chat-completions adapter and cannot declare or process Anthropic client toolsets.

Enigo provides cross-platform pointer and keyboard injection. Its current main branch also contains Linux XDG RemoteDesktop support, restore tokens, Windows physical-pixel DPI fixes, and an internal portal screenshot request used only to discover display dimensions. It has no public cross-platform capture API.

Browser Use is a separate in-progress capability and is outside this delivery.

## Proposed design

```text
Avalonia Computer Use Settings
  -> C# SDK -> C ABI -> Rust SDK facade
    -> configuration and ComputerManager
      -> policy / approval / audit / cancellation
      -> suncode-computer serialized actor
        -> pinned Enigo revision
          |- display and capture backend
          `- pointer and keyboard backend

Claude Messages adapter
  -> computer_toolset_20260801
  -> ordered client-toolset calls
  -> ComputerManager
  -> text/image tool results
  -> next Claude Messages request
```

The first provider integration uses Anthropic Messages directly. Provider-neutral contracts distinguish function tools from client toolsets and retain toolset identity on calls and results.

## Boundaries and dependencies

- `enigo` owns OS-facing capture, display geometry, permissions, pointer, keyboard, and coordinate metadata. It contains no provider, policy, database, session, or SunCode semantics.
- `suncode-computer` owns the 17 provider-independent desktop action types, validation, screenshot resizing, zoom, coordinate transforms, serial execution, held-input cleanup, and a mock backend.
- Agent core owns enablement, session lease, policy, approvals, audit, cancellation, screenshot lifetime, recovery, and provider-result orchestration.
- `suncode-llm` owns Anthropic wire shapes and provider-neutral function/client-toolset contracts. It cannot perform machine actions.
- The SDK exposes management and user-control methods. It never exposes model-authored actions as direct client APIs.
- Avalonia owns presentation, permission guidance, approval UX, emergency stop, and transient Settings interaction state.
- Browser Use and Computer Use share no runtime, profile, tool definitions, or control lease.

## Data and control flow

### Capture and coordinates

Every full-display capture creates a frame generation containing display identity, source dimensions, model dimensions, input bounds, and scale transforms. Screenshot resizing preserves aspect ratio. Model coordinates are resolved only against the generation that produced the image. Display or scale changes retire the generation.

`zoom` crops a region from a full-resolution capture and scales the crop for provider limits. It does not change the full-display coordinate frame used by later actions.

### Action batch

Core identifies client-toolset calls by `(toolset_name, member_name)`, evaluates them in response order, and delegates authorized actions to one project/session-independent desktop lease owned by `ComputerManager`. The executor stops at the first failure, releases transient modifiers, and returns success, failure, and skipped results for the complete batch.

### Control ownership

Only one of `agent` or `user` owns the desktop-control lease. User takeover prevents further model actions and retires the current frame. Returning control requires a new screenshot before coordinates can execute. Emergency stop cancels the active lease, releases held input, and requires explicit restart.

### Provider context

Image-bearing tool results are transient provider content. Durable tool-use rows retain redacted metadata and outcome only. Older screenshots are pruned in bounded batches while the most recent frames remain available for the active loop.

## Security and failure handling

- The real desktop is not sandboxed. System permissions and OS integrity boundaries remain authoritative.
- Computer Use is interactive-only and approval-gated for every input action initially.
- On-screen instructions never grant authority.
- Coordinates, regions, repeat counts, key chords, durations, image dimensions, and encoded sizes are bounded before execution.
- Cancellation, backend errors, provider errors, permission revocation, display changes, and process shutdown release held input.
- Unknown completion is visible and never replayed.
- Locked or unavailable displays fail closed.
- Screenshot contents, typed sensitive text, restore tokens, and raw provider payloads are redacted from diagnostics.

## Compatibility and migration

- Additive global Computer Use configuration is stored in the existing `configuration` table.
- Add model capability metadata for native Computer Use support.
- Convert the built-in Claude provider to an Anthropic adapter only when its stored endpoint and adapter still match the built-in defaults. Preserve custom OpenAI-compatible Claude gateways without Computer Use support.
- Advance the hand-written C ABI for additive Computer Use management methods.
- Existing sessions and ordinary tool history remain readable.
- Browser Use settings and state are not migrated or reused.

## Risks and rollback

- OS capture APIs and permission behavior differ substantially by platform.
- Coordinate mismatch can cause unintended actions; frame generation and conformance testing are release blockers.
- Wayland portal implementations vary by compositor.
- Native Claude toolset support adds a second provider wire protocol.
- Real-desktop actions cannot be rolled back by filesystem checkpoints.

Rollback disables Computer Use globally, removes the toolset from subsequent provider calls, cancels the active lease, releases held input, and deletes temporary screenshots. It does not undo actions already completed in external applications.

## Open questions

- Whether the Enigo capture API will be proposed upstream after the first stable SunCode integration.
- Whether stable Wayland support ships with the first platform set or remains explicitly experimental until GNOME and KDE pass the same conformance suite.

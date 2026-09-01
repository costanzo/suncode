# Architecture

## Current state

`design-system/src/projects/desktop/workspace/WorkspacePrimitives.jsx` owns the conversation specimen behavior, including the compact composer, attachment preview modal, and tool detail modal. `design-system/src/projects/desktop/workspace/conversation/index.jsx` enumerates the standalone conversation states. `design-system/src/styles/review.css` owns the review-surface styling.

## Proposed design

Extend the conversation specimen with three state-driven additions:

- an expanded-composer modal that shares the same draft state as the compact composer and exposes a larger textarea plus character count
- a live tool-output modal variant that adds a streaming monospace output pane for running commands
- a thinking-state indicator that replaces the generic three-dot activity marker with animated text

Update the conversation route so both overlays are represented as standalone review states. Update `DESIGN.md` so the behavior is part of the durable design authority rather than an implementation-only detail.

## Boundaries and dependencies

- Only `design-system/` and design documentation are in scope.
- Reuse the existing universal `Modal` component and shared icon system.
- Do not change Avalonia, Rust, or contracts.

## Data and control flow

- Conversation state continues to live locally in the React specimen.
- The expanded composer modal reads and writes the same `message` state as the compact composer.
- Running-tool modal content is driven by specimen tool-call data, with timed line reveal to demonstrate streaming output.
- The thinking indicator is driven by a dedicated conversation state branch and CSS keyframe animation.

## Security and failure handling

This change does not affect real credentials, process execution, provider traffic, or persistence. It is a design-only simulation.

## Compatibility and migration

Existing conversation states remain intact. The new overlays extend the same specimen and styling model without changing route structure.

## Risks and rollback

The main risk is visual drift between the specimen and the design authority. Updating `DESIGN.md` in the same change keeps them aligned. Rollback is limited to the design-system files and requirement package.

## Open questions

- None.

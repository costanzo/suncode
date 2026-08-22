# Requirement

## Background

The desktop chat currently either shows every assistant model-call message as an independent response or collapses them destructively into only the final response. Users need live visibility into assistant progress and tool activity without permanently cluttering completed conversation turns.

## Goals

- Present each turn as one conversation round.
- While a turn runs, show assistant progress and tool activity in occurrence order.
- When a turn becomes terminal, collapse its process timeline without deleting it.
- Place an expand/collapse control before the retained process region and final assistant response.
- When expanded, render retained process items below the control and before the final assistant response.
- Keep copy available only for the final assistant response.
- Restore the same timeline after session reload or resync.

## Non-goals

- Changing provider context construction or persisted runtime semantics.
- Storing client-only expansion state across application restarts.
- Showing thinking content in the primary chat timeline.

## Requirements

- User messages remain ordinary chat bubbles.
- Assistant intermediate messages and tool activity are ordered within their owning turn.
- Process items have no copy action.
- Active turns show process items expanded and have no collapse control until a final visible assistant response exists.
- Completed, failed, cancelled, and interrupted turns default to collapsed process items.
- The final visible assistant message remains visible and copyable in both states.
- The Rust SDK snapshot exposes normalized correlation data required to rebuild the timeline; Avalonia does not access SQLite.

## Edge cases

- Tool-call-only assistant messages do not render as empty process rows.
- Tool state updates replace the matching tool row instead of adding duplicates.
- Duplicate live message IDs remain idempotent.
- A terminal turn without a visible assistant response keeps its available process timeline visible.
- Resync and session switching replace the complete timeline atomically.

## Acceptance criteria

- A running multi-call turn displays assistant/tool activity in order.
- Completion collapses process activity and leaves the final assistant response visible.
- Expanding reveals all retained process activity without copy buttons.
- Reloading the session produces the same turn structure.
- Focused Rust and Avalonia tests pass.

## Open questions

- None.

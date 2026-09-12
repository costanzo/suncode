# Requirement

## Background

Each turn can contain many tool calls and tool output. Rendering every call and user message inline makes the conversation difficult to scan.

## Goals

- Add a bottom drawer named **Tool activity** beside Source control and Provider trace.
- Organize activity as Turn → tool call, with a selectable detail pane.
- Keep the conversation chronological by showing submitted user messages and assistant responses while retaining lightweight turn boundaries.
- Show the active tool inline and let users jump directly to its drawer detail.

## Non-goals

- No new persistence model or provider boundary.
- No change to the Rust event contract in the design phase.

## Requirements

1. Tool activity is mutually exclusive with Source control and Provider trace in the bottom drawer.
2. The left tree has two levels: turn rows and tool-call rows.
3. The right detail pane shows status, timing, request, live output, result, error, and approval context as available.
4. The active turn is expanded by default; the active tool is selected by default.
5. Live output is visibly streaming and follows the tail until the user scrolls away.
6. Conversation renders submitted user messages, assistant messages, and lightweight turn separators; historical tool calls remain in Tool activity.
7. Hovering a turn separator reveals the first N characters of its user message in a tooltip/popover, with ellipsis when truncated.
8. Conversation renders at most one compact active-tool row. Clicking it opens Tool activity and selects the corresponding turn/tool.
9. The standalone Tool activity page represents exactly three primary states: no turns, one active turn with its tool list, and one completed turn with all tools completed. Tool-level errors and approvals remain detail variations rather than extra primary page states.
10. Tool rows are compact single-line entries. Completed calls show only their name; unfinished calls show a pulsing status dot on the right; failed calls show their name in the danger color.

## Edge cases

- A turn has no tool calls.
- A tool completes while its detail is selected.
- Live output arrives after the user has scrolled upward.
- The current session is reloaded or receives a resync.
- At compact width, the drawer becomes a stacked list/detail layout while preserving selection.

## Acceptance criteria

- Reviewers can navigate to a standalone Tool activity page and inspect all required states.
- Workspace overview shows the drawer and the inline active-tool jump behavior.
- Conversation hover preview is visible and bounded to a readable width.
- Design-system build passes with no new console errors.

## Open questions

- N is a presentation constant; initial design uses 72 characters and can become a product token during implementation if needed.

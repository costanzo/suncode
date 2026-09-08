# Requirement

## Background

The Workspace title currently continues to show the selected session name after Explorer replaces Conversation with a read-only file. Users also lack a short path back to recently viewed files and sessions.

## Goals

- Make the centered Workspace title identify the content currently visible in the center region.
- Let the title open a compact history of recently viewed files and sessions.
- Establish and review the interaction in the design system before production work begins.

## Non-goals

- Editing files or changing the read-only editor contract.
- Searching, pinning, removing, or manually ordering recent content.
- Changing project switching, session persistence, or file-read authority.
- Modifying the Avalonia production client before design approval.

## Requirements

1. Conversation displays its current session title in the centered Workspace title control.
2. Editor displays its current file name in the same control.
3. The control includes a trailing down chevron and opens a dropdown when activated.
4. The dropdown interleaves recently viewed sessions and files, most recent first, with no more than 20 unique items.
5. Rows distinguish sessions from files through icon, type text, and restrained semantic color while retaining useful secondary metadata without exposing unrestricted filesystem information.
6. Selecting a recent item switches the central content, moves the item to the front, and closes the menu.
7. The current file or session remains in the internal recent history but is omitted from the dropdown; visible count and empty state use only the remaining items.

## Edge cases

- Long titles and paths elide without changing title-bar geometry.
- More than 20 unique items retains only the latest 20.
- When only the current item has been viewed, the dropdown shows its empty state.
- After switching content, the previous current item becomes visible in the dropdown.
- An empty history has an explicit non-actionable empty state.
- Escape and outside click close the menu.

## Acceptance criteria

- The design-system Workspace specimen demonstrates session and file titles plus the mixed recent-content menu.
- Selecting a recent file displays Editor and selecting a recent session displays Conversation.
- Current-item exclusion, hover/focus, long-name, bounded-list, empty, light, dark, and constrained-width behavior are specified.
- No production Avalonia or SDK source changes are made before user approval.

## Open questions

- None. The approved production behavior keeps recent content for the current project-window lifetime and does not persist it.

# Architecture

## Current state

Workspace owns transient selection of either a session Conversation or a read-only file Editor, but its centered title is a static session label.

## Proposed design

Workspace derives one current-content item from the active central view. A title-bar switcher renders that item and a bounded recent-content projection that excludes the current identity. Selecting a row invokes the same central session/file transition used by Sessions and Explorer.

## Boundaries and dependencies

- The design-system React implementation is review tooling only.
- A future Avalonia implementation will own presentation and transient interaction state.
- Session and file identities must come through existing client/SDK contracts; the switcher does not access SQLite or the filesystem.

## Data and control flow

1. Session or Explorer selection creates a typed recent-content item.
2. Workspace makes it current, deduplicates it into the front of the internal list, and truncates the list to 20.
3. Opening the title displays a derived option list containing only non-current entries; its count and empty state use that projection.
4. Selecting a recent row routes to Conversation or Editor and closes the menu.

## Security and failure handling

File rows use the same bounded display path already exposed by Explorer and Editor. The control never reads a file or mutates session/file state by itself.

## Compatibility and migration

The centered static title becomes an interactive control without changing the leading project switcher or central content surfaces.

## Risks and rollback

The main risks are ambiguous file/session identity and title-bar crowding. Typed items, stable identity deduplication, secondary metadata, elision, and bounded menu geometry address those risks. Rollback restores the static title without affecting content data.

## Open questions

- None. Recent content is transient ViewModel state for the current project window and is cleared when that ViewModel changes projects or is disposed.

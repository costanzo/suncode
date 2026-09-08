# Architecture

## Current state

Avalonia Workspace selects either Conversation or the read-only Editor as its central content surface. Explorer file nodes request bounded content through the Rust SDK facade; the desktop never reads project files directly.

## Implemented design

The desktop ViewModel owns the transient selected-file state. Explorer raises a file-selection event through the existing view boundary. Workspace chooses one central content mode at a time: selected session Conversation or selected file Editor. The Editor view owns only presentation and document viewport state; it does not read SQLite or providers directly.

## Boundaries and dependencies

- Avalonia views own layout and input routing.
- The desktop ViewModel requests bounded file reads through the existing Rust SDK/client boundary.
- AvaloniaEdit renders the document in read-only mode.
- TextMate supplies syntax highlighting for the detected language.
- Dependency files use the same read path and remain subject to existing dependency read authority.

## Data and control flow

1. Explorer file click emits the stable file identity/path.
2. ViewModel requests the bounded file contents and language metadata.
3. Workspace enters loading, then ready/empty/error editor state.
4. Session selection clears the selected file and reloads the selected conversation snapshot.

## Security and failure handling

File content must be obtained through the existing audited read boundary; the client must not bypass Rust authority with direct filesystem access. Reads are bounded and errors are shown without leaking credentials or unrestricted paths beyond the existing project/dependency display rules.

## Compatibility and migration

The change is additive to Workspace navigation. Conversation remains the default central surface and existing session, review, drawer, and undo behavior is unchanged.

## Risks and rollback

The main risks are large-file memory use, unsupported language grammars, and accidental edit affordances. Keep file size/read bounds explicit, fall back to plain text highlighting, and set all editor input paths read-only. Rollback removes the editor route and selected-file mode without changing session persistence.

## Resolved questions

- Production uses `Avalonia.AvaloniaEdit` 12.0.0 and `AvaloniaEdit.TextMate` 12.0.0. The latter supplies `TextMateSharp` and `TextMateSharp.Grammars` 2.0.3 transitively.

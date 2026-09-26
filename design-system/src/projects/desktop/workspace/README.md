# Workspace Review Surface

The Workspace review surface is organized by ownership:

- `shell/` contains the project window composition and title-bar-level controls.
- `panels/` contains one entrypoint per review panel.
- `shared/` contains low-level controls reused by more than one panel.
- `data/` owns fixture data grouped by review responsibility: sessions, projects, editor, explorer, conversation, and review/source-control. `fixtures.js` remains a compatibility barrel for older consumers.
- `styles/` is the stylesheet boundary; `index.css` is the only stylesheet imported by the application entrypoint and aggregates `base.css`, shell, panel, drawer, animation, and responsive rules. Base rules are grouped into guide, activity, conversation support, review/diff, and state override files. Review rules are grouped into panel, turn changes, approval/question, and state files. Shell rules are grouped into chrome, content switcher, project switcher, layout, status bar, and focused-shell files. Conversation rules are grouped into surface, messages, composer, and context files; child-session rules are grouped into list, detail, timeline, and state files. Drawer rules are grouped into `drawer-shell.css`, `source-control.css`, `provider-trace.css`, `tool-activity.css`, and their layout/modal/state companions. The thin aggregate files preserve source order for these focused files.

`WorkspacePrimitives.jsx` remains a compatibility facade for existing consumers. New routes should import from the focused `shell/`, `panels/`, or `shared/` entrypoints. The facade contains no panel implementation; it only re-exports the focused modules during the migration.

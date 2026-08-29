# Architecture

## Current state

The React design browser owns review-only desktop specimens. The Avalonia application owns production presentation and transient interaction state, consumes DTOs and events through the embedded Rust SDK facade, and already contains the feature modules represented by the prototypes.

## Proposed design

The prototypes remain a hand-reviewed visual contract rather than a runtime dependency or code-generation source. Avalonia maps that contract in three layers:

1. `App.axaml` owns theme resources and shared control styles.
2. ProjectHub, ProjectWorkspace, SettingsWindow, and AboutWindow own window-level composition and custom chrome.
3. Feature views own panel-specific layout, state presentation, and responsive behavior.

The implementation proceeds from shared resources outward so later panel work does not duplicate token or control-state fixes.

## Prototype-to-production mapping

| Prototype | Avalonia owner |
| --- | --- |
| ProjectHub | `Views/Projects/ProjectHub.axaml` |
| Workspace shell | `Views/Projects/ProjectWorkspace.axaml` |
| Sessions | `Views/Projects/ProjectSidebar.axaml` |
| Explorer | `Views/Projects/ProjectExplorer.axaml` |
| Conversation | `Views/Chat/ChatArea.axaml`, `Views/Chat/ChatInput.axaml` |
| Review | `Views/Review/AgentSidebar.axaml` |
| Source control | `Views/Review/GitViewer.axaml` |
| Provider trace | `Views/Review/ProviderTraceViewer.axaml` |
| Settings | `Views/Settings/SettingsWindow.axaml` |

## Boundaries and dependencies

- `design-system/` remains isolated review tooling and is not referenced by the desktop project.
- Avalonia continues to depend only on .NET/Avalonia libraries and the native SDK contract.
- Rust remains authoritative for providers, the agent loop, policy, SQLite, credentials, operations, approvals, recovery, and undo.
- Existing ViewModel properties remain the source of production state; prototype sample data is never copied into production logic.

## Data and control flow

No data-flow change is required. User input continues through Avalonia event handlers and ViewModels to the SDK facade. SDK snapshots and live events continue to project into Avalonia view state. This delivery changes resource lookup, visual tree composition, and presentation only.

## Security and failure handling

The visual update must not expose credentials, prompt or response contents, raw authorization headers, or hidden native envelopes. Existing warning, approval, disabled, loading, error, and retry states remain visible and must retain their semantic meaning in both themes.

## Compatibility and migration

No stored-data or protocol migration is required. Resource keys should remain stable where practical so views can be upgraded incrementally. Existing event names and bindings are preserved throughout the visual pass.

## Risks and rollback

- Shared resource changes can affect every window; build and manual theme checks follow the token stage before panel work continues.
- Custom-chrome changes can affect drag and resize hit regions; event-bearing elements and resize overlays remain in place.
- Dense layouts can clip localized or long content; constrained-width and long-label checks are included in each relevant stage.
- Each stage is kept as a coherent diff so a visual regression can be reverted without touching SDK behavior.

## Open questions

- None.


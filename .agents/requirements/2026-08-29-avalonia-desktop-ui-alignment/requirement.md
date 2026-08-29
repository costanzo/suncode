# Requirement

## Background

The desktop prototypes under `design-system/src/projects/desktop/` now define the approved ProjectHub, project workspace, settings, and workspace-panel presentation. The production Avalonia client already implements the corresponding workflows, but its visual tokens, window chrome, density, and several panel layouts have drifted from those prototypes.

## Goals

- Align the production Avalonia desktop UI with the approved desktop prototypes.
- Preserve all implemented project, session, conversation, review, source-control, provider-trace, settings, approval, question, checkpoint, and undo behavior.
- Establish shared Avalonia visual tokens and shell geometry before refining individual surfaces.
- Keep each implementation stage independently buildable and reviewable.
- Support both dark and light themes and the existing responsive desktop behavior.

## Non-goals

- Add React or the design-system browser to the production runtime.
- Generate Avalonia controls or contracts from the prototype source.
- Change the Rust SDK facade, persistence, provider, operation, approval, recovery, or undo boundaries.
- Add deferred CLI, TUI, Web, mobile, IDE-plugin, hosted, or Electron surfaces.
- Redesign product workflows that are not represented by the approved prototypes.

## Requirements

- Treat `design-system/src/projects/desktop/` and its shared token CSS as the visual and structural reference.
- Keep `apps/desktop-avalonia/` as the hand-implemented production UI.
- Map shared canvas, surface, text, border, accent, semantic-state, radius, control-height, UI-font, and monospace-font tokens into Avalonia resources.
- Use the prototype's neutral primary-action palette; reserve warning and danger colors for semantic states.
- Align custom desktop frames to a 36 px title bar, 14 px outer radius, and compact traffic-light geometry while retaining native drag, resize, minimize, maximize, and close behavior.
- Align ProjectHub toolbar, recent-project rows, and empty state without changing project-opening behavior.
- Align the workspace as a conversation-first shell with optional left navigation, optional right review, mutually exclusive bottom drawers, narrow gutters, and a compact status bar.
- Align Sessions, Explorer, Conversation, Review, Source control, and Provider trace against their corresponding prototype states.
- Align Settings navigation, content rows, warnings, credentials, models, and responsive behavior without changing configuration ownership.
- Preserve keyboard navigation, visible focus, readable truncation, scroll access, and empty/loading/error/disabled states.
- Provide Composer-only image attachments with local thumbnail and full-size preview, removal, and a maximum of three images; attachment bytes are intentionally excluded from the current text-only submit-turn request.
- Do not introduce direct SQLite, provider, project-file, Git, or operation access from Avalonia.

## Edge cases

- Long project, session, model, branch, file, and provider names must truncate or wrap without forcing horizontal window overflow.
- At constrained widths, secondary panes and drawers must collapse according to existing responsive rules while the conversation remains usable.
- Empty projects, no sessions, no turns, no Git changes, unavailable providers, and disconnected-agent states must remain understandable.
- Dialog overlays must remain above custom chrome and preserve their current cancellation and confirmation behavior.
- Light and dark themes must use the same layout and semantic hierarchy.

## Acceptance criteria

- Shared Avalonia resources match the approved design tokens for both themes.
- ProjectHub, Workspace, Settings, and About use consistent custom frame geometry and title-bar density.
- Each desktop prototype module has a documented Avalonia owner and is implemented in the staged plan.
- Existing bindings, event handlers, SDK calls, and Rust ownership boundaries remain intact unless a separately approved behavior requirement requires a change.
- `dotnet build apps/desktop-avalonia/SunCode.Desktop.csproj` succeeds.
- `dotnet test apps/desktop-avalonia/tests/SunCode.Desktop.Tests.csproj` succeeds.
- `git diff --check` succeeds.
- Manual review covers ProjectHub, Workspace, Settings, dialogs, dark/light themes, and constrained window widths.

## Open questions

- None for the approved implementation sequence. Any prototype ambiguity will be resolved by preserving current production behavior and applying the smallest visual translation.

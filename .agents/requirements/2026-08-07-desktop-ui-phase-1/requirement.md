# Requirement

## Background

Suncode needs a desktop client. It must use Qt and consume the client API defined by the TypeScript agent runtime. No product UI is implemented yet.

Per `PRODUCT.md`, the desktop application is the **second** committed surface. The CLI/TUI ships first, because it is the cheapest path to daily self-use and the only surface that makes non-interactive and CI execution possible. This package should therefore be scheduled after the CLI proves the client API, and it can expect that API to already exist and to have been exercised by a real client.

There is no generated client SDK. This surface hand-writes its transport adapter against the client-runtime contract document and its shared test vectors.

The desired experience is similar in spirit to the Codex desktop application: session-oriented, conversation-first, restrained, and optimized for supervising coding work. The first version should feel like a native workbench rather than an IDE clone, terminal wrapper, dashboard, or marketing surface.

## Users and primary jobs

The primary user is a software developer working in one or more local repositories. The Phase 1 desktop client must make these jobs efficient:

1. Open or switch to a project.
2. Start, find, resume, rename, pin, archive, or delete a session.
3. Give an agent instructions with relevant files or text attached as context.
4. Follow reasoning summaries, messages, tool activity, progress, and errors as they stream.
5. Approve or deny security-sensitive operations with a clearly stated scope.
6. Inspect changed files and diffs without losing the conversation.
7. Undo the filesystem changes a turn made, and understand what undo does not cover.
8. Interrupt an active turn and recover from connection or runtime failures.
9. Review what the agent was authorized to do and what it did.
10. Configure models, permissions, appearance, and runtime connection settings through the UI.

## Goals

- Deliver a clear, low-distraction desktop workflow centered on sessions and projects.
- Make agent state, scope, permissions, and code changes continuously understandable.
- Keep the conversation usable while a contextual file, diff, or activity view is open.
- Support keyboard-heavy use without excluding pointer or assistive-technology users.
- Define explicit loading, empty, error, offline, reconnecting, and recovery states.
- Keep the presentation layer replaceable and independent of runtime, persistence, and OS-operation internals.

## Non-goals

- Implementing the Qt application in this requirement delivery.
- Reproducing OpenAI branding, proprietary assets, exact copy, or undocumented behavior.
- Building a general-purpose source editor, debugger, Git client, or terminal emulator.
- Letting the client read SQLite, call model providers, or invoke the Rust core directly.
- Defining the complete client-runtime contract document.
- Supporting web, mobile, or IDE-plugin layouts. The CLI/TUI is a separate package that ships first.
- Supporting parallel subagents within one session.
- Generating client types or an SDK from the contract.
- Implementing cloud project provisioning, authentication infrastructure, packaging, updates, or code signing.

## Experience principles

1. **Session first.** The active session and its status are the center of the application.
2. **Project always visible.** Users must be able to tell which repository and runtime a session affects before sending or approving work.
3. **Progressive disclosure.** Messages are primary; verbose tool details, raw output, and metadata are collapsed until requested.
4. **Review beside conversation.** Files and diffs open in context without replacing or obscuring the session.
5. **Explicit authority.** Approval prompts state the requested action, affected resource, risk, and grant duration before presenting a decision.
6. **Stable geometry.** Streaming content and state changes must not cause the composer, navigation, or primary actions to jump unexpectedly.
7. **Native restraint.** Use platform conventions, thin separators, compact controls, and a limited accent color. Avoid decorative cards, oversized headings, and promotional layout.

## Information architecture

**`project-window-amendment.md` in this directory is normative for window and navigation structure and governs where it differs from this section.** The application uses a project-per-window model: each window is permanently bound to one project, and the sidebar lists only that project's sessions.

Within a project window, the destinations are:

- **Sessions:** the default work surface and this project's session history.
- **Settings:** provider configuration, model defaults, permissions and policy profiles, appearance, and notifications.
- **Diagnostics:** runtime connection health, versions, capabilities, and redacted client diagnostics. May live inside Settings in Phase 1.

Projects are not a destination inside a window. They are opened through the launcher, the application menu, OS file integration, or a recent-projects action, each of which opens or focuses that project's own window.

A session belongs to exactly one project. A window cannot retarget itself to another project.

## Primary window layout

### Wide layout

At a comfortable desktop width, the window has three regions:

```text
+----------------------+------------------------------------+--------------------------+
| Navigation           | Session                               | Context inspector        |
|                      |                                    |                          |
| New session             | Header: project / session / status  | Changes / Files /        |
| Search               |                                    | Activity                  |
|                      | Conversation timeline              |                          |
| Project groups     |                                    | Selected diff, file,      |
| and recent sessions     |                                    | command, or metadata      |
|                      |                                    |                          |
| Settings / account   | Composer and turn controls         |                          |
+----------------------+------------------------------------+--------------------------+
```

- The navigation sidebar is collapsible and resizable, with a target width of 240-320 logical pixels.
- The session pane is the primary region and must retain at least 520 logical pixels when the inspector is visible.
- The inspector is optional, resizable, and uses a target width of 360-560 logical pixels.
- Dividers, not nested cards, separate major regions.
- Pane sizes and collapsed state are restored per device, subject to the current window size.

### Medium layout

When three panes cannot fit without harming readability:

- The navigation sidebar collapses to a rail or drawer.
- The session pane remains visible.
- The inspector opens as a mutually exclusive side panel or replaces the session content temporarily, with an obvious Back action.

### Narrow layout

At the supported minimum window width:

- Only one primary surface is visible at a time.
- Navigation and inspector content open as drawers or dedicated views.
- The composer remains reachable and its controls wrap or collapse into menus without clipping.
- No horizontal page scrolling is permitted; code and diffs may scroll within their own viewport.

Exact breakpoints are implementation tokens and must be validated with the selected Qt technology. The application should target a minimum usable window of 720 x 560 logical pixels and a comfortable layout at 1280 x 800.

## Functional requirements

### Application startup and runtime connection

- The client must show one of: starting, connecting, ready, reconnecting, offline, incompatible, or diagnostic failure.
- Startup must not display an interactive composer until the runtime is ready for mutations.
- A local runtime connection must use the approved per-user authenticated discovery and handoff mechanism once that contract exists.
- A visible status affordance must identify local versus remote execution and the active project.
- Connection failures must preserve locally held draft text and offer retry plus diagnostics.
- Reconnection must resume ordered session events from the last applied sequence. If replay is unavailable, the client must fetch and replace with a fresh session snapshot and disclose that recovery occurred.

### Project management

- Users can open a local folder using the native folder picker.
- Recent projects show name, disambiguated path, availability, and runtime target.
- Users can switch projects without destroying session drafts in other projects.
- Missing, moved, unauthorized, or unavailable projects have explicit states and recovery actions.
- The UI must never imply that a remote project maps to the client device filesystem.
- The current project identity appears in the session header and in every approval prompt.

### Session navigation and lifecycle

- The sidebar provides a prominent New session action and search.
- Sessions are grouped by project and ordered by recent activity by default.
- Each row shows title, concise status, and relative last activity without becoming a multi-line card.
- Users can resume, rename, pin/unpin, archive/unarchive, and delete sessions.
- Delete requires confirmation and must explain whether deletion is permanent according to the runtime contract.
- Search covers session titles and message text when the runtime supports it; unsupported scopes must not be simulated locally.
- An active session remains visibly active if the user navigates elsewhere.
- Phase 1 supports multiple independent sessions, but each session has one sequential agent turn at a time.

### Session header

The session header must show:

- Session title.
- Project identity.
- Runtime target when it is not unambiguously local.
- Current state: idle, queued, working, awaiting approval, interrupted, failed, or complete.
- Compact actions for opening the inspector and the session menu.

The header must not become a second toolbar full of rarely used actions. Rename, archive, delete, and session metadata belong in the session menu or inspector.

### Conversation timeline

- User and assistant messages are visually distinct without rendering every message as a heavy card.
- Assistant output streams incrementally while preserving selection, scrolling, and accessibility announcements.
- Markdown, fenced code, tables, lists, and file references render safely.
- File references are actionable and open the relevant file or diff in the inspector when available.
- Tool calls appear as compact activity rows with an icon, plain-language label, status, elapsed time, and expandable details.
- Command output and logs are collapsed by default, use monospace text, preserve meaningful whitespace, and have bounded height with internal scrolling.
- Long messages support a per-message copy action and selection without exposing controls permanently.
- The timeline distinguishes cancelled, interrupted, failed, and recovered turns.
- The UI must not expose hidden model reasoning. It may show runtime-provided summaries and explicit activity events.
- When the user is near the bottom, streaming follows new content. When the user has scrolled upward, the viewport remains stable and a New activity affordance appears.

### Composer

- The composer is anchored to the bottom of the session pane and grows from one line to a bounded multi-line height.
- Enter submits and Shift+Enter inserts a newline by default; this behavior must be configurable for accessibility and international input methods.
- The primary send control changes to an interrupt control while the active turn can be cancelled.
- Users can attach project files or paste text as context through runtime-supported references.
- Attachments are visible before submission and can be removed individually.
- Model and interaction-mode selectors are compact controls near the composer, shown only when the runtime advertises those capabilities.
- Submission is disabled with a useful reason when the runtime is unavailable, no project is selected, required configuration is missing, or another sequential turn is active.
- Drafts survive session navigation, recoverable reconnects, and accidental window close according to a later persistence policy.
- Secrets detected by the runtime must not be echoed into attachment previews or logs.

### Turn control and status

- A submitted turn immediately receives a deterministic pending state; duplicate submission must be prevented.
- Users can interrupt a running turn without discarding already received messages and activity.
- Retrying a failed turn must clearly identify whether it resends the original input or starts a new turn.
- Status text must supplement, not rely only on, spinners or color.
- Background sessions show their working or approval-needed state in the sidebar and may raise a native notification according to user settings.

### Approvals

- Approval requests appear inline at the causal point in the timeline and may also surface through a non-blocking application notification.
- Each request states the action, canonical affected resource or safe summary, why it is needed, relevant argument restrictions, project, and requested grant lifetime.
- Available decisions are explicit: deny and the runtime-supported grant scopes such as allow once, allow for session, or allow for project.
- Persistent or broader grants require stronger visual emphasis and confirmation than one-operation grants.
- Closing or navigating away must not implicitly approve.
- Duplicate, expired, superseded, or already-resolved prompts become non-interactive audit entries.
- The UI forwards decisions to the runtime; it does not create, broaden, or enforce grants.

### Context inspector

The inspector provides tabs only when relevant:

- **Changes:** changed-file list, counts, and a unified or side-by-side diff viewer.
- **Files:** read-only file preview and file references relevant to the session.
- **Activity:** chronological tool operations, progress, and bounded output.

Requirements for the inspector:

- Opening it must not reset conversation scroll or composer state.
- Diff rows have stable line-number gutters, additions/deletions use color plus symbols, and long lines scroll within the viewer.
- Binary, deleted, renamed, too-large, unavailable, and permission-denied files have explicit presentations.
- The UI must distinguish an agent-reported proposed change from a change confirmed by runtime project state.
- All diffs shown here derive from runtime-provided core snapshots. The client never reads the filesystem to compute one.

### Undo

Reversibility is a product commitment, so the client must expose it rather than defer it.

- A turn that modified files offers undo, anchored to that turn in the conversation.
- Before undoing, the UI states which files will be restored and confirms the action.
- Undo wording must scope the promise honestly: it restores files the agent changed, and does not reverse external side effects such as a pushed commit, a published package, or a sent request.
- A file modified outside the agent since capture is presented as a conflict for the user to resolve, never silently overwritten.
- Expired or unavailable checkpoints are visible as such before a user relies on undo.
- Undo is itself an audited operation and appears in history like any other change.

### Authority review

- Approval prompts state the requested action, affected resource, risk, and grant duration before offering a decision.
- A session must be able to show what was authorized, by whom or by which policy profile, and what resulted — the visible surface of the audit stream.
- Active grants are viewable and revocable without opening a settings dialog.
- Where a policy profile pre-authorized an operation, the UI attributes it to the profile rather than implying a person approved it.

### Settings and onboarding

- First run guides the user only through requirements that block productive use: runtime connection, provider/account setup when needed, and opening a project.
- Configuration is edited through UI forms backed by runtime APIs; the user is never instructed to edit a configuration file.
- Secret fields never reveal stored values after persistence. Replacement and removal are explicit actions.
- Settings are grouped into General, Models and providers, Permissions, Appearance, Notifications, and Runtime.
- Project-specific settings clearly identify their narrower scope and inherit global defaults visibly.
- Unsaved settings changes are either applied immediately with feedback or use explicit Apply/Cancel semantics consistently within a screen.

### Diagnostics and logs

- Diagnostics show safe connection state, client/runtime/core versions, negotiated capabilities, and redacted identifiers useful for support.
- The UI can open or export redacted UI-layer diagnostics through an approved API or platform action.
- Raw credentials, secrets, sensitive file contents, and full authentication tokens must never appear.
- Runtime and OS logs remain separately owned; the client may request redacted diagnostic views but does not read their files directly.

### Window and platform integration

- Use standard window behaviors for minimize, maximize, restore, resizing, focus, and close on each supported desktop OS.
- Remember non-sensitive window geometry, pane sizes, theme, and last selected top-level destination per device.
- Warn before closing only when an unsent draft, unresolved local form edit, or other client-local data would actually be lost. A running session continues or stops according to explicit runtime lifecycle policy, not an accidental UI assumption.
- Native notifications are optional, configurable, and limited to actionable state such as approval needed, session completed, or session failed.
- External editor integration is optional for Phase 1 and must use explicit user configuration.

## Visual requirements

- The visual system uses neutral surfaces, readable contrast, one restrained accent, and semantic success/warning/error colors.
- Support light, dark, and follow-system themes from the first implemented release.
- Use a native UI sans-serif stack and a platform-appropriate monospace stack for code and output.
- Use a 4-pixel base spacing system with compact and comfortable density tokens.
- Controls and repeated items use at most an 8-pixel corner radius unless a platform convention requires otherwise.
- Use icons for familiar actions, with text labels where ambiguity or consequence requires them. Icon-only actions require accessible names and tooltips.
- Avoid gradients, decorative illustrations, card-within-card composition, oversized typography, and motion without meaning.
- Focus, hover, selected, disabled, destructive, and keyboard states must be visually distinct.
- Animation should be brief and functional; reduced-motion settings disable nonessential transitions.

## Accessibility and localization

- Target WCAG 2.2 AA for applicable desktop UI behavior and contrast.
- All functionality must be reachable by keyboard, with a logical focus order and visible focus indicator.
- Define shortcuts for new session, session search, send, interrupt, sidebar toggle, inspector toggle, and settings; avoid overriding standard text-editing shortcuts.
- Screen readers must receive names, roles, state changes, streamed-message completion, and approval urgency without announcing every streaming token.
- Controls must remain usable at 200% UI scaling and with increased system text size.
- Color is never the only carrier of state or diff meaning.
- Layout must tolerate text expansion and left-to-right localization. Right-to-left support may be deferred but must not be structurally precluded.
- Input handling must respect IME composition and must never submit an unfinished composition.

## Performance requirements

- The main window should become responsive within 2 seconds after process start on the reference development hardware, excluding runtime startup delays; runtime readiness is shown independently.
- User input, pane resizing, and scrolling should target 60 frames per second under normal session history sizes.
- The UI must virtualize or incrementally render long session lists, long conversations, large activity histories, and large diffs.
- Switching between recently viewed sessions should show cached presentation state immediately and reconcile with runtime state without displaying stale content as current.
- Streaming updates should be batched enough to avoid layout churn while remaining perceptibly live.
- Concrete reference hardware, history-size limits, and diff-size limits must be fixed before implementation acceptance testing.

## Privacy and security requirements

- The client treats all runtime content, filenames, command output, and model text as untrusted display data.
- Markdown and links must not execute code or access local resources implicitly.
- External links show their destination and require an explicit user action.
- The client must not persist secrets, authentication credentials, or sensitive tool payloads in its UI state or logs.
- Clipboard, drag-and-drop, file dialogs, external editor launch, and notification previews require explicit threat-model coverage before implementation.
- Security-sensitive decisions must remain auditable through runtime-owned events.

## Edge cases

- Runtime unavailable on startup or lost during a streamed turn.
- Event gap, duplicate event, out-of-order event, or replay no longer retained.
- Project moved, deleted, disconnected, or permission scope changed.
- Session opened concurrently by more than one client.
- Approval resolved on another client while visible locally.
- Model or mode capability removed while selected.
- Very long unbroken filenames, paths, URLs, code lines, or localized labels.
- Large pasted text, unsupported attachment, binary file, or oversized diff.
- User navigates, closes the inspector, switches theme, or resizes while output streams.
- IME composition active when the send shortcut is pressed.
- Application closes with an unsent draft or active turn.

## Acceptance criteria

This requirements delivery is accepted when:

1. Product and engineering agree on the primary workflow, Phase 1 scope, and explicit non-goals.
2. Annotated wireframes exist for wide, medium, and narrow states in light and dark themes.
3. Every primary surface has empty, loading, populated, disabled, error, offline, and recovery states where applicable.
4. A keyboard-only walkthrough can open a project, create a session, send input, inspect activity and a diff, answer an approval, undo a turn's changes, interrupt a turn, and open settings.
5. The client-runtime contract document covers project, session, turn, events, approvals, checkpoints, settings, models, artifacts, search, diagnostics, and reconnection. It is prose plus test vectors, hand-implemented on each side.
6. Security review approves approval wording, undo wording, link handling, file attachment behavior, logging, notification privacy, and secret-field behavior.
7. The chosen Qt UI technology passes a proof of concept for streaming rich text, virtualized history, accessible controls, and large diff rendering on supported platforms.
8. No client design requires direct access to SQLite, the core process, model providers, or unrestricted local files.
9. Undo is presented with honest scope: filesystem restoration only, with conflicts surfaced and expiry visible.
10. Authority is reviewable: a user can see what was authorized, by whom or which profile, and what resulted.
11. Product copy and visual assets are Suncode-specific and do not copy OpenAI branding.
12. Implementation begins only after the CLI/TUI has exercised the client API, the blocking decisions are resolved, and this requirement is approved.

## Open questions

Blocking:

- Which desktop operating systems and minimum versions are required for the first release?
- Should the implementation use Qt Quick/QML or Qt Widgets? Qt Quick/QML is the current recommendation for this interaction model, pending a proof of concept.
- How much of the client-runtime adapter can be shared with the CLI, given there is no generated SDK and the two surfaces are in different languages?

Resolved:

- Remote runtime selection is out of scope. Suncode is local-first, and the client connects to the local runtime via the discovery file.

Non-blocking:

- Which model and interaction-mode choices will the runtime expose initially?
- Is session deletion permanent, soft-delete, or archive-only in the first release?
- Which draft state is runtime-owned versus client-local, and how long is it retained?
- Can a running session continue when the last desktop client closes? Note the runtime outlives clients by design, so this is a UI expectation question rather than a lifecycle one.
- How is undo presented for a turn whose checkpoint has partially expired?
- Are native notifications enabled by default, and what content may they reveal?
- What are the supported limits for attachments, messages, histories, command output, and diffs?


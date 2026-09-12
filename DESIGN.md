---
name: SunCode Desktop
description: A quiet, reviewable control desk for general-purpose coding work.
colors:
  canvas: "#0d0f12"
  surface: "#121519"
  surface-raised: "#181c21"
  surface-hover: "#1e2329"
  surface-active: "#242a31"
  text: "#edf0f3"
  text-secondary: "#a7afb9"
  text-muted: "#7f8994"
  window-border: "#1c2126"
  window-border-light: "#e5e9ec"
  border: "#292f36"
  accent: "#d9e0e6"
  accent-hover: "#f3f6f8"
  success: "#9fb3c3"
  warning: "#ddb16c"
  danger: "#e68a83"
rounded:
  sm: "6px"
  md: "10px"
  lg: "14px"
spacing:
  panel: "16px"
  section: "24px"
  control: "36px"
components:
  button-primary:
    backgroundColor: "{colors.accent}"
    textColor: "#101317"
    rounded: "{rounded.sm}"
    height: "36px"
  button-neutral:
    backgroundColor: "{colors.surface-raised}"
    textColor: "{colors.text}"
    rounded: "{rounded.sm}"
    height: "36px"
  field:
    backgroundColor: "#15191e"
    textColor: "{colors.text}"
    rounded: "{rounded.sm}"
    height: "36px"

review:
  directory: "design-system/"
  entry: "design-system/index.html"
  app: "design-system/src/main.jsx"
  components: "design-system/src/components/universal/UniversalComponentsPage.jsx"
  tokens: "design-system/src/styles/tokens/"
  assets: "design-system/src/assets/"
---

# Design System: SunCode Desktop

This file is the sole repository-wide authority for durable visual and interaction design decisions. The `design-system/` application is its executable review surface; production clients must follow both.

## Review Surface

The visual review surface starts at [`design-system/index.html`](design-system/index.html), the single Vite/React entry for shared foundations, component states, platform boundaries, project mappings, rules, and assets. Its navigation has three levels: Core, Components, Platforms, and Projects switch the primary content domain from the upper-right; each switch first opens a concise card-based module index without redundant ownership or path labels; and the left sidebar exposes the detailed hierarchy. Every expandable child entry in Core, Components, Platforms, and Projects is a stable route rather than a scroll target: parent pages remain concise card indexes, while the final child page owns its content. The tree can recurse for large surfaces: Desktop Workspace has one complete project-window composition plus stable child routes for Sessions, Explorer, Conversation, Review, Source control, and Provider trace. The top-level switcher uses names only, while detailed hierarchy remains in the sidebar. All React source lives under [`design-system/src/`](design-system/src/), organized by application, core pages, components, platforms, projects, shared primitives, styles, and assets. Each component folder under [`design-system/src/components/universal/`](design-system/src/components/universal/) owns its specimen and stable export; [`design-system/src/components/universal/modules/`](design-system/src/components/universal/modules/) owns the category pages; and [`design-system/src/components/universal/UniversalComponentsPage.jsx`](design-system/src/components/universal/UniversalComponentsPage.jsx) is only the module index. Token sources live under [`design-system/src/styles/tokens/`](design-system/src/styles/tokens/), reusable product images and icons are cataloged in [`design-system/src/assets/`](design-system/src/assets/), and browser-direct files such as the favicon live under [`design-system/public/assets/`](design-system/public/assets/). Review the corresponding hash route before introducing a new module or component. React/Vite is design-review tooling only, not a production web client or Phase 1 runtime dependency.

`design-system/src/styles/tokens/` is the source token reference for the two themes. `foundation.css` owns document reset and focus behavior, `layout.css` owns small composition primitives, and `components.css` is the review entrypoint for colocated universal component styles. `browser.css` owns only the catalog shell and catalog-specific page composition. Desktop project styles live under `projects/desktop/styles/` and workspace styles under `projects/desktop/workspace/styles/`; `review.css` remains a compatibility entrypoint for the remaining shared review surface and legacy cross-surface rules. New component or page rules must not be added there. Avalonia resources in `apps/desktop-avalonia/App.axaml` are the runtime mapping and must retain the same semantic meanings. Feature views should consume named resources rather than add local raw colors, radii, shadows, or control heights.

Desktop Workspace review children also include Editor and Tool activity. Editor owns the read-only file-viewing states described below and remains specification tooling rather than a production web editor.

## Overview

**Creative North Star: “Quiet Control Desk”**

SunCode is designed as a calm professional console for high-consequence coding work. The conversation canvas is the primary surface; navigation, credentials, approvals, checkpoints, and diagnostics are supporting tool bays that can retreat when they are not needed. The system uses matte graphite layers and fine separators to create structure without visual noise.

The interface is intentionally restrained. A cool silver/charcoal accent is reserved for actions and active work, steel blue confirms healthy local state, amber calls attention to authority decisions, and red is reserved for destructive or denied outcomes. There is no decorative glow, glass, gradient text, or dashboard theater.

**Key Characteristics:**

- Conversation-first desktop composition
- Independently collapsible side bays
- Semantic state color, used sparingly
- Tonal layering over drop shadows
- Native Avalonia controls with quiet, component-appropriate focus feedback

## Colors

The palette supports both a dark graphite mode and a lighter paper-and-slate mode. Semantic colors are intentionally distinct and never used as decoration.

### Primary

- **Control Silver** (#d9e0e6 dark / #2c3742 light): Primary actions, active tabs, focus rings, and current work state.
- **Silver Hover** (#f3f6f8 dark / #1e2730 light): Hover state for actionable controls.

### Secondary

- **Healthy Steel** (#9fb3c3 dark / #4f6d82 light): Connected, configured, and ready states.
- **Approval Amber** (#ddb16c): Pending permission and caution states.
- **Risk Red** (#e68a83): Errors, denied actions, and destructive affordances.

### Neutral

- **Graphite Canvas** (#0d0f12): Main window background.
- **Graphite Surface** (#121519): Toolbar and composer surfaces.
- **Raised Graphite** (#181c21): Buttons, checkpoint rows, and focused utility containers.
- **Graphite Hover** (#1e2329): Hover state for neutral controls.
- **Primary Text** (#edf0f3): Main copy and message content.
- **Secondary Text** (#a7afb9): Supporting labels and status summaries.
- **Muted Text** (#7f8994 dark / #65717d light): Empty-state guidance, placeholders, and metadata with accessible normal-text contrast.
- **Window Chrome Hairline** (#1c2126 dark / #e5e9ec light): A 0.5-DIP low-contrast outer outline that lets the native window shadow carry elevation.
- **Hairline Border** (#292f36): Panel boundaries and list separators.

### Named Rules

**The Status Lamp Rule.** Color communicates state or action only. If a color cannot answer “what can I do or what changed?”, it does not belong on the screen.

## Typography

**Display Font:** Noto Sans, with an installed sans fallback only when the target family is unavailable.
**Body Font:** Noto Sans, with the same installed-font fallback.
**Chinese UI Font:** Noto Sans CJK SC, used through the UI fallback stack for Simplified Chinese glyphs when available.
**Code/Data Font:** JetBrains Mono, reserved for code, paths, commands, API values, model identifiers, and runtime data supplied by the client, with an installed monospace fallback only when unavailable.

**Character:** Noto Sans keeps the product calm, contemporary, and readable across dense operating surfaces. Chinese UI text uses Noto Sans CJK SC to avoid mismatched CJK fallback. JetBrains Mono is deliberately isolated to machine-readable strings, where character shapes, punctuation, and path segments need to be distinguishable.

### Hierarchy

- **Heading** (DemiBold, 20px): Current central workspace view and major panel title.
- **Title** (DemiBold, 16px): Product, project, and dialog title.
- **Body** (Regular, 14px, 1.25 line-height): Conversation content and explanatory copy.
- **Label** (Medium, 12px): Controls, status summaries, and section names.
- **Caption** (Regular, 11px): Sequence numbers, file paths, and secondary metadata.

### Named Rules

**The UI Sans + Code Mono Rule.** Product UI uses one sans stack: Noto Sans with Noto Sans CJK SC for Simplified Chinese fallback. JetBrains Mono appears only where the user needs code, path, command, identifier, or runtime data precision.

## Layout

The desktop application has five top-level window roles: ProjectHub, Workspace, DialogWindow, Settings, and About. It opens to a standalone ProjectHub when no project window is active. The hub is a focused list window: recent projects, settings, and the open-project action. It does not auto-select a project. Its default window size is 980 × 712 DIP with a 760 × 552 DIP minimum. Each opened project appears in its own Workspace window, following the IntelliJ-style mental model of one local project per frame. DialogWindow is a focused secondary-confirmation window sibling to the other desktop windows; it opens without a dialog and shows the shared confirmation dialog only after the user invokes a consequential action.

Workspace is the only window with application-drawn title-bar content. It uses `WindowDecorations=BorderOnly`, lets the platform own the outer border and resize behavior, and places a 36px custom title bar above a three-region row with no horizontal separator beneath it. It does not add a transparent outer margin, rounded window frame, hairline border, clipping wrapper, or manual resize hit areas. On macOS, double-clicking an unoccupied title-bar region toggles the window between normal and maximized states rather than entering full screen; the green traffic light remains the explicit full-screen action. ProjectHub, Settings, and About use the platform's full system decorations and do not duplicate title bars or window controls inside the client area. The conversation region is fluid and receives the remaining width. Navigation is constrained to roughly 24% of the window (236–300px); review is constrained to roughly 27% (276–352px). Both side regions can collapse independently, leaving the conversation full width. Source control, provider trace, and Tool activity occupy one mutually exclusive bottom drawer beneath those regions; their gutter controls communicate active state and closing the drawer restores the full conversation height. In the review browser, each Workspace child route renders its panel as a standalone card without repeating the project-window chrome.

The Workspace title bar's leading project title is an interactive project switcher: it shows the current project name and a trailing chevron, and opens a compact menu anchored below the title. `Open project` is always the first action and invokes the same local-folder picker flow as ProjectHub. A divider separates it from `Recent projects`, whose rows show only the project name and path; the menu does not expose window-open status. Selecting a recent project opens a new Workspace window when none exists; when that project already has a Workspace window, the action focuses that window instead. The menu supports keyboard focus, hover, pressed/open, empty recent-project, and constrained-width states without changing the title-bar geometry.

The centered Workspace title is a current-content switcher rather than a fixed session label. It shows the active session title while Conversation is visible and the active file name while the read-only Editor is visible, with a type icon and trailing down chevron. Activating the title opens an anchored `Recently viewed` menu that interleaves sessions and files in most-recent-first order, deduplicates them by stable identity, and retains at most 20 entries internally. The current item remains in that internal history but is omitted from the open menu, so the list contains only other content and the previously active item appears after a switch. Session rows show session recency; file rows show the project-relative or dependency display path. A restrained blue-violet icon surface and type label identify sessions, while a restrained teal-green pair identifies files; row text remains neutral so color is a redundant cue rather than the only distinction. The menu count and empty state reflect only its visible, non-current entries. Selecting a row changes the central content immediately, moves that item to the front, and closes the menu without changing project identity or making file content editable. The switcher supports Escape/outside-click dismissal, keyboard focus, long-title/path elision, a bounded scrolling list, an empty state, and constrained-width geometry.

Workspace presentation state resumes per project when its window is reopened. The left region stores one extensible destination (`closed`, Sessions, or Explorer), the right region stores one extensible destination (`closed` or Review), and the mutually exclusive bottom drawer stores `closed`, Source control, Provider trace, or Tool activity. Normal window bounds, maximized/full-screen state, navigation width, review width, bottom-drawer height, current central file/session, last selected session, and the bounded recent-content history are restored. Window bounds are constrained to an available display before presentation, and responsive breakpoints may temporarily hide a restored region without replacing the user's stored destination. Settings similarly reopens to its last global first-level or provider destination. This restoration changes no agent, project, session, file, or authority data and adds no visual state.

Panel content uses 16px horizontal padding, 10–12px control gaps, and 24px separation around conversation content. The composer occupies a stable 126px footer of the conversation region. The project window supports a 620px compact minimum: supporting panels and drawers retreat, the title-bar panel menu remains available, labels elide, and the conversation stays usable.

Module guide cards open upward from their title control, align to its left edge, and keep the same placement across all modules and responsive states.

## Elevation & Depth

SunCode primarily uses tonal layering rather than app-drawn shadows. The canvas, surface, raised surface, and active surface are close graphite steps separated by one-pixel borders. The conversation composer is the single floating internal surface and uses a soft downward shadow with enough surrounding layout space to render without clipping. Dialogs use a stronger border and a raised tonal surface. Top-level windows use the platform-provided outer border and shadow; clients do not draw a second window outline.

## Shapes

Controls use a compact 6px radius. Utility containers and approval surfaces use 10px. The undo dialog uses 14px. Internal borders are one DIP; keyboard focus may add a focus ring to actions, but input fields keep their one-DIP border without an outer outline. Top-level window border shape belongs to the platform. There are no pill-shaped cards or oversized rounded containers.

## Components

### Buttons

- **Shape:** 6px radius, 30px compact height or 36px regular height.
- **Primary:** High-contrast silver or charcoal background with inverse ink; used for the one action that advances the current task.
- **Neutral:** Raised graphite background with a hairline border; hover lifts to the next tonal layer.
- **Danger:** Transparent at rest, red text and a red-tinted hover surface.
- **Focus:** Two-pixel control-silver border, never removed.

### File Icons

- **Source and scope:** Technical resource lists use the VS Code default Seti file glyphs for supported file types. The shared `FileIcon` review component is the source inventory; folders remain part of the monochrome interface-icon family.
- **Matching:** Exact file names take priority over compound extensions; compound extensions take priority over a detected language or the generic-file glyph. Matching is case-insensitive.
- **Color:** A file glyph may use its Seti type color only to identify content class. It never communicates approval, health, warning, failure, or selection state, and its adjacent filename remains the primary identifier.
- **Geometry:** File glyphs are 14px in Explorer rows and align to the fixed 24px row rhythm. The same glyph retains its color in rest, hover, focus, disabled, and selected rows.

### Cards / Containers

- **Corner Style:** 10px for approval, checkpoint, and diagnostic containers.
- **Background:** Raised graphite or semantic state surface.
- **Shadow Strategy:** None; tonal layering and borders carry depth.
- **Border:** One-pixel hairline border.
- **Internal Padding:** 10–13px for compact inspector rows.

### Inputs / Fields

- **Style:** Field graphite background, 6px radius, one-pixel border, 36px height.
- **Hover:** Keep the one-pixel border and strengthen its color slightly.
- **Focus:** Keep the resting one-pixel border and field background without an additional dark border or outer focus ring. Validation colors remain visible while focused.
- **Disabled:** Muted text and canvas-level contrast; the control remains recognizable but clearly unavailable.

### Conversation Composer And Tool Inspection

- **Conversation history:** The conversation timeline renders submitted user messages and assistant responses in chronological order. User rows use the right-aligned message surface; assistant rows are reading surfaces, so pointer hover does not change their background. Tool payloads remain in Tool activity.
- **Message flow:** User and assistant messages remain visible in chronological order without per-turn divider metadata. Turn and tool details remain available in the supporting activity surfaces.
- **Turn duration:** The active tool row shows a muted duration label immediately above it, using `Working for 18s`-style copy. The latest terminal assistant message shows `Worked for 42s`-style copy immediately above its content.

- **Compact composer:** The default conversation composer remains a compact floating surface anchored to the bottom of the conversation region. Its draft text uses the 14px body size so compact and expanded drafting have the same readable text scale.
- **Expanded drafting:** The composer exposes an explicit expand action that opens a raised modal with a large multi-paragraph drafting textarea and minimal chrome. The modal edits the same draft as the compact composer rather than creating a second independent buffer.
- **Expanded drafting spacing:** When the expanded drafting modal hides its title and close affordance, the textarea begins at the same 20px inset as the dialog's horizontal edges so the editor does not carry an empty header band.
- **Expanded character feedback:** The live character count sits below the drafting field on the left, aligned with the field's content edge.
- **Composer focus:** Compact and expanded composer textareas keep their resting border without adding a dark focus ring when focused.
- **Character feedback:** The expanded drafting modal shows a live character count below the drafting field on the left so long prompts stay measurable without crowding the compact composer.
- **Unavailable model:** When the selected model has no provider API key, keep the conversation history and composer visible but show one compact `Configure an API key in Settings to send messages.` notice above the disabled input. Do not add a second placeholder message inside the disabled field. Disable text entry, expansion, and send while leaving model selection available so the user can switch to a configured model.
- **Conversation scrolling:** Entering a session positions the conversation at its newest message once. Later assistant, tool, and layout updates never move the viewport automatically. When the viewport is away from the newest message, show a compact circular down-arrow control above the composer; activating it returns to the latest message, and the control remains hidden while already at the bottom.
- **Current tool:** The conversation renders at most one compact running-tool row. Activating it opens Tool activity, expands the owning turn, selects that call, and transfers focus to its detail.
- **Tool activity:** The bottom drawer uses a two-level Turn → tool-call tree on the left and a single selected-tool detail on the right. It covers running, completed, failed, and approval states without duplicating Provider trace.
- **Output presentation:** The selected running tool exposes best-effort live output in a bounded monospace viewport. Tail following pauses when the user scrolls away; the terminal result remains the authoritative history.
- **Thinking feedback:** A dedicated thinking phase uses animated `Thinking` text that reveals from left to right and replaces the generic three-dot running marker during that phase.

### Read-only Project Editor

- **Entry:** Selecting a file row in Explorer opens the file in the central Workspace content region, replacing Conversation while leaving the project navigation and review bays intact. Selecting a session from the Sessions list clears the file selection and restores that session's Conversation.
- **Frame:** The editor uses the same surface, border, radius, and spacing tokens as Conversation. Its header is a compact 44px band with the file icon/name, a monospace path, detected language, and an explicit `READ ONLY` status. It is a viewing surface, not a second window.
- **Document:** AvaloniaEdit is configured read-only and exposes selectable text, line numbers, a stable monospace data font, and horizontal scrolling for long lines. No caret, typing affordance, save action, formatting toolbar, or mutation control is shown.
- **Syntax:** TextMate supplies language-aware token colors. Keyword, type, string, number, comment, and punctuation roles use dedicated editor syntax tokens with equivalent contrast in light and dark themes; unsupported languages fall back to readable plain text.
- **States:** The editor contract covers file loading, ready, empty document, bounded read failure, and constrained-width/long-line states. Loading and failure retain file identity and never present an editable field. Empty documents keep the header and read-only status visible.
- **Geometry:** The standalone review specimen keeps a 745px reading viewport inside the 620px Workspace minimum behavior. At constrained widths, the editor yields supporting navigation before shrinking below a usable 300px content minimum; document overflow scrolls horizontally instead of wrapping identifiers.

### Network Certificate Settings

- **Verification scope:** The HTTPS verification toggle remains the primary control for certificate-chain and hostname verification.
- **Certificate source:** When verification stays enabled, the settings surface exposes a subordinate certificate-source toggle for using system certificates.
- **Custom certificates:** Turning off system certificates enables a file-selector field for a custom certificate path. Leaving system certificates on disables that path field rather than hiding it, so the dependency remains visible.
- **Path selector:** Certificate path selection uses the same field-plus-browse-button language as other path selectors, but in file mode rather than folder mode.

### Keyboard Shortcut Settings

- **Placement:** Keyboard shortcuts is a first-level child page in the Settings navigation, alongside Defaults, Appearance, Network, and Logging.
- **Current scope:** The page is a read-only catalog. It must not present editable fields, save actions, or affordances that imply shortcut customization is currently supported.
- **Rows:** Each row places the operation name on the left and a right-aligned combination of compact `<kbd>` keycaps on the right. Use the existing mono data font, raised surface, hairline border, and compact 4px keycap radius.
- **Platform note:** The review surface may use macOS notation for the specimen, but the page should state that Windows and Linux use `Ctrl` where applicable. The catalog should describe shortcuts that are actually implemented by the desktop client.

### MCP Server Settings

- **Placement:** MCP servers is a first-level Settings destination. Individual servers stay in the content list rather than becoming navigation children, so the navigation remains stable as the catalog grows.
- **List:** Each row shows server identity, local-command or remote-URL summary, effective connection state, discovered tool count, an enabled toggle, and compact edit/delete actions. Status colors communicate real connected, connecting, failed, and disabled states only.
- **Scope:** Server definitions are global, but connections and advertised roots are project-scoped. Settings labels the project whose runtime status is shown; without a current project it shows a not-started state.
- **Create and edit:** One shared separate native window edits the server name, transport, transport-specific configuration, enabled state, and timeouts. It has no backdrop and never replaces the Settings content panel. Closing it discards unsaved edits. Local stdio and remote Streamable HTTP are the initial transport choices. Secret environment/header values are write-only and hidden after persistence; `NAME=value` sets or replaces a value and `-NAME` removes a stored key.
- **Immediate application:** A pending mutation disables duplicate actions until SQLite persistence and runtime reconciliation finish. Success updates the row in place; failure remains visible on the affected row with an explicit retry action. No new session is required, and the next turn uses the effective catalog.
- **Deletion:** Delete uses the shared confirmation dialog, names the exact server, and explains that its tools are removed from subsequent model requests while an already executing call may finish.
- **Authority:** Enabling local stdio explicitly warns that it starts a process with the user's authority. MCP tool approvals state that remote side effects may not be undoable by SunCode; connection state must never imply sandboxing or trust.
- **Responsive behavior:** At narrow widths, row metadata and actions wrap into stable bands without hiding the enable control or destructive action. The editor window uses a stable 620 × 610 DIP default with a 520 × 520 DIP minimum; long commands, URLs, and errors scroll or wrap without resizing the window.

### Confirmation Dialogs

- **Shared pattern:** Consequential actions use the reusable confirmation dialog rather than implementing page-specific modal structure or performing the action immediately.
- **Decision content:** The title names the action as a question, the description states the consequence and reversibility, and the body identifies the exact affected item when ambiguity is possible.
- **Actions:** Cancel appears before an explicit verb-led confirmation label. Destructive or list-removal confirmations use the danger button treatment; neutral confirmations use the primary treatment.
- **Safe dismissal:** Cancel receives initial keyboard focus. Escape, the close action, backdrop dismissal, and Cancel all leave state unchanged; only the explicit confirmation action commits the operation.
- **Session archive:** Choosing Archive from a session menu opens a confirmation dialog naming that session. Confirmation removes it from the active list while preserving the ability to reopen it later.

### Navigation

The left bay is project and session navigation, with uppercase section labels, a strong project identity line, and clear session actions. Expanded windows use the side gutter; the title-bar panel menu remains available at every width so hidden supporting surfaces can always be restored without replacing the conversation with a drawer overlay.

### Native Window Frames

Reusable top-level desktop window templates live under `design-system/src/platforms/desktop/components/titlebar/` and are reviewed at Platforms → Desktop → Titlebar. Native-decorated Desktop project specimens such as ProjectHub, Settings, and About compose through this shared frame rather than drawing local window chrome. Workspace is the explicit exception: it owns application-drawn chrome and does not use `NativeWindowFrame`. The native frame owns only the platform chrome and a client-area slot. macOS uses a 28-DIP native title bar with 12-DIP close, minimize, and maximize traffic lights at the leading edge while keeping a 13-DIP window title optically centered; Retina screenshots render these dimensions at twice their logical pixel size. Windows uses a 32-DIP native title bar with application identity at the leading edge and 46-DIP-wide minimize, maximize, and close targets at the trailing edge; close alone receives the native red hover treatment. Product toolbars, navigation, and content begin below this platform-owned title bar. The standard review specimen is 760 × 440px and adapts without changing control order at narrower widths.

### About Version Information

The About window presents the installed `Desktop` and `Agent SDK` product versions as two labeled rows beneath the SunCode identity. Version values use the code/data font and `vX.Y.Z` display format. Desktop reads the Avalonia application assembly metadata, while Agent SDK asynchronously queries the embedded Rust agent core through the SDK bindings and shows an explicit loading or unavailable value when needed. The C ABI compatibility number is an internal binding contract and is not presented as the Agent SDK product version.

### Review Inspector

The right bay contains approval, turn changes, touched files, and runtime health in that order. Approval is the only state allowed to interrupt the visual hierarchy; its amber surface and explicit “Approve once” / “Deny” actions keep authority decisions legible.

### Markdown Content

Assistant messages are rendered as Markdown and use the same semantic content tokens in both themes. The review pages must show the complete reading surface: heading hierarchy, paragraphs, bold/italic/deleted text, links, ordered and unordered lists, task lists, blockquotes, horizontal rules, inline code, fenced code blocks, and tables.

- **Reading measure:** Keep rendered Markdown readable at roughly 680–760px maximum width. Do not force assistant content into a card when the conversation surface already provides the frame.
- **Hierarchy:** Markdown headings are smaller than the application title scale. `h1` starts at 26px in a message, `h2` at 19px, and `h3` at 15px, with spacing that groups related content.
- **Body:** Use the normal UI sans for prose at 14px and 1.6 line height. Links use the control-silver accent with an underline; emphasis changes weight or tone rather than adding semantic colors.
- **Machine content:** Inline code and fenced code use JetBrains Mono on the inset surface. Code blocks scroll horizontally instead of wrapping long identifiers or commands.
- **Structure:** Blockquotes use a quiet outlined accent surface. Tables use compact headers, hairline row separators, and horizontal scrolling on narrow widths. Task-list controls are visual state indicators, not editable product settings.
- **Safety:** Markdown content must not use status colors decoratively. Warning and danger colors remain reserved for actual authority, error, or destructive states around the content.

### Component Coverage

The review pages are required to show, at minimum:

- color tokens for surfaces, text, action, borders, and semantic states
- typography hierarchy, UI/data font split, spacing, radii, and control dimensions
- primary, neutral, quiet, danger, compact, icon-only, focus, pressed, and disabled buttons
- text fields, select fields, textareas, validation, checkbox, radio, and toggle controls
- cards, project rows, activity rows, approval surfaces, navigation, tabs, and segmented controls
- badges, alerts, progress, loading skeletons, empty states, code blocks, and data tables
- Markdown reading surfaces: heading hierarchy, prose, links, lists, task lists, blockquotes, inline code, fenced code, horizontal rules, and compact tables

The same semantic inventory must be present in dark and light pages. Theme changes may alter contrast values, but must not change the meaning of a token or state.

## Do's and Don'ts

### Do:

- **Do** keep the conversation and composer visually dominant.
- **Do** use the control-silver accent only for actions, focus, and active work.
- **Do** make approval scope and undo limitations explicit.
- **Do** preserve familiar Avalonia control behavior and component-appropriate keyboard feedback; text-entry fields must not add a dark focus outline.
- **Do** collapse supporting panels when the user needs room to think.

### Don't:

- **Don't** add gradients, glassmorphism, neon glow, or decorative dashboard metrics.
- **Don't** hide approvals, credential status, conflicts, or runtime errors behind generic success styling.
- **Don't** use equal-weight cards as the page structure.
- **Don't** introduce a second display font or decorative icon language.
- **Don't** let side panels crowd the conversation at compact widths.

## Asset Management

All reusable source-imported design images, brand marks, and interface icons belong in `design-system/src/assets/`. The catalog has stable `logos/` and `icons/` areas with an inventory guide in `design-system/src/assets/README.md`. Browser-direct files such as the favicon belong in `design-system/public/assets/`. Client packaging may copy an approved asset into its own build boundary, but new visual material must first be reviewed and cataloged here.

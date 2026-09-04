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

`design-system/src/styles/tokens/` is the source token reference for the two themes; `design-system/src/styles/review.css` provides shared specimens and `design-system/src/styles/browser.css` provides the catalog shell. Avalonia resources in `apps/desktop-avalonia/App.axaml` are the runtime mapping and must retain the same semantic meanings. Feature views should consume named resources rather than add local raw colors, radii, shadows, or control heights.

## Overview

**Creative North Star: “Quiet Control Desk”**

SunCode is designed as a calm professional console for high-consequence coding work. The conversation canvas is the primary surface; navigation, credentials, approvals, checkpoints, and diagnostics are supporting tool bays that can retreat when they are not needed. The system uses matte graphite layers and fine separators to create structure without visual noise.

The interface is intentionally restrained. A cool silver/charcoal accent is reserved for actions and active work, steel blue confirms healthy local state, amber calls attention to authority decisions, and red is reserved for destructive or denied outcomes. There is no decorative glow, glass, gradient text, or dashboard theater.

**Key Characteristics:**

- Conversation-first desktop composition
- Independently collapsible side bays
- Semantic state color, used sparingly
- Tonal layering over drop shadows
- Native Avalonia controls with visible focus

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

Workspace is the only window with application-drawn title-bar content. It uses `WindowDecorations=BorderOnly`, lets the platform own the outer border and resize behavior, and places a 36px custom title bar above a three-region row with no horizontal separator beneath it. It does not add a transparent outer margin, rounded window frame, hairline border, clipping wrapper, or manual resize hit areas. On macOS, double-clicking an unoccupied title-bar region toggles the window between normal and maximized states rather than entering full screen; the green traffic light remains the explicit full-screen action. ProjectHub, Settings, and About use the platform's full system decorations and do not duplicate title bars or window controls inside the client area. The conversation region is fluid and receives the remaining width. Navigation is constrained to roughly 24% of the window (236–300px); review is constrained to roughly 27% (276–352px). Both side regions can collapse independently, leaving the conversation full width. Source control and provider trace occupy one mutually exclusive bottom drawer beneath those regions; their gutter controls communicate active state and closing the drawer restores the full conversation height. In the review browser, each Workspace child route renders its panel as a standalone card without repeating the project-window chrome.

Panel content uses 16px horizontal padding, 10–12px control gaps, and 24px separation around conversation content. The composer occupies a stable 126px footer of the conversation region. The project window supports a 620px compact minimum: supporting panels and drawers retreat, the title-bar panel menu remains available, labels elide, and the conversation stays usable.

## Elevation & Depth

SunCode primarily uses tonal layering rather than app-drawn shadows. The canvas, surface, raised surface, and active surface are close graphite steps separated by one-pixel borders. The conversation composer is the single floating internal surface and uses a soft downward shadow with enough surrounding layout space to render without clipping. Dialogs use a stronger border and a raised tonal surface. Top-level windows use the platform-provided outer border and shadow; clients do not draw a second window outline.

## Shapes

Controls use a compact 6px radius. Utility containers and approval surfaces use 10px. The undo dialog uses 14px. Internal borders are one DIP at rest and two DIPs only for keyboard focus; top-level window border shape belongs to the platform. There are no pill-shaped cards or oversized rounded containers.

## Components

### Buttons

- **Shape:** 6px radius, 30px compact height or 36px regular height.
- **Primary:** High-contrast silver or charcoal background with inverse ink; used for the one action that advances the current task.
- **Neutral:** Raised graphite background with a hairline border; hover lifts to the next tonal layer.
- **Danger:** Transparent at rest, red text and a red-tinted hover surface.
- **Focus:** Two-pixel control-silver border, never removed.

### Cards / Containers

- **Corner Style:** 10px for approval, checkpoint, and diagnostic containers.
- **Background:** Raised graphite or semantic state surface.
- **Shadow Strategy:** None; tonal layering and borders carry depth.
- **Border:** One-pixel hairline border.
- **Internal Padding:** 10–13px for compact inspector rows.

### Inputs / Fields

- **Style:** Field graphite background, 6px radius, one-pixel border, 36px height.
- **Focus:** Two-pixel control-silver border and slightly lifted field background.
- **Disabled:** Muted text and canvas-level contrast; the control remains recognizable but clearly unavailable.

### Conversation Composer And Tool Inspection

- **Compact composer:** The default conversation composer remains a compact floating surface anchored to the bottom of the conversation region.
- **Stable message rows:** User and assistant message rows are reading surfaces, not list actions, so pointer hover does not change their background. Only actionable controls inside a message, such as attachments, tool rows, and copy actions, receive hover feedback.
- **Expanded drafting:** The composer exposes an explicit expand action that opens a raised modal with a large multi-paragraph drafting textarea and minimal chrome. The modal edits the same draft as the compact composer rather than creating a second independent buffer.
- **Expanded drafting spacing:** When the expanded drafting modal hides its title and close affordance, the textarea begins at the same 20px inset as the dialog's horizontal edges so the editor does not carry an empty header band.
- **Expanded character feedback:** The live character count sits below the drafting field on the left, aligned with the field's content edge.
- **Composer focus:** Compact and expanded composer textareas keep their resting border without adding a dark focus ring when focused.
- **Character feedback:** The expanded drafting modal shows a live character count below the drafting field on the left so long prompts stay measurable without crowding the compact composer.
- **Long message preview:** Submitted user messages that exceed the compact reading measure clamp to five lines with an ellipsis. An eye icon sits inside the message bubble immediately after the truncation and opens a read-only full-message dialog with character count and copy feedback.
- **Tool inspection:** Tool rows stay compact inside the timeline, but opening a running command can reveal a modal with the full request, current status, and a scrollable live command-output pane.
- **Output presentation:** Live command output uses the code/data monospace treatment, wraps safely for narrow widths, and never turns the conversation into a dashboard.
- **Thinking feedback:** A dedicated thinking phase uses animated `Thinking` text that reveals from left to right and replaces the generic three-dot running marker during that phase.

### Network Certificate Settings

- **Verification scope:** The HTTPS verification toggle remains the primary control for certificate-chain and hostname verification.
- **Certificate source:** When verification stays enabled, the settings surface exposes a subordinate certificate-source toggle for using system certificates.
- **Custom certificates:** Turning off system certificates enables a file-selector field for a custom certificate path. Leaving system certificates on disables that path field rather than hiding it, so the dependency remains visible.
- **Path selector:** Certificate path selection uses the same field-plus-browse-button language as other path selectors, but in file mode rather than folder mode.

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
- **Do** preserve native keyboard focus and familiar Avalonia control behavior.
- **Do** collapse supporting panels when the user needs room to think.

### Don't:

- **Don't** add gradients, glassmorphism, neon glow, or decorative dashboard metrics.
- **Don't** hide approvals, credential status, conflicts, or runtime errors behind generic success styling.
- **Don't** use equal-weight cards as the page structure.
- **Don't** introduce a second display font or decorative icon language.
- **Don't** let side panels crowd the conversation at compact widths.

## Asset Management

All reusable source-imported design images, brand marks, and interface icons belong in `design-system/src/assets/`. The catalog has stable `logos/` and `icons/` areas with an inventory guide in `design-system/src/assets/README.md`. Browser-direct files such as the favicon belong in `design-system/public/assets/`. Client packaging may copy an approved asset into its own build boundary, but new visual material must first be reviewed and cataloged here.

---
name: SunCode Desktop
description: A quiet, reviewable control desk for local-first coding work.
colors:
  canvas: "#0d0f12"
  surface: "#121519"
  surface-raised: "#181c21"
  surface-hover: "#1e2329"
  surface-active: "#242a31"
  text: "#edf0f3"
  text-secondary: "#a7afb9"
  text-muted: "#838d98"
  border: "#292f36"
  accent: "#69c5b0"
  accent-hover: "#7dd2be"
  success: "#78c99b"
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
    textColor: "#07120f"
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
---

# Design System: SunCode Desktop

## Overview

**Creative North Star: “Quiet Control Desk”**

SunCode is designed as a calm professional console for high-consequence local work. The conversation canvas is the primary surface; navigation, credentials, approvals, checkpoints, and diagnostics are supporting tool bays that can retreat when they are not needed. The system uses matte graphite layers and fine separators to create structure without visual noise.

The interface is intentionally restrained. Teal is reserved for actions and active work, green confirms healthy local state, amber calls attention to authority decisions, and red is reserved for destructive or denied outcomes. There is no decorative glow, glass, gradient text, or dashboard theater.

**Key Characteristics:**

- Conversation-first desktop composition
- Independently collapsible side bays
- Semantic state color, used sparingly
- Tonal layering over drop shadows
- Native Avalonia controls with visible focus

## Colors

The palette supports both a dark graphite mode and a lighter paper-and-slate mode. Semantic colors are intentionally distinct and never used as decoration.

### Primary

- **Muted Control Teal** (#69c5b0): Primary actions, active tabs, focus rings, and current work state.
- **Teal Hover** (#7dd2be): Hover state for actionable controls.

### Secondary

- **Healthy Green** (#78c99b): Connected, configured, and ready states.
- **Approval Amber** (#ddb16c): Pending permission and caution states.
- **Risk Red** (#e68a83): Errors, denied actions, and destructive affordances.

### Neutral

- **Graphite Canvas** (#0d0f12): Main window background.
- **Graphite Surface** (#121519): Toolbar and composer surfaces.
- **Raised Graphite** (#181c21): Buttons, checkpoint rows, and focused utility containers.
- **Graphite Hover** (#1e2329): Hover state for neutral controls.
- **Primary Text** (#edf0f3): Main copy and message content.
- **Secondary Text** (#a7afb9): Supporting labels and status summaries.
- **Muted Text** (#838d98): Empty-state guidance and metadata with accessible contrast on graphite surfaces.
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

The app opens to a standalone project hub when no project window is active. The hub is a focused list window: recent projects, settings, and the open-project action. It does not auto-select a project. Each opened project appears in its own project window, following the IntelliJ-style mental model of one local project per frame.

The default project window frame is a 54px top bar above a three-region row. The conversation region is fluid and receives the remaining width. Navigation is constrained to roughly 24% of the window (236–300px); review is constrained to roughly 27% (276–352px). Both side regions can collapse independently, leaving the conversation full width.

Panel content uses 16px horizontal padding, 10–12px control gaps, and 24px separation around conversation content. The composer is a stable 148px footer of the conversation region. At the 900px minimum width, labels elide and the center remains usable rather than allowing side panels to consume the work surface.

## Elevation & Depth

SunCode uses tonal layering rather than shadows. The canvas, surface, raised surface, and active surface are close graphite steps separated by one-pixel borders. Dialogs use a stronger border and a raised tonal surface; shadows are not part of the visual language.

## Shapes

Controls use a compact 6px radius. Utility containers and approval surfaces use 10px. The undo dialog uses 14px. Borders are one pixel at rest and two pixels only for keyboard focus. There are no pill-shaped cards or oversized rounded containers.

## Components

### Buttons

- **Shape:** 6px radius, 30px compact height or 36px regular height.
- **Primary:** Muted teal background with near-black ink; used for the one action that advances the current task.
- **Neutral:** Raised graphite background with a hairline border; hover lifts to the next tonal layer.
- **Danger:** Transparent at rest, red text and a red-tinted hover surface.
- **Focus:** Two-pixel teal border, never removed.

### Cards / Containers

- **Corner Style:** 10px for approval, checkpoint, and diagnostic containers.
- **Background:** Raised graphite or semantic state surface.
- **Shadow Strategy:** None; tonal layering and borders carry depth.
- **Border:** One-pixel hairline border.
- **Internal Padding:** 10–13px for compact inspector rows.

### Inputs / Fields

- **Style:** Field graphite background, 6px radius, one-pixel border, 36px height.
- **Focus:** Two-pixel teal border and slightly lifted field background.
- **Disabled:** Muted text and canvas-level contrast; the control remains recognizable but clearly unavailable.

### Navigation

The left bay is project and session navigation, with uppercase section labels, a strong project identity line, and clear session actions. It is hidden or restored using a top-bar control, preserving the central conversation rather than replacing it with a drawer overlay.

### Review Inspector

The right bay contains approval, turn changes, touched files, and runtime health in that order. Approval is the only state allowed to interrupt the visual hierarchy; its amber surface and explicit “Approve once” / “Deny” actions keep authority decisions legible.

## Do's and Don'ts

### Do:

- **Do** keep the conversation and composer visually dominant.
- **Do** use teal only for actions, focus, and active work.
- **Do** make approval scope and undo limitations explicit.
- **Do** preserve native keyboard focus and familiar Avalonia control behavior.
- **Do** collapse supporting panels when the user needs room to think.

### Don't:

- **Don't** add gradients, glassmorphism, neon glow, or decorative dashboard metrics.
- **Don't** hide approvals, credential status, conflicts, or runtime errors behind generic success styling.
- **Don't** use equal-weight cards as the page structure.
- **Don't** introduce a second display font or decorative icon language.
- **Don't** let side panels crowd the conversation at compact widths.

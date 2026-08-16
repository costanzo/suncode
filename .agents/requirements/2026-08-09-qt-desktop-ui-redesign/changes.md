# Changes

## Source

- Added semantic Qt/QML theme tokens and shared `AppButton`, `AppField`, and `SectionLabel` primitives.
- Rebuilt `Main.qml`, `ConnectionPanel.qml`, `ConversationPanel.qml`, and `ReviewPanel.qml` around the Quiet Control Desk direction.
- Added standalone project hub, separate project windows, global settings, a model selector beside the composer, and a right-side agent/process panel.
- Added independent navigation/review collapse controls and improved empty, approval, credential, runtime, undo, and active-turn states.
- Removed the visible Activity tab for now; the runtime event/activity data remains intact for a future dedicated surface.
- Fixed hub startup so recent projects load without auto-selecting a project or requesting an empty session snapshot.
- Fixed QML theme binding shadowing in hub/settings/project windows so shared controls receive the intended design tokens.
- Restyled the composer into a floating card with rounded corners, shadow, model selector chip, and icon-only send/stop action.
- Added tonal separation and shadow transitions between the side bays and the central conversation region.
- Deepened the sidebar surfaces and raised the central workspace contrast so the panel boundaries read more clearly in dark mode.
- Added user-selectable light and dark appearance modes in global settings and wired them across hub, project, and main windows.
- Reorganized global settings into a left-hand tree with mutually exclusive Defaults, Appearance, and Model providers → DeepSeek detail pages.
- Added a compact top-right gear action in project windows, removed sidebar icon tooltips, and made side-bay visibility changes immediate instead of animated.
- Reframed the project workspace as an inset card layout: the canvas remains visible around a rounded conversation card with matching rounded navigation and review cards separated by a stable gap.
- Added the desktop typography stack: UI text prefers Noto Sans with Noto Sans CJK SC for Simplified Chinese fallback, while paths, commands, JSON/API values, model identifiers, and runtime data use JetBrains Mono when available.
- Moved the left navigation open/close control out of the navigation card and into the transparent 24px gutter to the card's left; the navigation card now collapses to zero width instead of showing an internal rail.
- Removed per-row Open buttons from the project hub recent-project list; each project row is now a full-width focusable click target that opens the project directly.
- Set the project hub recent-project rows to use a pointing-hand cursor on hover.
- Replaced the ProjectWindow system title bar with a frameless custom title bar that exposes project actions, settings, minimize, maximize, and close controls while preserving drag-to-move behavior.
- Refined the ProjectWindow custom title bar for macOS so the traffic-light controls sit on the left, the title stays centered, and the drag region stays clear of the buttons.
- Softened the ProjectWindow right-side title-bar action buttons on macOS so they read as lightweight toolbar controls instead of boxed widgets.
- Standardized pointing-hand cursors for enabled clickable controls across shared buttons, sidebar controls, title-bar buttons, settings tree items, model/theme selectors, recent-project rows, and composer actions.
- Rounded the frameless ProjectWindow chrome on macOS, matched the custom title-bar color to the window canvas, and recentered title-bar button icons inside their hit targets.
- Removed the ProjectWindow title-bar divider so the custom chrome and main window canvas read as one continuous surface.
- Reworked the custom title-bar drag region to use Qt's thresholded `DragHandler`, with a separate double-click handler for maximize/restore so ordinary clicks no longer start a move immediately.
- Added a standard Preferences shortcut so `Command+,` on macOS opens the global settings window from the project hub or a project window, reusing an existing settings window when one is already open.
- Bound the Preferences shortcut with `Shortcut.sequences` so Qt registers all platform key bindings without the single-sequence warning.
- Replaced the QML `Shortcut` binding with standard `Action.shortcut` wiring for the hub and project menu after the standalone shortcut failed to trigger reliably on macOS.
- Added a reusable `WindowStateController` helper that owns remembered normal geometry and routes custom-title-bar move/resize gestures through `startSystemMove()` / `startSystemResize()`.
- Reworked the macOS custom title-bar drag region so a drag from the maximized state restores to the remembered size before handing off to the native move loop.
- Added transparent `WindowResizeHandles` overlays to the title bar and window body so the frameless project window still has native edge and corner resize hot zones.
- Scoped the Preferences shortcut to the active visible hub/project window and made project windows reuse the hub-owned settings instance, so `Command+,` cannot create duplicate settings windows during normal app flow.
- Made global settings application-modal and transient for the invoking window, with Esc/Cancel closing support.
- Added a window-level project navigation toggle action bound to `Ctrl+1`, which maps to `Command+1` on macOS through Qt's `QKeySequence` platform conventions.
- Added an Open Recent Project submenu to ProjectWindow; selecting a recent project opens that project in its own window, or focuses the existing window if it is already open.
- Tightened project-window lifecycle handling so the hub stays hidden while any project window remains open and reappears only after the last one closes.
- Restored Windows minimize, maximize/restore, and close controls in the frameless ProjectWindow title bar, reduced its Windows chrome height, and kept Windows maximize behavior distinct from macOS fullscreen.
- Made project windows explicitly non-transient so hiding the project hub no longer removes the SunCode taskbar entry on Windows.
- Restored a compact 4px outer inset around Windows project windows so their frameless chrome has comparable visual breathing room to macOS without increasing the 36px title bar.
- Added the SunCode logo to the upper-left corner of Windows project windows while preserving the macOS traffic-light control cluster.
- Aligned the Windows title-bar logo to the left gutter centerline and refined the minimize, maximize/restore, and close controls to Windows caption-button proportions and hover states, including the native-style red close hover.
- Replaced the Windows minimize, maximize, and close glyphs with clean per-icon assets extracted from the supplied combined SVG while retaining the existing restore-state glyph.
- Rendered the revised Windows caption assets at their native 12px canvas size without shrinking their 46x36px hit regions.
- Centered the Windows settings action within the 36px title bar and reduced its gear glyph to the standard 16px toolbar size.
- Matched the Windows close-button hover surface to the requested native red `#E81123`.
- Made shared themed SVG icons rasterize at the active window's device-pixel ratio so 150% Windows scaling no longer magnifies a low-resolution intermediate texture.
- Rendered enabled Windows caption glyphs as pure black in light mode and white in dark mode, with the close glyph switching to white over its red hover/pressed surface.
- Hid Windows verbatim-path prefixes in ProjectHub project rows while preserving canonical runtime values, including correct `\\?\UNC\` to `\\server\share` display conversion.
- Added a restrained outer shadow around restored Windows project windows using a separate lightweight shadow source, without offscreen-rendering the complete application content.

## Contracts and generated artifacts

- No protocol or generated artifact changes planned.

## Configuration and persistence

- Added user-scoped settings calls for the default model. Provider credentials remain in the OS credential store; no database schema changes were made.

## Tests

- `cmake --build apps/desktop-qt/build -j2` passed.
- Offscreen Qt startup reached the project hub without QML TypeError, Runtime SDK connection-race errors, or missing-font alias warnings after font fallback resolution.
- `node .agents/skills/impeccable/scripts/detect.mjs --json` returned `[]`.
- `node .agents/skills/impeccable/scripts/detect.mjs --json --scope type apps/desktop-qt/qml apps/desktop-qt/src/main.cpp DESIGN.md` returned `[]`.
- `node .agents/skills/impeccable/scripts/detect.mjs --json --scope layout apps/desktop-qt/qml/ProjectWindow.qml apps/desktop-qt/qml/ConnectionPanel.qml` returned `[]`.
- `node .agents/skills/impeccable/scripts/detect.mjs --json --scope layout apps/desktop-qt/qml/AppButton.qml apps/desktop-qt/qml/ProjectHub.qml apps/desktop-qt/qml/ProjectWindow.qml apps/desktop-qt/qml/GlobalSettings.qml apps/desktop-qt/qml/ConversationPanel.qml apps/desktop-qt/qml/SidebarToggleButton.qml apps/desktop-qt/qml/SidebarLockButton.qml apps/desktop-qt/qml/SidebarRail.qml` returned `[]`.
- `node .agents/skills/impeccable/scripts/detect.mjs --json --scope layout apps/desktop-qt/qml/ProjectWindow.qml` returned `[]`.
- `git diff --check` passed.

## Documentation

- Added this requirement package and updated durable Qt desktop feature/design notes.

- Changed the macOS green title-bar control to drive real fullscreen mode (`showFullScreen()` / `showNormal()`), while keeping the custom title bar visible and disabling minimize in fullscreen.

- Moved ProjectWindow project actions into the macOS system menu bar and set the app display name to SunCode.
- Kept the native Open Recent Project submenu structurally visible while project data loads, disabling it when empty; this avoids a Qt 6.11.1 macOS crash in `QQuickMenuPrivate::setNativeMenuVisible()` when the asynchronous project list changes.
- Added a compact project-window footer with a right-aligned placeholder and kept the frameless window's native bottom-edge and corner resize behavior attached to the true outer edge.
- Unified ProjectWindow under one canvas-colored background and inset the toolbar, navigation, conversation, process, and footer surfaces as consistently spaced cards.
- Made the exposed top canvas strip, clear toolbar area, and left navigation gutter native window-drag regions while keeping traffic lights, settings, and the navigation toggle interactive.
- Replaced the footer placeholder with compact runtime connection and selected-model status while preserving fullscreen footer suppression.
- Replaced the right sidebar's full-height collapsed rail with a transparent draggable gutter and top-aligned toggle that mirrors the left navigation gutter.
- Tightened ProjectWindow card gaps and outer inset, and reduced both side-card target widths so the conversation retains more space at default and minimum window sizes.
- Flattened the ProjectWindow header and footer into the window background by removing their card surfaces and borders, and removed the redundant footer connection-status display.
- Reduced the transparent ProjectWindow footer to 18px so its single model label does not take space from the conversation.
- Reduced the horizontal window inset to 4px and both transparent sidebar gutters to 26px while preserving the existing vertical inset and control hit areas.
- Removed the duplicate collapse button and pin/auto-hide lock from the right process panel; the outer right-gutter toggle is now its only visibility control.
- Constrained ProjectWindow conversation messages and Composer to a centered 780px content column, matching the default width with both side panels open while still shrinking with existing margins at narrower widths.
- Removed the visible conversation scrollbar while retaining wheel, trackpad, and direct ListView scrolling.
- Added a shared rounded ComboBox treatment for model/theme selectors, including rounded option popups, and replaced the Session dialog's default square title header with a rounded-surface-compatible header.
- Replaced the Session row action menu's default square popup chrome with a themed rounded surface and rounded hover/focus states.
- Simplified the Session create/rename dialog by removing explanatory copy and centering its title.
- Regenerated all desktop logo PNGs and the macOS `suncode-desktop.icns` from the SVG sources with transparent corners, removing the opaque white canvas visible around the Dock icon.

### 2026-08-15

- Kept the macOS ProjectWindow custom title bar visible in fullscreen, disabled the minimize traffic light in that state, and left close/fullscreen controls available.
- Regenerated `suncode-desktop.icns` with macOS `iconutil` from a complete iconset whose artwork is scaled to 76% of the canvas, giving the Dock icon more transparent optical padding while preserving the existing mark.
- Removed the dark full-canvas background from the SunCode logo SVGs and regenerated the raster logo assets and `suncode-desktop.icns` from the transparent source art so the Dock icon no longer reads with a black rim.
- Kept the compact ProjectWindow footer visible at a fixed 18px height in macOS fullscreen instead of suppressing it.

### 2026-08-16

- Regenerated the macOS Dock icon assets with 80% artwork coverage so SunCode reads closer to neighboring Dock icons while preserving transparent padding.

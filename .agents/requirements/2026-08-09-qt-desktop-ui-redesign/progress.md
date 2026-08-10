# Progress

- Status: Complete
- Last updated: 2026-08-12

## Completed

- Confirmed the primary workflow and constraints with the user.
- Selected the Quiet Control Desk direction through the Impeccable direction flow.
- Recorded the delivery requirement and presentation boundary.

## In progress

- None.

## Blocked

- None.

## Log

### 2026-08-09

- Requirement initialized and design direction approved.
- Rebuilt the QML theme, app shell, navigation, conversation, and review inspector.
- Added independent collapse controls for both side bays.
- Added DESIGN.md and the Impeccable design sidecar from the shipped visual system.
- Qt build and Impeccable detector passed; native screenshot capture was unavailable for the unbundled binary.
- Refined global settings into a provider tree with one visible detail page at a time.
- Added the project-window gear entry, removed sidebar icon tooltips, and removed side-bay width animations.
- Rebuilt and ran the desktop target after the refinement; QML layout detection remains clean.

### 2026-08-10

- Confirmed `cmake --build apps/desktop-qt/build -j2` passes after the settings and shell updates.
- Confirmed `qmllint` reports no error or missing-property diagnostics for `GlobalSettings.qml`.
- Confirmed offscreen desktop startup reaches the Qt event loop without QML runtime errors; only the existing system font alias notice is emitted.
- Wrapped the project navigation, conversation, and agent review regions in rounded inset cards while preserving the existing client bindings and collapse behavior.
- Added a resolved desktop font stack: Noto Sans for UI, Noto Sans CJK SC for Simplified Chinese fallback, and JetBrains Mono for code/path/command/runtime strings when those families are installed.
- Confirmed offscreen desktop startup reaches the Qt event loop without missing-font alias warnings after resolving unavailable font families to installed fallbacks.
- Moved the left navigation toggle into the transparent left gutter and removed the duplicate toggle from inside the navigation card.
- Removed the project hub's per-row Open button and made the full recent-project row open the project directly.
- Added a pointing-hand cursor for hover over clickable recent-project rows.
- Replaced the ProjectWindow native title bar with a custom frameless header that handles drag, project actions, settings, and window controls.
- Refined the ProjectWindow header to follow macOS chrome conventions more closely: left-aligned traffic-light controls, centered title, and a clearer drag strip between the control clusters.
- Softened the non-traffic-light title-bar buttons on macOS so the whole header reads more like native toolbar chrome.
- Standardized enabled clickable controls to use pointing-hand cursors across the shared button system, sidebar controls, title bar, project hub rows, settings navigation/selectors, and composer actions.
- Rounded the frameless ProjectWindow chrome on macOS, aligned the title-bar surface color with the main canvas, and recentered title-bar icons.
- Removed the ProjectWindow title-bar divider to make the custom chrome blend into the window canvas.
- Reworked title-bar dragging to start only after the pointer crosses Qt's drag threshold and kept double-click maximize/restore independent from the move gesture.

### 2026-08-11

- Added the standard Preferences shortcut (`Command+,` on macOS) in the project hub and project windows, and made repeated settings invocations focus the existing settings window.
- Replaced the single `Shortcut.sequence` binding with `Shortcut.sequences` for the Preferences standard key to remove Qt's multiple-bindings warning.
- Replaced the QML `Shortcut` item with standard `Action.shortcut` wiring so the Preferences command participates in Qt Quick Controls' action shortcut path.
- Added a reusable window-state helper for custom-title-bar maximize/restore and native system move/resize calls.
- Reworked the custom title bar drag region to restore from maximized state before starting a native move, using a drag-thresholded pointer path rather than immediate move-on-press.
- Added transparent resize handles for the frameless window body so the custom chrome still supports native edge and corner resizing.
- Changed settings invocation so project windows delegate to the hub-owned single settings window, preventing duplicate `Command+,` settings windows.
- Made the settings window application-modal with the active project window as transient parent, and added Esc/Cancel closing behavior.
- Added a project-window `Command+1` shortcut for toggling the left project navigation sidebar.
- Added an Open Recent Project submenu to ProjectWindow and made opening an already-open project focus the existing window instead of creating a duplicate.
- Kept the hub hidden while project windows exist and restored it only after the final project window closes.

- Fixed the macOS green title-bar control to enter and exit fullscreen with the title bar hidden and the window shell removed in fullscreen.

- Moved ProjectWindow project actions into the macOS system menu bar and set the app display name to Suncode.

### 2026-08-12

- Diagnosed the project-window startup crash from the macOS `.ips` report: the asynchronous `projectsChanged` signal reevaluated the native recent-project menu's `visible` binding inside Qt's menu event delivery.
- Replaced the native submenu visibility binding with an enabled-state binding so its structure remains stable while project data loads.
- Added a compact footer to ProjectWindow with right-aligned placeholder content and moved the bottom resize handles from the content boundary to the footer's outer edge.
- Reframed the complete ProjectWindow as one background surface containing inset toolbar, navigation, conversation, process, and footer cards with consistent gutters.
- Extended native window dragging to the exposed top and left background areas and retained clear-toolbar dragging without intercepting embedded controls.
- Matched the right sidebar's collapsed presentation to the left: the process card now leaves the layout when hidden, while its toggle stays top-aligned in a transparent draggable 28px gutter.
- Reduced the inset shell to a 6px outer inset and 4px card gap, with narrower navigation/process targets that return roughly 54–56px to the conversation at supported widths.
- Removed the header/footer card styling so both regions blend into the outer canvas, and simplified the footer to the selected model only.
- Compressed the model-only footer from 26–30px to 18px while retaining the outer bottom resize handle.
- Tightened the ProjectWindow's left and right edge space by 4px per side without changing its top/bottom rhythm or sidebar toggle size.
- Removed the redundant in-panel process-sidebar toggle, lock state, and hover-auto-hide behavior so the right gutter exclusively controls panel visibility.
- Added a shared responsive width constraint for the ProjectWindow message list and Composer: 780px maximum, centered when extra space is available, fluid below that limit.
- Removed the visible message scrollbar after the edge-mounted style proved unreliable; native wheel, trackpad, and direct scrolling remain available.
- Unified all desktop model/theme selectors under `AppComboBox` and completed the Session create/rename dialog's rounded chrome by removing the Fusion header surface.
- Rounded the Session row three-dot action menu and its Rename/Archive interaction states instead of relying on the square default Qt menu background.
- Removed the redundant Session dialog description and centered the create/rename title for a more compact hierarchy.
- Rebuilt every desktop logo raster size and macOS iconset layer with a preserved alpha channel so the Dock icon is centered without baked-in white margins.

# Test Plan

## Scope

Qt Quick presentation and interaction wiring for the Phase 1 desktop client.

## Unit tests

No standalone QML unit harness exists; compile-time validation covers component/property errors.

## Integration and conformance tests

No runtime contracts change. Existing Rust and Qt focused tests remain applicable.

## Regression checks

- Configure and build `apps/desktop-qt`.
- Launch the binary and capture the default window.
- Verify the default launch is the project hub, not an auto-selected project window.
- Verify recent projects render from runtime project storage when no project is selected.
- Verify recent project rows open their projects directly on row click or keyboard activation, without a per-row Open button.
- Verify opening another project creates an additional project window rather than merging projects.
- Verify left and right panel toggle behavior.
- Verify settings navigation selects one detail page at a time, provider folders expand/collapse, and the project-window gear opens settings.
- Verify sidebar rail and lock icons do not display hover tooltips and panel visibility changes do not animate.
- Verify the project workspace shows visible canvas padding, rounded card corners on all three regions, and consistent gaps between cards at default and minimum window sizes.
- Verify the left navigation toggle sits in the transparent gutter outside the navigation card, can both open and close the card, and does not leave an internal collapsed rail behind.
- Verify ordinary UI text uses the resolved Noto Sans stack, Simplified Chinese can fall through to Noto Sans CJK SC when installed, and code/path/command/runtime strings use the resolved JetBrains Mono stack.
- Verify the ProjectWindow frameless title bar renders project title, actions, settings, minimize, maximize/restore, and close controls, and that the window can still be dragged from the custom title bar.
- Verify the ProjectWindow title bar uses macOS-style left traffic-light controls on macOS, keeps the title centered, and leaves the control cluster clear of drag gestures.
- Verify a short title-bar click does not move the window, a drag begins after the normal pointer threshold, and a double-click on the clear title strip toggles maximize/restore without affecting the traffic-light or action buttons.
- Verify dragging the clear title strip from a maximized project window restores the remembered normal geometry before handing off to native window movement.
- Verify the frameless project window can be resized from transparent title-bar top handles plus body left/right/bottom edges and corners without visible resize chrome.
- Verify the ProjectWindow frameless chrome has rounded outer corners on macOS when not maximized, uses the same canvas color through the title bar and body without a divider line, and keeps title-bar icons visually centered.
- Verify the ProjectWindow footer stays at the bottom with only the selected model on the right, does not overlap the work area, hides in fullscreen, and preserves bottom-edge/corner resizing.
- Verify the ProjectWindow header and footer render directly on the outer canvas without separate card fills, borders, or rounded containers.
- Verify ProjectWindow uses one continuous outer background with inset toolbar, navigation, conversation, process, and footer cards separated by consistent gutters at default and minimum sizes.
- Verify the exposed top background strip, clear toolbar area, and left navigation gutter can move the window without intercepting traffic lights, settings, or the navigation toggle.
- Verify the right sidebar uses a transparent 26px gutter with its toggle aligned to the top like the left navigation toggle, and that collapsing the process card leaves no full-height colored rail.
- Verify the right gutter is the only process-panel visibility toggle and the process panel no longer auto-hides when the pointer leaves it.
- Verify messages and Composer remain centered at a 780px maximum when either or both side panels collapse, and shrink with their existing margins when the conversation region becomes narrower than 780px.
- Verify overflowing conversations remain scrollable with wheel, trackpad, and direct drag input without rendering a visible scrollbar.
- Verify Composer/default-model/theme selectors have rounded fields and rounded option popups in both themes, including hover, selected, focus, disabled, and long-model-name states.
- Verify the shared Session create/rename dialog has rounded outer corners without a square Fusion title header covering its top corners.
- Verify the Session row three-dot menu has rounded outer corners and rounded pointer/keyboard selection states in both themes.
- Verify the Session create/rename dialog contains no explanatory paragraph and centers its title above the name field.
- Verify all logo PNGs and every macOS iconset layer have transparent corner pixels, and verify the bundled Dock icon has no white canvas or directional offset at standard and Retina sizes.
- Verify the compact 6px outer inset and 4px card gaps remain visually distinct, while the narrower side cards preserve usable controls and leave a materially wider conversation at 1440px and 900px widths.
- Verify enabled clickable buttons, clickable rows, sidebar controls, settings navigation items, and model/theme selectors show a pointing-hand cursor while disabled controls do not.
- Verify `Command+,` on macOS opens global settings from the project hub and project windows, and repeated activation focuses the existing settings window instead of opening duplicates.
- Verify only the active visible project/hub window handles `Command+,`, settings opens as a modal child above the invoking window, and the project/hub cannot be operated until settings closes.
- Verify Esc closes the settings window without creating or leaving duplicate settings windows.
- Verify `Command+1` on macOS toggles the ProjectWindow left project navigation sidebar and stays scoped to the active project window.
- Verify ProjectWindow's Open Recent Project submenu opens a separate project window, focuses an already-open window for the same project, and never merges projects into tabs.
- Verify the hub remains hidden while any project window exists and reappears only after the last project window closes.
- Verify the macOS green title-bar control enters fullscreen, hides the custom title bar, and exits back to the prior normal window state.
- Verify the macOS system menu bar shows Project actions at the top of the screen and the app display name reads Suncode instead of suncode-desktop.
- Verify loading the asynchronous recent-project list while a ProjectWindow exists does not crash, and that the Open Recent Project submenu is disabled only when the list is empty.
- Verify connection, project/session, credential, composer, approval, undo, and diagnostic controls retain bindings.

## Manual checks

- Default 1440×900 hierarchy and alignment.
- Minimum 900×620 layout.
- Empty, disconnected, disabled, approval, active-turn, and long-content states where available.
- Keyboard focus visibility and text contrast.

## Commands and results

- `cmake --build apps/desktop-qt/build -j2` passed.
- Offscreen startup of `apps/desktop-qt/build/suncode-desktop` reached the project hub without QML TypeError, Runtime SDK connection-race errors, or missing-font alias warnings after font fallback resolution.
- `node .agents/skills/impeccable/scripts/detect.mjs --json` returned `[]`.
- `node .agents/skills/impeccable/scripts/detect.mjs --json --scope type apps/desktop-qt/qml apps/desktop-qt/src/main.cpp DESIGN.md` returned `[]`.
- `node .agents/skills/impeccable/scripts/detect.mjs --json --scope layout apps/desktop-qt/qml/AppButton.qml apps/desktop-qt/qml/ProjectHub.qml apps/desktop-qt/qml/ProjectWindow.qml apps/desktop-qt/qml/GlobalSettings.qml apps/desktop-qt/qml/ConversationPanel.qml apps/desktop-qt/qml/SidebarToggleButton.qml apps/desktop-qt/qml/SidebarLockButton.qml apps/desktop-qt/qml/SidebarRail.qml` returned `[]`.
- `node .agents/skills/impeccable/scripts/detect.mjs --json --scope layout apps/desktop-qt/qml/ProjectWindow.qml` returned `[]`.
- `qmllint apps/desktop-qt/qml/*.qml` completed with exit code 0; it reported existing broad QML style warnings for unqualified access and layout-managed width/height.
- `git diff --check` passed.
- `qmllint apps/desktop-qt/qml/ProjectHub.qml apps/desktop-qt/qml/ProjectWindow.qml` completed with exit code 0 after adding the Preferences shortcut; it still reports existing unresolved `Suncode.Runtime` and unqualified-access warnings.
- `node .agents/skills/impeccable/scripts/detect.mjs --json --scope layout apps/desktop-qt/qml/ProjectHub.qml apps/desktop-qt/qml/ProjectWindow.qml` returned `[]`.
- `cmake --build apps/desktop-qt/build -j2` passed after adding the shortcut.
- Offscreen startup of `apps/desktop-qt/build/suncode-desktop` reached the Qt event loop with no QML runtime output before manual interruption.
- Offscreen startup after switching Preferences to `Shortcut.sequences` produced no `QML Shortcut` multiple-bindings warning before manual interruption.
- `cmake --build apps/desktop-qt/build -j2` passed after replacing QML `Shortcut` with `Action.shortcut`.
- Real GUI startup after the `Action.shortcut` change produced no QML shortcut warnings before manual interruption; macOS still emitted the unrelated IMK run-loop message on interruption.
- `cmake --build apps/desktop-qt/build -j2` passed after adding the title-bar state controller and transparent resize handles.
- `/Users/shuyi/Softwares/qt/6.11.1/macos/bin/qmllint apps/desktop-qt/qml/ProjectWindow.qml apps/desktop-qt/qml/WindowStateController.qml apps/desktop-qt/qml/WindowResizeHandles.qml` completed with exit code 0 after the title-bar refinement; it still reports the existing unresolved `Suncode.Runtime` import and unqualified-access warnings.
- `node .agents/skills/impeccable/scripts/detect.mjs --json --scope layout apps/desktop-qt/qml/ProjectWindow.qml apps/desktop-qt/qml/WindowStateController.qml apps/desktop-qt/qml/WindowResizeHandles.qml` returned `[]`.
- `git diff --check` passed after the title-bar refinement.
- Offscreen startup of `apps/desktop-qt/build/suncode-desktop` reached the Qt event loop with no QML runtime output before manual interruption after the title-bar refinement.
- `cmake --build apps/desktop-qt/build -j2` passed after making settings single-instance modal.
- `/Users/shuyi/Softwares/qt/6.11.1/macos/bin/qmllint apps/desktop-qt/qml/ProjectHub.qml apps/desktop-qt/qml/ProjectWindow.qml apps/desktop-qt/qml/GlobalSettings.qml` completed with exit code 0 after the settings-modal refinement; it still reports the existing unresolved `Suncode.Runtime` import and broad unqualified-access warnings.
- `node .agents/skills/impeccable/scripts/detect.mjs --json --scope layout apps/desktop-qt/qml/ProjectHub.qml apps/desktop-qt/qml/ProjectWindow.qml apps/desktop-qt/qml/GlobalSettings.qml` returned `[]`.
- `git diff --check` passed after the settings-modal refinement.
- Offscreen startup of `apps/desktop-qt/build/suncode-desktop` reached the Qt event loop with no QML runtime output before manual interruption after the settings-modal refinement.
- `cmake --build apps/desktop-qt/build -j2` passed after adding the ProjectWindow `Command+1` navigation shortcut.
- `/Users/shuyi/Softwares/qt/6.11.1/macos/bin/qmllint apps/desktop-qt/qml/ProjectWindow.qml` completed with exit code 0 after adding the navigation shortcut; it still reports the existing unresolved `Suncode.Runtime` import and unqualified-access warnings.
- `node .agents/skills/impeccable/scripts/detect.mjs --json --scope layout apps/desktop-qt/qml/ProjectWindow.qml` returned `[]`.
- `git diff --check` passed after adding the navigation shortcut.
- Offscreen startup of `apps/desktop-qt/build/suncode-desktop` reached the Qt event loop with no QML runtime output before manual interruption after adding the navigation shortcut.
- `cmake --build apps/desktop-qt/build -j2` passed after adding the Open Recent Project submenu and project-window reuse logic.
- `node .agents/skills/impeccable/scripts/detect.mjs --json --scope layout apps/desktop-qt/qml/ProjectWindow.qml apps/desktop-qt/qml/ProjectHub.qml` returned `[]` after the Open Recent Project submenu change.
- `git diff --check` passed after the Open Recent Project submenu change.
- The macOS crash report identified `RuntimeClient::projectsChanged()` reevaluating the recent-project menu `visible` binding in `QQuickMenuPrivate::setNativeMenuVisible()` as the startup segmentation-fault path.
- `cmake --build apps/desktop-qt/build -j2` passed after keeping the native recent-project submenu structurally visible and binding only its enabled state.
- `/Users/shuyi/Softwares/qt/6.11.1/macos/bin/qmllint apps/desktop-qt/qml/ProjectWindow.qml` completed with exit code 0 after the crash fix; existing standalone import and unqualified-access warnings remain.
- `node .agents/skills/impeccable/scripts/detect.mjs --json --scope layout apps/desktop-qt/qml/ProjectWindow.qml` returned `[]` after the crash fix.
- `git diff --check` passed after the crash fix.

## Residual risks

- Runtime-populated states not reachable without a configured local runtime require source-level review if unavailable during validation.
- Physical macOS dragging from maximized state and transparent edge/corner resizing need a manual pointer pass on the live window because offscreen Qt cannot exercise native window-manager move/resize loops.
- Physical `Command+,`, Esc, and modal blocking behavior need a manual pass on the live macOS window because offscreen Qt cannot exercise platform focus routing.
- Physical `Command+1` shortcut routing needs a manual pass on the live macOS window because offscreen Qt cannot exercise platform keyboard dispatch.
- Menu hover/keyboard navigation for the new Open Recent Project submenu still benefits from a live macOS pass because Qt's native menubar behavior is easiest to verify in the running app.

- Native macOS fullscreen transition and menu-bar suppression still benefit from a live pointer pass because Qt's offscreen platform cannot exercise the real system fullscreen space.
- The exact project-window startup regression still needs one physical macOS pass because Qt's offscreen platform does not instantiate the native menu-bar path that crashed.

## Inset shell verification

- `cmake --build apps/desktop-qt/build -j2` passed after the unified background and inset-card shell change.
- `cmake --build apps/desktop-qt/build --target all_qmllint -j2` completed with exit code 0; existing repository-wide import-resolution and unqualified-access warnings remain.
- `node .agents/skills/impeccable/scripts/detect.mjs --json --scope layout apps/desktop-qt/qml/app/ProjectWindow.qml` returned `[]`.
- Offscreen startup reached the Qt event loop without QML runtime output before manual interruption.
- `git diff --check` passed.
- Native top/left dragging, embedded-control hit testing, and the default/minimum-size visual pass still require interaction with a visible macOS project window.

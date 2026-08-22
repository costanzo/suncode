# Changes

## Source

- Added `apps/desktop-avalonia/` targeting .NET 10 and Avalonia 12.1.
- Added a hand-written C# P/Invoke adapter with UTF-8 ownership, SDK envelope parsing, async calls, a reference-counted process-wide runtime handle, deterministic subscription disposal, and UI-thread event delivery.
- Added observable project, session, model, credential, message, activity, checkpoint, Git, diff, approval, diagnostics, usage, and theme projections.
- Added the project hub, independent project windows, responsive project workbench, composer, approval/review inspector, Git drawer, application-modal settings, in-window input and confirmation dialogs, macOS project menu, dark/light theme, and compact-width behavior.
- Split the Avalonia desktop surface into focused `ProjectHub`, `ProjectWorkspace`, `ChatArea`, `ProjectSidebar`, `AgentSidebar`, and `GitViewer` controls while keeping window lifecycle, native menus, dragging, resizing, and full-screen behavior in `MainWindow`.
- Consolidated the desktop implementation in Avalonia and removed the obsolete alternate client.
- Closed the macOS parity gaps for threshold-based project-window dragging, traffic-light hover/press assets, animated native `NSWindow` full screen through Avalonia's tracked window state, Dock/window icons, and the Hub border/elevation treatment.
- Restored native macOS shadows for the hub, project, and settings windows by using border-only native decorations on macOS while retaining the custom title chrome and decoration-free behavior on other platforms.
- Added theme-specific 0.5-DIP low-contrast window chrome borders so the native shadow carries top-level elevation without weakening internal panel and control boundaries or shifting content geometry.
- Added macOS safe-area metadata and full-size-content window configuration so packaged full-screen windows can extend through the display's menu-bar/camera area instead of being compatibility-letterboxed.
- Routed project-window full-screen transitions through Avalonia's tracked `WindowState.FullScreen` path, which invokes native Cocoa full screen on macOS, and removed the redundant direct Objective-C toggle that could report success without entering a full-screen space after border-only decorations were enabled.
- Preserved the 4-DIP horizontal and 6-DIP vertical content insets in full screen so gutter controls and top/bottom chrome keep their normal optical spacing without carrying over the restored window's shadow margin.
- Applied restored-window chrome before the native exit-full-screen transition, kept one transparent-capable composition surface throughout the lifecycle, and removed manual full-screen and restored-frame overrides so Cocoa alone owns the animated geometry without a duplicate-window afterimage.
- Moved the project chrome inset from an external margin to internal padding so the canvas reaches the visible window edge and each 24px gutter control has the same 5px optical spacing to the window edge and its adjacent sidebar.
- Restored footer alignment by vertically centering every Git summary label alongside the already-centered branch icon.
- Compressed the project footer toolbar from 24px to 14px while preserving its icon, label, and session-summary alignment.
- Split the workspace gutters by responsibility: project navigation remains at the upper left, the agent sidebar remains at the upper right, and the Git drawer control now anchors at the lower left.
- Unified the three gutter control states so closed panels have no border, while open panels show an active surface, accent border, and accent icon.
- Kept a transparent 1px gutter border in the closed state so opening a panel changes color without resizing the icon content area.
- Made the footer Git branch and change summary read-only and removed its duplicate branch icon, leaving the lower-left gutter as the sole Git drawer toggle.
- Increased Git diff density by removing inherited list-item padding and fixing code rows to a compact 12px line box with correspondingly smaller code and line-number text.
- Raised compact Git diff code text to 12px and line numbers to 10px after readability review while retaining the 12px row rhythm.
- Added 1px vertical padding around each 12px Git diff line box, producing a compact 14px row without crowding adjacent code lines.
- Increased Git diff row spacing to 2px above and below the 12px line box, resulting in a more readable 16px row.
- Expanded the Git diff text line box to 14px and the padded row to 18px so monospace descenders render without clipping.
- Split the Git viewer's rounded clipping surface from a top-layer non-interactive outline so opaque drawer content can no longer cover the bottom corner borders.
- Removed the ChatArea composer TextBox's accent focus outline with a local focus style and Fluent template resource override while preserving the caret and all other input focus behavior.
- Reduced the ChatArea composer model selector and turn action buttons to compact 24px controls with roughly half their previous visual area.
- Replaced the disabled composer shown without a selected session with a centered empty state, while keeping the message list and composer available whenever a session is selected.
- Reduced the composer TextBox's reserved bottom action space from 40px to 30px, returning 10px of vertical room to message input without changing the controls.
- Moved the model and turn action controls into the composer surface, added tunnel-routed Enter submission with Shift+Enter newline behavior, and rendered assistant responses as Markdown during both streaming and completed states.
- Extracted the chat composer into an independent `ChatInput` control that owns its input, model selection, submission, cancellation, and keyboard behavior while preserving the existing layout and bindings.
- Kept the chat composer editable while an agent turn is active, with the visible turn action remaining the existing cancel control.
- Optically aligned the assistant-message copy glyph with the Markdown text edge while preserving its 26-DIP button target.
- Kept tool-call-only assistant records in runtime storage and trace diagnostics while omitting their empty text rows from snapshot and live conversation projection.
- Preserved the composer's floating shadow after component extraction by reserving shadow-rendering space outside the unchanged 102px input surface.
- Grouped the desktop views and their namespaces by product module under `Shell`, `Projects`, `Chat`, `Review`, `Settings`, and `About` so each surface can evolve without adding to one flat view directory.
- Added a publish-time macOS `.app` bundle with `Info.plist` and the desktop icon resource.
- Upgraded the core Avalonia framework, desktop backend, Fluent theme, and Inter font packages to `12.1.1`, migrated SVG rendering to `Svg.Controls.Skia.Avalonia 12.0.0.15`, and migrated Markdown rendering to the Avalonia 12-compatible `LiveMarkdown.Avalonia 2.3.2` package.

## Contracts and generated artifacts

- No SDK payload changed. The Rust library now emits `cdylib` in addition to `rlib` and `staticlib`.

## Configuration and persistence

- No configuration or persistence format changed.

## Tests

- Existing Rust workspace tests cover 43 runtime and operation behaviors.
- Compiled Avalonia bindings and native loading were exercised by Debug/Release builds and startup checks.
- Multi-window runtime sharing, recent-project activation, dialog behavior, settings modality, default/minimum geometry, and Git drawer behavior were manually verified on macOS.
- Added desktop regression coverage for filtering tool-call-only assistant messages from snapshot and live conversation projection and for combining multiple visible text parts.
- Added turn-scoped chat timelines that retain live assistant progress and tool activity in occurrence order, collapse those process items when the turn ends, and restore them from normalized snapshots; only the final assistant response exposes copy, with a control before the process region to expand or collapse retained work below it.

## Documentation

- Updated current product, architecture, contributor, design, SDK, contract, feature, and decision records.
- Updated product and architecture records to describe Avalonia as the only desktop client.

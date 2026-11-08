# Diff Reviewer

Implemented and focused-tested in the .NET 10 Avalonia desktop client. Installed OS title-bar and window-manager smoke verification remains pending.

## Ownership and entry

Diff Reviewer presents desktop-owned read-only system Git comparisons and Rust/SDK-owned retained turn comparisons. The expand action immediately after Refresh in Source Control opens one non-modal Git reviewer per project using `Show(owner)`. The action requires a loaded repository. Opening copies the current Git range and selected file into the reviewer. Its later range and file selections are independent. Reopening activates the existing window and restores it if minimized.

The expand action immediately before Close in Turn Diff opens a separate reviewer for the loaded turn and selected file. It requires a ready, nonempty turn diff. The initial document uses the drawer's retained SDK snapshot, and Refresh calls `GetTurnDiffAsync` for the same captured session/turn identifiers. The toolbar identifies Turn changes, the panes identify Before turn and After turn, and Git scope/branch controls are hidden. Changing turn/session or clearing/disposing the project closes the turn reviewer. Current repository state and live files do not supply historical content.

`GitReviewViewModel` owns the per-project Git reviewer model, and `TurnDiffViewModel` owns the selected-turn reviewer model. Closing either reviewer releases it for later reopening. Project disposal or owner-window closure closes the window, cancels pending reads, and disposes the TextMate installations and document presentation. Superseded refresh or file-selection requests are cancelled and their results are ignored.

## Comparison behavior

- The Explorer-style tree contains changed files and their parent directories, with shared file icons, folder expansion, selection states, and semantic Git status markers. All compares HEAD to working tree, Staged compares HEAD to index, and Unstaged compares index to working tree. Conflicted Unstaged content uses the existing HEAD-to-working-tree fallback when an ordinary two-way patch is unavailable, and its before-pane heading identifies HEAD.
- Selecting a file starts at its first contiguous difference block. Previous and next navigate within the selected file and disable at the endpoints. Cached document-line anchors and binary search track the current block during manual scrolling. The panes synchronize vertical offsets and retain separate horizontal scrolling.
- Unchanged blocks longer than three lines have shared expand/collapse state. Fold labels are vertically centered within the 32-DIP strip, aligned with the code column. The read-only AvaloniaEdit documents retain their source with synthetic fold-header and alignment rows. Native visual collapse keeps hidden context available to TextMate grammar state. The margins render original source line numbers. Difference lines use added/deleted surfaces, and inline changes use stronger highlights. The current difference has one continuous 1-DIP outline around its complete block and gutter.
- `DiffReviewDocument` derives alignment and contiguous difference blocks from the source's edit script. Retained turn hunks adapt to the same presentation records, preserving their historical source line numbers. DiffPlex 1.9.0 computes character-level inline ranges for paired lines up to 512 characters. Longer lines use a bounded common-prefix/common-suffix range. Copy maps ordinary and rectangular selections back to source, strips synthetic fold headers and padding, and preserves missing final newlines.
- Binary metadata and PDF, Office, and OpenDocument extensions show an explicit Cannot compare this file message before per-file reading or text projection. A binary diff response also bypasses projection. The editors clear their text and syntax installation, and difference navigation disables. Plain-text and empty panes do not install TextMate. File changes clear stale selections before replacing the text, and the projection stays detached until native folding can be installed. Renderers use existing visual lines without triggering layout during a render pass.
- Refresh retains the selected file, folder expansion, and current difference index when valid. A removed selection falls back to the first remaining changed file, and a shorter comparison clamps the difference index. Loading, retryable failures, empty ranges, no selection, binary content, no textual differences, truncated diffs, and truncated status lists have explicit presentation states.

## Bounded reads

The reviewer uses the existing `IGitService.DiffAsync` with optional `includeUnchanged`. That mode requests `--unified=12000` while retaining the existing 2 MiB patch, 12,000 structured-line, 2,000-hunk, and 2,000-status-file limits. When context consumes the patch budget, the service retries with compact context and marks the result limited so a long unchanged prefix does not hide the actual edits. The fallback remains subject to the same bounded read limits.

Git execution continues through the desktop's bounded read-only service with structured arguments, disabled configured helpers and filters, project-relative paths, process deadlines, and cancellation. Historical comparisons continue through the retained SDK turn-diff API.

## Visual contract

Root `DESIGN.md` and `#/projects/desktop/workspace/diff-reviewer` define the composition. The fully system-decorated window defaults to 1200 × 680 DIP with an 820 × 480 DIP minimum, fitted to the available screen when necessary. The changed-file tree starts at 240 DIP and resizes from 180 to 360 DIP while reserving 280 DIP per comparison pane at supported sizes. Code rows are 22 DIP, context controls are 32 DIP, toolbars are 36 DIP, and version headings are 30 DIP. The tree and comparison fill the client area's bottom edge. Shared theme resources and live global localization cover light/dark and `en-US`/`zh-CN`.

## Source and verification

The implementation is concentrated in `Controls/DiffCodePane.cs`, `Models/UiModels/Review/DiffReviewDocument.cs`, `ViewModels/Review/DiffReviewerViewModel.cs`, and `Views/ProjectWorkspace/Review/DiffReviewerWindow.axaml` with its code-behind, under `apps/desktop-avalonia/`. `GitViewer` and `TurnDiffViewer` supply entry and window activation, their feature view models own lifetime, and `Infrastructure/Git/GitService.cs` supplies bounded Git context reads.

The 51 focused tests across `GitServiceTests`, `GitReviewViewModelTests`, `TurnDiffViewModelTests`, `DiffReviewDocumentTests`, `DiffReviewerViewModelTests`, `DiffReviewerWindowTests`, and `LocalizationTests` pass. They cover actual Git comparison ranges and retained context, near-end edits in large files, helper/filter and symlink handling, source copy with rectangular selections and missing final newlines, navigation across long folded context, reviewer entry/reopening/closure, default/minimum sizing, both themes, and both interface locales. Regression cases cover retained turn content and source line numbers, PDF/Excel paths with false binary metadata, binary diff responses, skipped binary reads, and grammar-backed text → binary → plain text → grammar-backed text transitions.

An additional 56 Explorer, editor, UI-state, merged-window, question-window, menu-shortcut, and title-bar regression tests pass with the same temporary compile exclusions.

The production desktop build passes with zero warnings. The design-system build passes with its existing bundle-size warning. The normal test-project compile is blocked by pre-existing errors in `LiveEventRoutingTests`, `WorkspaceLayoutTests`, `DesktopMenuControllerTests`, `SessionSnapshotProjectionTests`, and `SdkTypedModelTests`. The focused run used temporary local MSBuild exclusions for those files.

Headless native Skia client-area captures cover light and dark at 1200-DIP and 820-DIP widths, including retained-turn text and unsupported PDF states. The design-system browser check covers PDF and Excel messages and source switching. Actual OS decorations, activation, ownership, resizing, and window-manager lifecycle still require installed-platform smoke verification.

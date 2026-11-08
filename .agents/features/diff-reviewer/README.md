# Diff Reviewer

Implemented and focused-tested in the .NET 10 Avalonia desktop client. Installed OS title-bar and window-manager smoke verification remains pending.

## Ownership and entry

Diff Reviewer is a desktop-owned read-only system Git capability. The expand action immediately after Refresh in Source Control opens one non-modal reviewer per project using `Show(owner)`. The action requires a loaded repository. Opening copies the current Git range and selected file into the reviewer. Its later range and file selections are independent. Reopening activates the existing window and restores it if minimized.

`GitReviewViewModel` owns the per-project reviewer model. Closing the reviewer releases it for later reopening. Project disposal or owner-window closure closes the window, cancels pending reads, and disposes the TextMate installations and document presentation. Superseded refresh or file-selection requests are cancelled and their results are ignored.

## Comparison behavior

- The Explorer-style tree contains changed files and their parent directories, with shared file icons, folder expansion, selection states, and semantic Git status markers. All compares HEAD to working tree, Staged compares HEAD to index, and Unstaged compares index to working tree. Conflicted Unstaged content uses the existing HEAD-to-working-tree fallback when an ordinary two-way patch is unavailable, and its before-pane heading identifies HEAD.
- Selecting a file starts at its first contiguous difference block. Previous and next navigate within the selected file and disable at the endpoints. Cached document-line anchors and binary search track the current block during manual scrolling. The panes synchronize vertical offsets and retain separate horizontal scrolling.
- Unchanged blocks longer than three lines have shared expand/collapse state. The read-only AvaloniaEdit documents retain their source with synthetic fold-header and alignment rows. Native visual collapse keeps hidden context available to TextMate grammar state. The margins render original source line numbers. Difference lines use added/deleted surfaces, and inline changes use stronger highlights.
- `DiffReviewDocument` derives alignment and contiguous difference blocks from Git's patch. DiffPlex 1.9.0 computes character-level inline ranges for paired lines up to 512 characters. Longer lines use a bounded common-prefix/common-suffix range. Copy maps ordinary and rectangular selections back to source, strips synthetic fold headers and padding, and preserves missing final newlines.
- Refresh retains the selected file, folder expansion, and current difference index when valid. A removed selection falls back to the first remaining changed file, and a shorter comparison clamps the difference index. Loading, retryable failures, empty ranges, no selection, binary content, no textual differences, truncated diffs, and truncated status lists have explicit presentation states.

## Bounded reads

The reviewer uses the existing `IGitService.DiffAsync` with optional `includeUnchanged`. That mode requests `--unified=12000` while retaining the existing 2 MiB patch, 12,000 structured-line, 2,000-hunk, and 2,000-status-file limits. When context consumes the patch budget, the service retries with compact context and marks the result limited so a long unchanged prefix does not hide the actual edits. The fallback remains subject to the same bounded read limits.

Git execution continues through the desktop's bounded read-only service with structured arguments, disabled configured helpers and filters, project-relative paths, process deadlines, and cancellation. This delivery is confined to desktop presentation and its Git service.

## Visual contract

Root `DESIGN.md` and `#/projects/desktop/workspace/diff-reviewer` define the composition. The fully system-decorated window defaults to 1200 × 680 DIP with an 820 × 480 DIP minimum, fitted to the available screen when necessary. The changed-file tree starts at 240 DIP and resizes from 180 to 360 DIP while reserving 280 DIP per comparison pane at supported sizes. Code rows are 22 DIP, context controls are 32 DIP, toolbars are 36 DIP, and version headings are 30 DIP. The tree and comparison fill the client area's bottom edge. Shared theme resources and live global localization cover light/dark and `en-US`/`zh-CN`.

## Source and verification

The implementation is concentrated in `Controls/DiffCodePane.cs`, `Models/UiModels/Review/DiffReviewDocument.cs`, `ViewModels/Review/DiffReviewerViewModel.cs`, and `Views/ProjectWorkspace/Review/DiffReviewerWindow.axaml` with its code-behind, under `apps/desktop-avalonia/`. `GitViewer` supplies the entry and window activation, `GitReviewViewModel` owns the lifetime, and `Infrastructure/Git/GitService.cs` supplies bounded context reads.

The 41 focused tests across `GitServiceTests`, `GitReviewViewModelTests`, `DiffReviewDocumentTests`, `DiffReviewerViewModelTests`, `DiffReviewerWindowTests`, and `LocalizationTests` pass. Coverage includes actual Git comparison ranges and retained context, near-end edits in large files, helper/filter and symlink handling, source copy with rectangular selections and missing final newlines, navigation across long folded context, one reviewer per project, project closure, default/minimum sizing, both themes, and both interface locales.

An additional 57 Explorer, editor, UI-state, merged-window, question-window, menu-shortcut, and title-bar regression tests pass with the same temporary compile exclusions.

The production desktop build passes with zero warnings. The design-system build passes with its existing bundle-size warning. The normal test-project compile is blocked by pre-existing errors in `LiveEventRoutingTests`, `WorkspaceLayoutTests`, `DesktopMenuControllerTests`, `SessionSnapshotProjectionTests`, and `SdkTypedModelTests`. The focused run used temporary local MSBuild exclusions for those files.

Headless native Skia client-area captures cover light and dark at 1200-DIP and 820-DIP widths. Actual OS decorations, activation, ownership, resizing, and window-manager lifecycle still require installed-platform smoke verification.

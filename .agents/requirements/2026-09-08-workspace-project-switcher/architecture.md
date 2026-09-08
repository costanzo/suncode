# Architecture

## Current state

ProjectHub and Workspace already call application-level project opening methods. `App` owns independent Workspace windows and duplicate-window activation. Workspace also exposes project actions through a native menu.

## Proposed design

Add an Avalonia menu trigger to the leading Workspace title-bar project label. Populate it from the same recent-project collection used by ProjectHub and route its actions through the existing Workspace window and application project-opening methods.

## Boundaries and dependencies

- Avalonia owns menu presentation and transient open/focus state.
- Existing application coordination owns folder selection, new windows, and duplicate activation.
- The Rust SDK remains the source of project records through existing managed APIs; no new persistence contract is introduced.

## Data and control flow

`Open project` invokes the existing folder picker, then the application opens or activates the selected project. A recent-project row passes its existing project item to the same application-level window coordination path.

## Security and failure handling

The project boundary and path validation remain in existing SDK/application flows. Invalid or unavailable paths must surface the same bounded error state used by ProjectHub and must not close the current Workspace.

## Compatibility and migration

No data migration or protocol change is required. The native project menu remains available.

## Risks and rollback

The primary risk is conflicting title-bar drag behavior. The trigger and popup must be excluded from drag initiation. The UI can be removed without changing stored data or SDK contracts.

## Open questions

- None pending beyond design approval.

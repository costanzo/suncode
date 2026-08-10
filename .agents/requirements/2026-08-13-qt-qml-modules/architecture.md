# Architecture

## Current state

The Qt client used one flat `apps/desktop-qt/qml/` directory containing application shells, feature panels, shared controls, theme tokens, and window helpers.

## Proposed design

Keep one `Suncode.Desktop` QML module, but organize its source tree into three ownership layers:

```text
app -> features -> shared
```

`app/` composes the product and owns window lifecycle. `features/` owns user-facing workflows. `shared/` contains reusable presentation primitives and must not import feature modules.

## Boundaries and dependencies

- `features/project` owns project/session navigation.
- `features/conversation` owns the conversation surface.
- `features/review` owns activity, approvals, and undo review.
- `features/settings` owns global settings.
- `shared/components`, `shared/navigation`, `shared/theme`, and `shared/window` are reusable infrastructure.
- Runtime state remains exposed by the existing `RuntimeClient`; this change does not alter ownership boundaries described in `.agents/ARCHITECTURE.md`.

## Compatibility and migration

QML type names remain unchanged. Relative directory imports replace implicit same-directory visibility, and dynamic creation uses resource URLs so it is independent of the source file's directory.

## Risks and rollback

The main risk is an incorrect QML resource or import path. CMake compilation and QML cache generation exercise those paths; reverting this delivery only requires restoring the previous QML file locations and resource URL.

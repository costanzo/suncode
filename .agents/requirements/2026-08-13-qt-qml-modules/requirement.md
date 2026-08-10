# Requirement

## Background

The Qt client QML sources were stored in one flat directory. That made ownership unclear and made the feature surface harder to extend safely.

## Goals

- Group QML by application shell, product feature, and shared presentation responsibility.
- Preserve existing Qt behavior, runtime bindings, and the single QML module URI.
- Make future additions discoverable through a documented directory convention.

## Non-goals

- No visual redesign or new product behavior.
- No Rust SDK, protocol, persistence, or C++ runtime changes beyond the startup resource path.

## Requirements

- Application entry points and window shells live under `qml/app/`.
- Feature panels live under `qml/features/<feature>/`.
- Reusable controls and presentation infrastructure live under `qml/shared/`.
- Cross-directory dependencies use explicit relative imports.
- Dynamically created QML components use stable resource URLs.
- CMake lists QML files by module group.

## Acceptance criteria

- No production QML file remains directly under `qml/`.
- The Qt desktop target builds successfully.
- The project hub starts from its new resource URL and can still create project/settings windows.

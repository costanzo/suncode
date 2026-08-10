# Changes

## Source

- Moved application shells to `qml/app/`.
- Moved project, conversation, review, and settings panels to `qml/features/`.
- Moved controls, navigation helpers, theme tokens, and window helpers to `qml/shared/`.
- Added explicit relative imports between those layers.
- Updated dynamic component URLs and the C++ startup URL to the new QML resource paths.
- Grouped `qt_add_qml_module` entries by ownership in CMake.

## Tests

- `cmake --build apps/desktop-qt/build -j2` passed.
- QML cache generation completed for every moved file.
- `git diff --check` passed.

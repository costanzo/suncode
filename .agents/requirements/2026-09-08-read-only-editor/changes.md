# Changes

## Source

- Design-system Workspace Editor route and read-only editor specimen.
- Rust operations and SDK facade support for bounded project/dependency UTF-8 file reads.
- C ABI and typed C# bindings for `read_project_file`.
- Production Avalonia Explorer selection, central content mode switching, and AvaloniaEdit/TextMate viewer.

## Contracts and generated artifacts

- No generated artifacts. The editor interaction is documented and hand-implemented.

## Configuration and persistence

- No new persistence is planned. Selected file state is transient Workspace UI state.

## Tests

- Design-system build and route/state review.
- Focused Rust boundary/facade tests, C ABI tests, C# DTO tests, editor language tests, and desktop build/test verification.

## Documentation

- Root `DESIGN.md` editor interaction rules plus stable desktop feature and agent specification updates.

# Changes

## Source

- Add Roslyn Language Server to the design-system language-server catalog.
- Add a server-specific C# Settings specimen and direct review route.
- Add the Rust catalog adapter, project-scoped runtime mapping, root glob matcher, detection, launch arguments, and workspace settings.
- Add typed blocking Rust, C ABI, C# SDK, and Avalonia Settings support.

## Contracts and bindings

- Bump the native ABI from 22 to 23.
- Add `configure_roslyn_language_server` across the hand-written Rust, C, and C# contracts.
- Extend language-server DTOs with an optional typed Roslyn configuration.

## Configuration and persistence

- Persist project-scoped Roslyn records as language-server configuration version 6.
- Keep existing versions 1-5 readable through optional typed fields.
- Keep environment values write-only and expose key names in read DTOs.

## Tests

- Run the design-system production build and targeted formatting checks.
- Review catalog and detail layouts at desktop and constrained widths in both themes.
- Exercise custom project-limit and solution states plus dirty actions.
- Cover root glob matching, persisted configuration mapping, redaction, project isolation, typed C# serialization, and desktop projection.
- Complete real stdio initialization against the installed Roslyn Language Server with production launch arguments.

## Documentation

- Update `DESIGN.md`, the stable language-server feature, product status, SDK contract, Settings review README, and review guide.

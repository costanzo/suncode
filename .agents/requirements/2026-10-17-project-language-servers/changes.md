# Changes

## Source

- Replace the design-system free-form LSP list and editor with a project catalog and jdtls detail page.
- Add project-aware server selection to `suncode-lsp` and core.
- Add typed jdtls catalog/configuration behavior to the Rust SDK and C/C# bindings.
- Replace the Avalonia free-form editor with the catalog and in-panel jdtls detail flow.

## Contracts and generated artifacts

- ABI 19 adds typed project-scoped jdtls configuration.
- Existing five model-facing semantic tool schemas remain unchanged.

## Configuration and persistence

- Version-2 `language_server` configuration embeds project identity and built-in server kind.
- Legacy version-1 free-form rows remain readable but are ignored by the catalog/runtime.

## Tests

- Added Rust SDK project-isolation, redaction, revision, and startup-failure coverage.
- Updated typed C# serialization, localization, and view-model tests.
- Built and launched the production Avalonia client with an isolated data directory.

## Documentation

- Update `DESIGN.md` and the Desktop Settings review-surface README.

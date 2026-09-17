# Test Plan

## Scope

Global proxy persistence, redaction, validation, HTTP-client application, runtime updates, SDK bindings, and Settings presentation.

## Unit tests

- Proxy URL and bypass validation.
- Password preserve, replace, clear, and read redaction.
- No proxy, system proxy, custom proxy, and bypass builder selection.

## Integration and conformance tests

- Rust facade persistence and live update behavior.
- Provider, WebFetch, and remote MCP proxy routing.
- C and C# contract serialization.

## Regression checks

- Existing HTTPS certificate settings.
- Existing MCP reconciliation and settings behavior.

## Manual checks

- Network Settings modes, dirty state, validation, password state, themes, and constrained width.

## Commands and results

- `cargo test --workspace` in `agent/`: passed.
- `cargo test` in `sdks/rust/`: 19 passed.
- `cargo check` in `sdks/c/`: passed.
- `dotnet build SunCode.Desktop.csproj`: passed with zero warnings.
- Focused `SdkTypedModelTests`: 8 passed.
- Desktop tests excluding the unrelated existing Workspace gutter assertion: 99 passed.
- `npm run build` in `design-system/`: passed.
- Targeted Prettier and Impeccable detector checks: passed.
- `git diff --check`: passed.

## Residual risks

- The full desktop suite retains one unrelated existing failure in `WorkspaceLayoutTests.ResponsiveLayoutSuppressesSecondarySurfacesWithoutChangingUserPreferences`: expected gutter width 26, current implementation returns 34.
- System proxy behavior is limited to reqwest-supported operating-system and standard environment configuration; PAC/WPAD and integrated authentication remain unsupported.

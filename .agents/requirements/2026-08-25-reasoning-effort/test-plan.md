# Test Plan

## Scope

Model capability projection, provider request serialization, native ABI signature, and Avalonia selection state.

## Unit tests

- OpenAI-compatible provider request contains `reasoning_effort` when selected.
- Model catalog round trip retains `supports_reasoning_effort`.
- Unsupported model rejection is enforced by core before provider execution.

## Regression checks

- All `CompletionRequest` and `ModelCapabilities` constructors compile after the contract extension.
- Avalonia reads nested `capabilities.reasoning_effort` from `list_models`.
- `dotnet build ... -p:DesignTimeBuild=true` compiles the managed client and updated P/Invoke signature.

## Commands and results

- `git diff --check` - passed.
- Rust library tests - passed: `suncode-agent` 41, `suncode-db` 41, `suncode-llm` 6, and `suncode-tool` 33.
- `cargo fmt --all -- --check` - passed.
- `dotnet build apps/desktop-avalonia/SunCode.Desktop.csproj --no-restore -p:DesignTimeBuild=true` - passed with 0 warnings and 0 errors.
- `dotnet build apps/desktop-avalonia/tests/SunCode.Desktop.Tests.csproj --no-restore -p:BuildProjectReferences=false` - passed.
- `dotnet test apps/desktop-avalonia/tests/SunCode.Desktop.Tests.csproj --no-build --no-restore` - passed: 44 tests.

## Residual risks

The current catalog supports only the common low/medium/high values. Models with provider-specific levels need an expanded catalog representation before being enabled.

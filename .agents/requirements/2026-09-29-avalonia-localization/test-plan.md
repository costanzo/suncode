# Test Plan

## Scope

Locale selection, persistence, resource fallback, dynamic refresh, and Settings layout.

## Unit tests

- Supported locale catalog and native display names.
- Invalid locale fallback to `en-US`.
- English fallback for missing resource keys.
- English/Chinese resource-key parity.

## Integration and conformance tests

- `ui_locale` is read and written through the generic SDK settings boundary.
- Language changes raise one refresh event and retain selected values by stable locale code.

## Regression checks

- Existing theme switching remains unchanged.
- Existing Settings navigation and model/provider selectors continue to work.
- User content and model output remain unchanged.

## Manual checks

- Switch language with ProjectHub, Settings, and Workspace windows open.
- Restart and verify the selected locale.
- Check minimum Settings width with Chinese labels.

## Commands and results

- `dotnet build apps/desktop-avalonia/SunCode.Desktop.csproj -p:SkipBrowserRuntimeCheck=true --no-restore` — passed.
- `dotnet test apps/desktop-avalonia/tests/SunCode.Desktop.Tests.csproj --no-restore -p:SkipBrowserRuntimeCheck=true` — passed, 126 tests.
- `git diff --check` — passed.

## Residual risks

Any literal user-facing text not migrated will remain English until it is added to the resource catalog.

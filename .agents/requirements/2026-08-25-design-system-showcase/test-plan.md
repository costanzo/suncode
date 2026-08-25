# Test Plan

## Scope

Static design review artifacts and their local references.

## Unit tests

Not applicable; there is no production code path in this delivery.

## Integration and conformance tests

Not applicable; no protocol or runtime contract changed.

## Regression checks

- Confirm all review HTML documents reference `tokens.css`.
- Confirm every image reference resolves under `design/assets/`.
- Confirm the pages contain the required component sections.
- Confirm `index.html` exposes both theme links and has no horizontal overflow in a desktop viewport.

## Manual checks

- Open `index.html`, then both theme HTML files, directly in a browser or local static server.
- Resize to desktop and compact widths.
- Compare dark/light semantic contrast and focus behavior.

## Commands and results

- `git diff --check` - passed.
- `dotnet build apps/desktop-avalonia/SunCode.Desktop.csproj --no-restore -p:DesignTimeBuild=true` - passed with 0 warnings and 0 errors.
- Static reference checks - passed.
- `dotnet test apps/desktop-avalonia-tests/SunCode.Desktop.Tests.csproj --no-restore` - blocked because the current shell cannot resolve `cargo`, which the project invokes to build the embedded Rust SDK.

## Residual risks

Avalonia view-local styles can still drift until each view is migrated to the named semantic resource contract. The new design pages make that drift visible and establish the review baseline.

# Test Plan

## Scope

Design review currently covers the catalog and Roslyn-specific Settings surface. Production adapter, persistence, SDK, and Avalonia tests are deferred until approval.

## Unit tests

- None for the fixture-only React specimen.

## Integration and conformance tests

- Build the complete design-system application.
- Confirm the direct Roslyn route resolves without runtime errors.

## Regression checks

- Confirm the existing four server-specific detail dispatch paths remain unchanged.
- Confirm the catalog reports five supported servers.
- Confirm Save and Reset are disabled for an unchanged draft and enabled after editing.
- Confirm custom project limit and custom solution fields appear only for their modes.

## Manual checks

- Review desktop and constrained-width layouts.
- Review light and dark themes.
- Exercise project loading, solution selection, restore, source generators, diagnostics, advanced disclosure, detection, enablement, save, reset, and retry states.

## Commands and results

- `npm run build` from `design-system/`: passed (268 modules transformed).
- Targeted Prettier check: passed.
- `git diff --check`: passed.
- Headless browser review: passed for the five-entry catalog, Roslyn direct route, light and dark themes, and 760px constrained width.
- Interaction review: Save and Reset start disabled; custom project-limit and custom solution fields appear on selection; Save enables after a draft change.
- Responsive review: the Settings panel and detail content have equal client/scroll widths at 760px with no horizontal overflow.

## Residual risks

- The design fixture does not prove production Roslyn compatibility, MSBuild project loading, or .NET SDK discovery.

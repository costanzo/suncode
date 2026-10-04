# Changes

## Source

- Add a Web remote-control review project and styles under `design-system/src/projects/web/`.
- Register `/projects/web` and child routes in the design-system navigation and router.
- Add a Web card to the Projects module index.
- Update the design authority with the Web remote-control composition and security rules.

## Contracts and generated artifacts

- No contract changes and no generated artifacts.

## Configuration and persistence

- None. Fixture state is in-memory review data only.

## Tests

- Run `npm run build` in `design-system/`.
- Run `git diff --check` from the repository root.
- Manually inspect the Web route in light/dark and constrained widths.

## Documentation

- Keep this requirement package as the active design-only delivery record until `apps/web` exists.

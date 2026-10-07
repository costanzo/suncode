# Test Plan

## Scope

The public home page, metadata routes, responsive navigation, external GitHub links, and production build.

## Unit tests

No unit tests are required for static content in the first pass.

## Integration and conformance tests

- TypeScript compilation.
- Next.js production build.
- Generated sitemap and robots routes compile with the application.

## Regression checks

- Existing repository applications remain outside the website boundary.
- `git diff --check` reports no whitespace errors.

## Manual checks

- Inspect desktop and mobile rendering.
- Verify keyboard focus and mobile navigation behavior.
- Verify the primary GitHub CTA destination.

## Commands and results

`pnpm typecheck` passed. `pnpm build` passed. `node .agents/skills/impeccable/scripts/detect.mjs --json` returned no findings. `git diff --check` passed.

## Residual risks

Copy should be revisited when the current product scope changes.

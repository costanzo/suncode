# Requirement

## Background

SunCode needs a small public website at `suncode.ai` that explains the current product and directs developers to the public GitHub repository.

## Goals

- Present the current Phase 1 product truthfully.
- Make reviewable machine access the memorable product distinction.
- Give visitors a direct path to https://github.com/costanzo/suncode.
- Provide a deployable Next.js SSR surface under `website/` for Vercel.

## Non-goals

- Pricing, customer stories, benchmarks, accounts, forms, or hosted execution.
- Marketing claims for deferred Web, TUI, IDE plugin, or cloud surfaces.
- Replacing the existing remote-control client under `apps/web/`.

## Requirements

- Use Next.js App Router, TypeScript, and server-rendered page content.
- Include responsive navigation, product overview, current capabilities, workflow, and GitHub CTA.
- Include metadata, Open Graph metadata, sitemap, robots, and the `suncode.ai` canonical URL.
- Reuse the existing SunCode brand mark and visual language.
- Keep client-side JavaScript limited to necessary navigation behavior.

## Edge cases

- Long links and narrow viewports must not overflow.
- Reduced-motion users must receive the same content without animation.
- GitHub links must remain valid if the page is rendered statically or cached by Vercel.

## Acceptance criteria

- `pnpm build` succeeds from `website/`.
- The page identifies SunCode within the first viewport and links to the supplied repository.
- The page does not claim deferred or unimplemented product surfaces as available.
- Desktop and mobile layouts preserve readable type, navigation, and CTA access.

## Open questions

- None for the first public website pass.

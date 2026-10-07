# Architecture

## Current state

`apps/web/` is the remote-control client and uses Vite. The repository has no public marketing site.

## Proposed design

Create an independent Next.js App Router application in `website/`. The home page is server-rendered and uses static content. A small client component owns the mobile navigation disclosure.

## Boundaries and dependencies

- The website reads no agent database and does not call SunCode APIs.
- Product copy is derived from `.agents/PRODUCT.md`, `.agents/ARCHITECTURE.md`, and implemented feature records.
- The website may copy the approved logo into its own public asset directory.
- The website is deployable as a Vercel project with `website/` as its root directory.

## Data and control flow

The browser requests the Next.js route, Next renders the static page and metadata on the server, and external navigation goes directly to the GitHub repository.

## Security and failure handling

No credentials or user data are handled. External links use HTTPS. The page includes accessible focus states and reduced-motion behavior.

## Compatibility and migration

The site is independent of the current `apps/web/` Vite application and does not change desktop or Rust runtime behavior.

## Risks and rollback

The website can be removed as one directory without affecting production agent clients. The main risk is copy drift as product scope changes; the requirement package identifies the source records.

## Open questions

None for the first pass.

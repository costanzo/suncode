# SunCode official website

The public product information site uses Next.js App Router and TypeScript. It is separate from the deferred Web agent client.

## Pages

- `/`: product overview, user-controlled illustrative workflow, capabilities, and working loop.
- `/features`: the complete capability guide, searchable by text and availability, with category anchors and scope disclosures.
- `/updates/v1/{channel}/{target}/latest.json`: the existing signed component metadata route. See [updates/README.md](updates/README.md); current sample archive values are not download releases.

## Feature content

`lib/feature-catalog.json` is the public editorial mapping of all `.agents/features/*/README.md` records. Each public entry uses the owning feature directory name as its stable ID and technical-notes destination. Related capabilities are grouped for readers, while the coverage test requires every durable feature record exactly once. Engineering tooling is labeled separately.

When a feature changes, update its public description, status, behavior, and scope together. Review `.agents/features/WEBSITE-CLAIMS.md` before adding claims. An implementation status does not establish release availability. Browser/Computer Use, mobile, package installation, provider compatibility, and checkpoint undo need their explicit qualifications.

The guide is server-rendered before client hydration; search and availability filtering are local to the page. Text search matches titles, descriptions, behavior details, scope notes, and feature IDs. Filters combine, category links show only matching groups, and the empty state clears both filters. Scope uses native `details`/`summary` disclosures. There is no account, remote data source, or telemetry added by the guide.

## Design and assets

Root `DESIGN.md` and the design browser's `/projects/website` route own appearance and interaction. Light paper/slate and dark graphite/silver themes apply to the complete site and illustrative workbench. A header toggle remains visible on mobile, defaults to the system preference, and persists manual choices in localStorage. The head bootstrap applies the choice before first paint; blocked storage still allows switching for the visit. Shared navigation, feature rows, workbench examples, native disclosures, empty recovery, mobile layouts, and reduced-motion behavior are specified there.

Fonts under `public/fonts/` are self-hosted copies of the repository's Noto Sans Regular (400), SemiBold (600), Bold (700), and JetBrains Mono Regular (400) assets. Their SIL Open Font Licenses are included. JetBrains Mono is reserved for code. Logos reuse the approved optical SunCode marks. The three workbench stages are user-controlled pressed-state buttons. `app/components/WorkbenchDemo.tsx` imports the shared design-system website preview, which composes the existing Desktop Workspace window with Ubuntu chrome and local sample data. The shared component and styles must be available beside `website/` when building from a repository checkout. Next.js resolves the shared sources from the repository root and uses the website’s React installation. Preserve the adjacent `design-system/src/` and desktop caption assets in source-based deployment inputs.

The preview supports panel toggles, example files, approval feedback, and diff inspection. Its label and footer identify sample data. Both themes and container-based narrow layouts use the same source in the public page and review route. The window height and drawer adaptations are specified in the design-system website tokens.

## Development and checks

```sh
pnpm install --frozen-lockfile
pnpm dev
pnpm build
pnpm typecheck
pnpm test:features
pnpm test:theme
pnpm test:updates
```

The feature tests verify complete one-to-one coverage and valid groups, availability, behavior, and scope. Theme tests cover first-paint system preference, saved-choice precedence, invalid values, and blocked storage. The update tests protect catalog serving and publication behavior. Check the home and features routes at desktop and mobile widths in both themes, including persistence across navigation/reload, workflow switching, search/filter combinations, empty recovery, scope disclosure, navigation dismissal, focus, and horizontal overflow. Build the design-system specimen as part of visual changes.

Production deployment uses the existing website infrastructure and is separate from local implementation and validation.

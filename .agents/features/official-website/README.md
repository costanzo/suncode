# Official Website

**Status:** Implemented and build-verified

`website/` is the public, server-rendered Next.js site for `suncode.ai`. It explains the current product and links to the public GitHub repository. Its copy describes only implemented product capabilities; deferred Web, TUI, IDE, and hosted execution surfaces are not presented as available clients.

The site uses the Next.js App Router and TypeScript, includes responsive navigation, canonical/Open Graph metadata, sitemap and robots routes, and reuses the SunCode brand assets. It is separate from the design-system Web review specimen and from the deferred remote-control Web client.

Build and verify from `website/` with `pnpm build`.

The public `/updates/v1/stable/{target}/latest.json` route serves Desktop component catalogs from checked-in files with JSON content types, ETag/304 handling, and bounded HTTP caching. The six target files are normally served signed 0.1.0 catalogs with First release notes. Per the approved development setup, package URLs/sizes/hashes are sample values; actual component archives do not yet exist. Desktop catalog checking is enabled against `https://www.suncode.ai`, with both apex and www download origins allowed. Valid signatures do not establish usable downloads or installed-package conformance. An offline Node/TypeScript publisher computes real archive hashes/sizes, checks the signer against Desktop pinned public keys, copies immutable archives under `public/updates/releases`, and signs/replaces the catalog. Website deployment has no private signing key and performs no Agent or installed-version decision. See [`website/updates/README.md`](../../../website/updates/README.md) for publication/renewal and `pnpm test:updates` for focused verification. Repository changes require rebuilding/redeploying the existing website; no production deployment or signed installed-package conformance is claimed.

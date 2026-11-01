# Desktop update publication

The website serves `https://www.suncode.ai/updates/v1/stable/{target}/latest.json` through a Node.js App Router handler. This is a public distribution endpoint, separate from Agent APIs. It accepts no installed version, credentials, or device identity. Supported targets are `win-x64`, `win-arm64`, `osx-x64`, `osx-arm64`, `linux-x64`, and `linux-arm64`.

## Current catalogs

`v1/stable/<target>/latest.json` files are complete signed v1 catalogs with revision **1**, Desktop and SDK version **0.1.0**, ABI **27**, and **First release.** notes. They have no special status fields. The route serves them normally with **200**, `Content-Type: application/json`, a SHA-256 ETag over the exact response bytes, and `Cache-Control: public, max-age=300, must-revalidate`. Conditional GET supports ETag lists, weak tags, and `*`, returning **304** without a body. HEAD returns the corresponding headers without a body. Unsupported channels/targets return 400; missing files return 404; damaged files return 503.

At the user's request, archive URLs follow the intended same-site layout, while `sizeBytes` is **1** and `sha256` is **64 zeroes**. No corresponding component archives have been published. The signatures use the pinned release key and are cryptographically valid; the archive metadata is sample data and does not establish installable packages or platform conformance. Desktop checking is enabled against `https://www.suncode.ai`; both apex/www archive origins are allowed.

The client separates version checking from installation eligibility. An unmanaged build whose running Desktop and SDK versions/ABIs match the recommended pair can show Up to date after verifying fresh signed metadata; this creates no receipt or installable plan and does not validate sample archive bytes. Managed installations still check complete receipt identity, so a real receipt with different archive hashes reports an immutable-release conflict. An installation needing an upgrade or SDK repair cannot successfully download/install these nonexistent archives. No client signature, identity, integrity, or compatibility checks are bypassed.

The normal publisher still requires real archives and enforces immutable component identities. When real archives are ready, publish the affected components under new product versions and a larger catalog revision rather than replacing a signed 0.1.0 identity with different bytes. Renew signed expiry before it lapses; HTTP 304 does not extend it.

Catalog files live outside `public/` and `next.config.ts` includes them in route output tracing for serverless/standalone deployment. Run the production server from the website project directory (the existing `pnpm start` convention). No private key is needed in the deployed website or its environment. Rebuild and redeploy the existing website to make repository changes live; this implementation does not perform deployment.

## Publish real component packages

Use Node.js 22.18+ and run commands from `website/`.

1. Build, sign, and platform-test the independent Desktop and native SDK archives following [`Infrastructure/Updates/README.md`](../../apps/desktop-avalonia/Infrastructure/Updates/README.md). Desktop archives exclude the SDK. Confirm actual product versions, C ABI, and exact tested pairs before signing metadata. The publisher computes archive identity; it does not build apps, inspect native exports, sign/notarize application binaries, or create initial-installer receipts.
2. Copy `release.example.json` to a local release descriptor. Replace both `archive` paths and release timestamps/notes with real release facts. Paths resolve relative to the descriptor file. Start at revision 1 for a never-published target. List the recommended pair and any additional exact pairs that were tested. Do not copy example content into a deployed catalog manually.
3. Run the offline publisher with the existing private key stored outside the repository:

   ```sh
   pnpm updates:publish \
     --release /absolute/path/release.json \
     --private-key /outside/repository/ed25519-private.pem \
     --key-id release-20261101-a16ba224
   ```

   The default public trust/origin configuration comes from `apps/desktop-avalonia/Updates/update-distribution.json`. Use `--distribution /absolute/path/update-distribution.json` for another distribution. The publisher uses the distribution's public trust/origin configuration independently of its checking switch. It requires the private key to match an active pinned public key, validates versions/pairs/ABI/timestamps, reads real nonempty ZIP/tar.gz archives, and computes exact byte lengths and SHA-256 hashes. It signs `UTF8("SunCode Update Catalog v1\n") || JCS(signed)` with pure Ed25519 and verifies the result. Private key bytes are never copied into output or printed.
4. Review the generated files:

   ```text
   updates/v1/stable/<target>/latest.json
   public/updates/releases/<target>/desktop/<version>.zip (or .tar.gz)
   public/updates/releases/<target>/sdk/<version>.zip (or .tar.gz)
   ```

   New download URLs use the configured canonical origin, currently `https://www.suncode.ai/updates/releases/...`. Existing signed 0.1.0 catalogs retain their apex download URLs; both domains are allowed during redirects. Archives are copied before the catalog is atomically replaced. Existing archives must match exactly. A published component cannot be downgraded or have its descriptor changed at the same version; a renewed catalog must increase its revision. Publication for one target is locked against concurrent writes. If a local publisher is forcibly interrupted, remove its stale `latest.json.publish.lock`/`.tmp` only after confirming no publisher is still running.
5. Deploy archives before or atomically with the catalog, retain every referenced immutable archive, and verify the actual HTTPS 200/304 behavior after deployment. `public/updates/releases` supports same-site hosting; check your hosting provider's individual file/deployment size limits before shipping large Desktop/CEF archives. Never put private signing material into `public/`, deployment artifacts, or Git.

The default catalog publication time is the command's current UTC time and expiry is seven days later. Before expiry, run the publisher again with a larger revision and unchanged component descriptors, then redeploy. Optional top-level `publishedAt` and `expiresAt` control an explicit publication window; future-dated or expired metadata is rejected. Renewal needs no component version bump or user package re-download.

Desktop `enabled` is true for metadata checks. Actual installation still requires trusted initial installation receipts, platform signing configuration, real published packages, and installed update conformance.

## Verification

```sh
pnpm test:updates
pnpm typecheck
pnpm build
```

Tests use temporary keys and temporary archives, covering canonical JSON/signatures, package hashes/sizes, immutable versions, revision renewal, publication failure, HTTP caching/statuses, and path validation. They do not publish fixtures or establish Developer ID/Authenticode or installed-package conformance. The normative wire contract remains [`contracts/updates`](../../contracts/updates/README.md).

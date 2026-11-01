# Desktop component update protocol

Status: v1 contract with a focused-tested Avalonia C# client, Next.js catalog route, and offline signed publisher. Catalog checking is enabled against the canonical www domain. Real component archives, platform signing configuration, initial-installer integration, and installed OS conformance remain release work. The configured origin and pinned public key do not claim a deployed update release.

[`updates.openapi.yaml`](updates.openapi.yaml) is the normative HTTP/schema description, using [OpenAPI 3.2.1](https://spec.openapis.org/oas/v3.2.1.html), the latest published version checked on 2026-11-01. This document supplies the normative rules that schemas cannot express. Contracts and eventual C# DTOs are hand-written; no code is generated.

## Ownership and distribution

Avalonia C# owns checking, catalog verification, local version comparison, update-plan selection, downloading, and install coordination. These paths must work before loading or opening Agent SDK, including when that library is missing or damaged. The existing Rust version/ABI queries may confirm a loaded runtime's identity but are not prerequisites for checking or repair. No new Rust API, SQLite access by C#, provider request, Remote Server operation, authentication, device identity, or telemetry is introduced.

There are exactly two update components:

| Identifier | Full component package |
| --- | --- |
| `desktop` | Avalonia application, C# binding, managed/runtime dependencies, and update helper; excludes the Rust Agent SDK runtime. |
| `sdk` | Platform-native `sdks/c` library, linked Rust SDK/Agent implementation, and its necessary native dependencies; excludes Desktop. |

The CLI embeds the Rust facade directly and is not updated by this protocol. Full offline installers may bundle both components, but subsequent Desktop update archives must not silently include the SDK. Delta downloads, release-channel switching, hosted agents, and remote installer execution are outside v1.

## Check request and HTTP behavior

```http
GET /updates/v1/stable/win-x64/latest.json HTTP/1.1
Host: www.suncode.ai
Accept: application/json
If-None-Match: "stable-win-x64-42"
```

The distribution uses `https://www.suncode.ai/updates` as its update root: `catalogOrigin` is `https://www.suncode.ai`, and the client appends `/updates/v1/{channel}/{target}/latest.json`. This configuration does not claim that signed catalogs are deployed. The origin is configured by the installed Desktop distribution, never supplied by a project, model, or provider. The URL selects the release channel and **running process/native-library target**, including architecture under emulation. Known targets are `win-x64`, `win-arm64`, `osx-x64`, `osx-arm64`, `linux-x64`, and `linux-arm64`; listing a target does not claim that a build is published. v1 supports `stable` only. No installed version, project path, user identifier, API key, cookie, or SDK handle is sent.

- **200:** A signed catalog, even if both installed versions already match. The client derives update availability locally; there is no server-computed `updateAvailable` field.
- **304:** No body. Use only a previously verified catalog for the exact origin and request path after checking trust, expiry, and publication revision again. Recompute the plan because installed versions may have changed. If no usable cache exists, retry once without `If-None-Match`; failure to obtain a valid 200 remains a failed check. A 304 never extends signed expiry or directly means Up to date.
- **400 / 404:** Invalid request or no published catalog. Neither means Up to date. A static host may return a generic 404 instead of the documented problem body; it is still a failed check.
- **429 / 503:** Respect an optional seconds-form `Retry-After`, bounded to 24 hours; use normal retry scheduling when it is absent or invalid. Do not retry in a tight loop.
- **Other status, malformed/non-JSON body, unsupported schema, signature failure, or transport failure:** Retain the installed versions and last-known display information, mark the check failed, and do not create a new installable plan from untrusted/stale information. Error text is display-only and never release authority.

Client timeout policy distinguishes failure from explicit cancellation. A catalog request has 30 seconds for headers and body; each archive response wait has 30 seconds, each streamed body read has a 30-second no-data deadline, and the whole archive transfer (including redirects) has a 30-minute budget. Receiving data resets only the idle deadline. HTTP 408/504 are timeout failures as well. A timeout never means Up to date or a completed download. Explicit user/lifetime cancellation is handled separately; failures preserve installed identities, remove incomplete transfer/staging files, and expose manual retry. These budgets do not add immediate automatic retries or resumable transfers.

200/304 responses must carry `ETag` and `Cache-Control: public, max-age=300, must-revalidate`, or a shorter freshness lifetime. Static hosting/CDN is sufficient. Cache at most 1 MiB of decompressed catalog JSON, reject duplicate object keys, and require the schema's numeric and collection limits. Manual checking revalidates the catalog even if its HTTP cache is fresh. The client may check at startup and periodically when automatic checks are enabled; checks never imply permission to download or restart.

## Signed catalog and release identity

The JSON envelope contains `signed` and `signatures`. `signed` includes schema version, publication revision, channel, target, publication/expiry times, both component descriptors, and tested `compatiblePairs`. Catalog revision is not a component version. An unchanged component keeps its release descriptor rather than being republished with the other component's version.

Each release exposes a product version, ABI requirement/identity, publication time, full archive URL/format, exact byte count, SHA-256, optional platform prerequisites, and plain-text localized release notes. Stable versions are integer `MAJOR.MINOR.PATCH`, without `v`, leading zeroes, prerelease, or build metadata; each integer is at most 2147483647. Compare numeric triples. UI may add a `v` prefix. A `(component, target, version)` identifies immutable archive bytes and ABI metadata; changing either requires a new product version, never merely a new URL or catalog revision. Release timestamps must not be later than the enclosing catalog's `publishedAt`.

Catalogs always include both recommended descriptors and explicitly certify their pair. `desktop.requiredSdkAbi` must equal `sdk.abi` for that pair. Additional pairs can certify a new component with an older installed counterpart. Pair membership is evidence of publisher-tested C functions, ownership/free rules, callbacks, JSON DTOs, events, and host capabilities; equality of the ABI number alone is not enough. Duplicate pairs, a missing recommended pair, mismatched ABI metadata, illegal versions, or prerequisites on the wrong platform make the catalog invalid.

## Verification and freshness

1. Parse bounded JSON without discarding unknown fields inside `signed`. Authenticate them along with known fields; unknown informational fields may be ignored only after verification where the schema permits extensions. Additive v1 fields must be informational only. New mandatory compatibility, prerequisite, or verification semantics require a new schema/path version; a different `schemaVersion` must fail closed rather than be guessed.
2. Canonicalize the complete `signed` object with [RFC 8785 JCS](https://www.rfc-editor.org/rfc/rfc8785.html). Verify [RFC 8032 Ed25519](https://www.rfc-editor.org/rfc/rfc8032.html) over `UTF8("SunCode Update Catalog v1\n") || UTF8(JCS(signed))`; `\n` is one LF byte. Signatures are canonical unpadded base64url encodings of exactly 64 bytes. Reject invalid/non-canonical encodings. This is pure Ed25519, not Ed25519ph or Ed25519ctx.
3. Require at least one valid signature from a non-revoked public key pinned in the installed Desktop distribution. `keyId` selects an already trusted key; it does not authorize downloading a new one. Multiple signatures allow overlap during key rollover. A new key must first arrive through a trusted Desktop distribution; this catalog has no key-bootstrap endpoint. Never fetch a key from unverified metadata or accept an unknown key as a fallback.
4. Require signed channel and target to equal the request and validate the revision's numeric bounds. Keep the highest authenticated revision and SHA-256 of the canonical `signed` bytes in Desktop-owned update state per origin/channel/target. A lower revision is a rollback attempt. The same revision with different signed bytes is a publication conflict; a renewed signature over identical bytes is allowed. A higher authenticated revision raises the high-water mark even if later freshness or semantic checks make its plan unusable. Persist the mark before those checks and before exposing a plan; failure to persist it fails the check. Do not clear it on failed checks, restart, or component binary recovery. Every signed-content change, including expiry renewal or compatibility withdrawal, requires a larger revision.
5. Reject `publishedAt` more than five minutes in the future. Require `publishedAt < expiresAt <= publishedAt + 7 days` and `now < expiresAt`. Clock errors are surfaced; expiry is never bypassed. HTTPS certificate-chain and hostname verification is mandatory for metadata, archives, and redirects, independent of the provider's insecure-TLS preference. Proxy routing may be reused without weakening these checks.

Well-formed unknown-key or non-verifying extra signatures do not invalidate an envelope if an already trusted signature authenticates the complete same payload. Structurally invalid signatures still fail schema validation. A valid signature never bypasses schema, expiry, revision, compatibility, or platform checks. This contract does not claim protection after local trust state or the Desktop process is compromised.

The OpenAPI examples are independent synthetic publication scenarios, **not a sequential release history**. Their signature strings are shape-only placeholders and must fail real cryptographic verification. They are not production keys or installable manifests.

## Local update-plan selection

Local installation records identify the active component versions, target, archive digest, and Desktop binding/SDK ABI. The running Desktop must confirm its own assembly identity. Records are update metadata, not Agent SQLite rows; a staged archive is not an installed version. Reuse a component or cached download only after validating its identity/integrity. A missing/damaged SDK is repairable without invoking that SDK. A missing/mismatched Desktop identity requires installer repair rather than guessing a version.

Version status and installation eligibility are separate. Without a trusted installation receipt, the Desktop may still report Up to date after authenticating a fresh catalog if its running assembly product version and binding ABI, plus the native library's existing product-version/ABI exports, exactly match the recommended certified pair. These native queries do not open an Agent or SQLite. A missing/unloadable SDK or any version/ABI mismatch cannot produce Up to date. This informational check neither verifies archive identity nor enrolls the installation; it creates no receipt, download plan, or activation authority. Managed installations continue to require their full receipt/file/release identity checks. A differing unmanaged build reports that automatic installation is unavailable until a trusted initial installer provisions its receipt.

For a verified catalog on a managed installation:

1. Consider combinations of the installed Desktop or catalog Desktop and the installed healthy SDK or catalog SDK. A missing/damaged SDK cannot be retained; select a catalog SDK for repair even when its version equals the installation record.
2. A changed combination must occur exactly in `compatiblePairs`, match the target and declared platform prerequisites, and have matching exact Desktop-binding and SDK ABI values. Use local binding/installation metadata for a retained component and the signed descriptor for a replacement. If an applicable prerequisite cannot be determined, refuse that candidate. Numeric dotted OS/glibc comparisons pad missing parts with zero; non-Linux packages must not declare a glibc prerequisite.
3. Refuse automatic component downgrades. A retained healthy component needs no download. A replacement is a strictly newer release, except same-version SDK repair of identical release identity. If the same recorded release version has a different signed archive digest or ABI, fail the check as an immutable-release conflict; do not call it a repair.
4. Among admissible changed combinations, choose the greatest Desktop version, then greatest SDK version using numeric triples. When both published releases are newer, their required certified pair dominates single changes. The selected plan contains only replaced/repaired components, and its byte total is the sum of their `package.sizeBytes` values; cached verified bytes can reduce the remaining network transfer, not the release's declared size.
5. When no changed combination is admissible, show Up to date only if both healthy installed identities match the catalog's recommended pair. Otherwise report no compatible update, unsupported environment, or repair required as appropriate. Never call a missing/unverifiable SDK Up to date.

| Installed Desktop / SDK | Catalog Desktop / SDK | Required certified pair | Plan |
| --- | --- | --- | --- |
| 0.1.0 / 0.1.0 | 0.1.0 / 0.1.1 | 0.1.0 / 0.1.1 | SDK only |
| 0.1.0 / 0.1.0 | 0.1.1 / 0.1.0 | 0.1.1 / 0.1.0 | Desktop only |
| 0.1.0 / 0.1.0 | 0.2.0 / 0.2.0 | Only 0.2.0 / 0.2.0, new ABI | Both |
| 0.1.0 / missing | 0.1.0 / 0.1.1 | 0.1.0 / 0.1.1 | Repair SDK |
| 0.1.0 / 0.1.1 | 0.1.0 / 0.1.1 | 0.1.0 / 0.1.1 | Up to date |

## Download, installation, and recovery boundary

Upload immutable component archives before atomically replacing the catalog. Preserve them for the lifetime of catalogs that refer to them. Renew and re-sign the catalog with a higher revision before expiry even when no binaries change; unchanged component versions do not require new package downloads. Static hosting still needs this publication step. URLs and redirects must remain HTTPS, contain no credentials/fragments, and stay within distribution-configured download origins. Do not derive trust from the URL alone. Verify archive length and SHA-256 before extraction, then apply bounded safe extraction and platform code-signature checks. No catalog field supplies a shell command, install script, destination path, native entry point, or approval bypass. Product notes are plain text with exact-locale, language-only, then `en-US` fallback.

Checking and downloading do not change installed versions or acquire Agent machine-operation authority. The one Desktop coordinator stages the selected plan, rechecks trust/freshness/revision, installed identities, prerequisites and compatibility before application, blocks restart while turns/terminals are active, and requires explicit restart confirmation. If staged metadata has expired, obtain a currently valid catalog and require it still to authorize the exact staged releases/digests and selected pair; otherwise discard the plan and present the new decision. Downloads do not authorize an unverified partial paired installation. Only successful activation updates installed-version records.

Retain the previous usable binary combination, but restore it only when compatible with current persistent data. This protocol grants no database downgrade, backup restoration, automatic replay, or data-loss permission. SQLite migration and recovery remain Rust-owned. The C# client uses immutable version directories, one durable paired installation record, and a pre-database identity barrier; failure before commit keeps the old pointer. Installed platform/signing conformance must still be established before automatic recovery is advertised as release-verified.

## Validation and implementation conformance

Validate the OpenAPI document with the official [3.2 JSON Schema](https://spec.openapis.org/oas/3.2/schema/2026-08-30) and all examples against their referenced schemas. JSON Schema cannot verify signatures, cross-field pairs, numeric version parts, time ordering, or local plan selection. C# tests under `apps/desktop-avalonia/tests/Updates/` exercise these client rules and startup/extraction/state behavior; the client source README records coverage and remaining release checks. The Next.js route and offline publisher under `website/` have focused tests; see `website/updates/README.md` for real-archive publication and renewal. Initial-installer conformance, native signed-package application, and platform recovery must also be verified against this contract. Fixture, headless, and preflight tests alone do not claim production distribution conformance.

# Desktop component updater

The C# client implements [`contracts/updates`](../../../../contracts/updates/README.md). `UpdateRuntime` runs before the Agent SDK and owns one application-wide `UpdateCoordinator`; Settings windows are observers rather than update owners. Closing Settings does not cancel a download. Rust source and Agent SQLite are unchanged.

## Distribution configuration

The checked-in `Updates/update-distribution.json` deliberately disables delivery. It contains the configured `https://suncode.ai` catalog/download origin and the public verification key; it contains no private key or verification bypass. Disabled builds show an unavailable state and make no update requests. Publish a distribution-owned configuration with `-p:SunCodeUpdateDistributionFile=<configuration-file>`; it becomes `update-distribution.json` beside the managed executable and is covered by the Desktop payload/platform signature.

An enabled configuration needs `schemaVersion: 1`, `enabled: true`, an HTTPS `catalogOrigin` containing only an authority, `channel: stable`, `downloadOrigins`, and `publicKeys`. Each public key has a stable `id`, canonical unpadded base64url `publicKey` encoding 32 Ed25519 bytes, and optional `revoked`. Only public verification keys ship in Desktop; private signing material does not. macOS also requires the expected 10-character `macTeamId`; Windows requires the expected signing certificate SHA-256 fingerprint as `windowsPublisherSha256`. Never add an unsigned/testing acceptance mode to production configuration.

The configured update root is `https://suncode.ai/updates`. The client appends `/updates/v1/{channel}/{target}/latest.json` to `catalogOrigin`, for example `https://suncode.ai/updates/v1/stable/osx-arm64/latest.json`. `downloadOrigins` allows archives from `https://suncode.ai`; full immutable package URLs belong in the signed catalog. Both current component product versions are `0.1.0`, read from the Desktop assembly and native SDK/installation identity rather than this distribution configuration.

Updates use the operating system HTTP proxy and normal TLS verification. Agent/provider insecure-TLS preferences never weaken updates; accessing Agent configuration is not required for checking or repair. Currently there is no independent custom update-proxy editor.

## Package layout and initial installation

Archives are complete immutable component payloads, not scripts:

- macOS Desktop: exactly one `SunCode.app/` at the archive root, with its managed executable and distribution configuration under `Contents/MacOS/`. The bundle must already have its release signature/notarization and contain no Rust SDK runtime.
- Windows/Linux Desktop: executable, `SunCode.dll`, runtime/dependency files, and distribution configuration directly at the root. The apphost is `SunCode.exe` or `SunCode`. Preserve Unix executable permissions.
- SDK: `suncode_agent.dll`, `libsuncode_agent.dylib`, or `libsuncode_agent.so` directly at the root, plus necessary native dependencies. It contains no `SunCode.dll`. SDK product/ABI exports must match the signed release metadata.

ZIP and tar.gz are supported. Absolute paths, traversal, case-colliding entries, unsupported entry types, escaping links and link cycles are refused. Relative links must not contain parent traversal. Framework links are resolved and recorded without recursively following them. The client bounds catalogs to 1 MiB, archives to 8 GiB, expanded payloads to 32 GiB, and entries to 100,000. Archives must fit the available filesystem. macOS uses strict `codesign`/Team ID checks and `spctl` for Desktop; Windows uses WinVerifyTrust and the configured publisher certificate. Linux relies on the signed catalog/archive identity.

The trusted **initial installer** must provision `UpdateState.Installation` before this client can manage an existing installation. This initial installer/publisher is outside the client implementation. The receipt identifies target and both components, with version, exact ABI, original signed archive SHA-256, absolute installed directory, and complete installed file manifest. File entries use relative paths and SHA-256; link entries use `link:` plus SHA-256 of the raw relative link target. Receipts and mutable updater state stay outside component archives in Desktop-owned `desktop-updates/state.json`, under the existing application data directory. Do not infer a trusted receipt from arbitrary local files or construct one by editing state manually. A development/legacy build without one keeps bundled native probing but reports that managed update support requires an official installation; it does not silently enroll unverified binaries.

## Activation and recovery

The original application installation remains an immutable launcher. New Desktop and SDK payloads are staged separately and activated from unique directories under `desktop-updates/versions/`. The launcher forwards to the active Desktop before opening instance IPC or Agent state. The C# SDK resolves the active native runtime by absolute path once; no in-process library swap is attempted.

On confirmed restart, the coordinator verifies the plan again, re-extracts signed archive bytes, checks platform identities, records a pending journal, and starts its own executable in bounded helper mode. It checks application-wide turns, terminal processes, loading/submission activity and queued input before preparation and immediately before shutdown. Uncertain Agent activity blocks restart. The helper waits for the exact old process instance, starts the candidate with a preflight identifier, and waits for actual assembly/native ABI/product-version checks. Those checks do not open the Agent database.

Only then does the helper durably replace the single paired installation record and release the startup barrier. The state write uses file flush plus directory synchronization on Unix or write-through replacement on Windows. Before that commit, failed or interrupted activation leaves the previous pointer untouched. After commit, startup never automatically returns to an older SDK that may not support data already migrated by the new one. Native repair and metadata high-water marks remain independent of Agent availability. Previous binaries are retained; cleanup preserves active and previous combinations plus two recent unused version directories. Cleanup runs only in the primary desktop process and never prunes an active activation.

## Verification boundaries

`tests/Updates/` covers signed JCS/Ed25519 metadata, monotonic revisions and conflicts, caching/expiry, compatible plans, repair, bounded transport, download integrity/cancellation, safe extraction, activation records/barriers, real host preflight with valid/invalid native libraries, view-model state, localization, and constrained layouts. Unit tests use ephemeral in-memory signing keys and fake HTTP/platform adapters; real-host preflight checks use the built native library in temporary directories.

The distribution origin and public verification key are configured; delivery remains disabled pending signed catalog publication and platform signing configuration. Website catalog routing and the offline publisher are implemented under `website/updates/`; see its README for publication/renewal. Initial-installer integration, real Developer ID/Authenticode releases, signed installed-package restart, and Windows/Linux runtime conformance remain release verification. Passing local fixture/headless tests does not claim those delivery paths are deployed or OS-verified.

# Desktop Settings Review Surface

Settings is organized into:

- `components/` for small settings controls and row primitives.
- `sections/` for top-level settings panels and their list/state orchestration, including the Remote Server connection and mobile pairing flow.
- `dialogs/` for editor windows and consequential settings flows.
- `data/` for provider, guide, MCP, and language-server catalog fixtures.
- `styles/` for settings-owned visual rules.

`index.jsx` is the stable route entrypoint. `SettingsPage.jsx` owns only page state, navigation, and section selection; section implementations, dialogs, and fixtures live under their focused directories. Keep route imports pointed at `index.jsx` so internal file moves do not spread through the application shell.

Language servers use a SunCode-owned catalog rather than a free-form server editor. The overview is scoped to the current project, and each supported server opens an in-panel configuration page tailored to that server. The fixtures cover the implemented jdtls, vtsls, basedpyright, rust-analyzer, and Roslyn Language Server surfaces. jdtls specifies Java runtime and project-unique data directories; vtsls specifies Node.js validation, project matching, TypeScript SDK policy, and tsserver memory; basedpyright specifies a target Python interpreter, diagnostic scope, type-checking mode, and project configuration discovery; rust-analyzer specifies toolchain detection, Cargo analysis, diagnostics, and compilation target policy; Roslyn specifies .NET detection, C# project loading, solution selection, restore, source generators, and diagnostic scope. All five cover project enablement, environment, timeouts, runtime status, dirty actions, and retry behavior.

The Remote Server specimen covers disconnected input, pairing-code validation, connected status, mobile QR presentation, the pairing link with a copy affordance below the QR, QR renewal, and disconnect. Its pairing QR is a visual placeholder and does not encode a live credential.

## Updates — client implemented; React review simulated

Open `#/projects/desktop/settings/updates` for the update specification. Updates follows Logging in the Settings navigation. `sections/UpdatesPanel.jsx` owns the in-memory interaction, `data/updates.js` owns synthetic releases and 21 scenarios, `components/UpdateReviewControls.jsx` owns the external review toolbar, and `styles/updates.css` owns the scoped styles. The Avalonia C# client uses the shared native settings expander and global SCToggle mapping (`sc-toggle-switch settings-switch`), with its 34 × 20-DIP track, 14-DIP knob, shared theme/state resources, and no local brush overrides or redundant On/Off label.

The two version rows keep Desktop and Agent SDK distinct. A single compatible plan covers either component or a required pair, with its total download size and expandable release notes. Automatic checks default on; download and confirmed restart are explicit. Downloaded targets become installed versions only after successful activation; React simulates that transition after restart. The shared confirmation initially focuses Later, and the active-work fixture disables restart while a turn and terminal are running. Failed targets retain installed version identity and show Update not installed; verification failure never offers restart. The restored-version fixture applies only after safe, supported recovery and does not specify database downgrade or data restoration. SDK repair remains accessible without a running SDK.

Check timeout, download timeout, and stalled download reuse the failed-state layout with distinct localized headings. The stalled hint explains the 30-second no-data limit. Check timeout has no installation plan and offers Try again; download timeout/stall retain the compatible plan for Download again. Installed identities stay unchanged and incomplete files/staging are removed. The client bounds catalog requests and archive response waits to 30 seconds, streamed no-data waits to 30 seconds excluding disk writes, and total archive transfers to 30 minutes including redirects/local writes. HTTP 408/504 are timeouts; explicit cancellation remains separate. Recovery is manual, without new automatic download/retry/resume behavior; four-hour checks and the single unusable-304 fallback remain unchanged.

The toolbar selects macOS, Windows, or Linux, default 900 × 672 or minimum 720 × 552 window size, and static review scenarios, including unconfigured delivery and unmanaged installation. Without a trusted installation receipt, the client may show the existing Current / Up to date state when actual running Desktop/SDK versions and their exact ABI identities match the recommended certified pair after catalog authentication, freshness, revision, and schema checks. This is an informational version comparison: it creates no receipt, plan, staged update, download or activation authority, and does not certify archive integrity. Managed receipt/hash/file-health validation is unchanged.

Differing versions, a missing/unloadable SDK, or an ABI mismatch retain the automatic-installation availability warning while SunCode remains usable, with no installation plan. The React unmanaged fixture represents differing installed/recommended versions; repeated checks return to that warning. The Current fixture represents matching versions. Receipt absence is distinct from genuine damaged-installation or invalid-state repair errors; no receipt is invented.

Explicit actions animate checks, downloads, verification, and restart in memory. The React panel stays mounted across Settings navigation. Both navigation and content scroll inside the native frame; below a 620px catalog viewport the bounded navigation stacks above the panel. All versions, sizes, and notes are examples; this React review invokes no update service, filesystem installation, or system restart.

The C# client owns an application-wide coordinator, so closing Settings preserves a download. Its checked-in distribution configuration enables catalog checks against the canonical www origin with pinned public trust. Current signed 0.1.0 metadata uses sample package values; real archives, trusted initial-installer receipt integration, platform signing configuration, and signed installed-package restart/platform release conformance remain release work. See root `DESIGN.md` for the interaction contract and `apps/desktop-avalonia/Infrastructure/Updates/README.md` for client delivery boundaries.

## Provider credentials

Open `#/projects/desktop/settings/provider-credentials` for system-store-only API key settings. The review toolbar selects macOS, Windows, or Linux wording and normal, locked, denied, unavailable, missing-entry, saving, save-failed, or remove-failed states. Retry returns the fixture to Normal.

DeepSeek and OpenAI demonstrate saved system-store keys. Zhipu demonstrates an empty provider. Save, Cancel, replacement, and confirmed removal follow the production Rust SDK and Avalonia interaction. The form shows the platform store without a storage selector. Only masks leave `ProviderCredential.jsx`, and fixture changes survive in-page provider navigation. Secret values remain unpersisted component state. `styles/provider-credentials.css` owns the styles.

The new system has no plaintext-key migration flow. Linux requires a running, unlocked Secret Service in the user session. Real OS authorization and installed-platform behavior require release verification.

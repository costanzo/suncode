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

Open `#/projects/desktop/settings/updates` for the update specification. Updates follows Logging in the Settings navigation. `sections/UpdatesPanel.jsx` owns the in-memory interaction, `data/updates.js` owns synthetic releases and 17 scenarios, `components/UpdateReviewControls.jsx` owns the external review toolbar, and `styles/updates.css` owns the scoped styles. The Avalonia C# client implements this surface using the shared native settings expander and toggle without a redundant On/Off label.

The two version rows keep Desktop and Agent SDK distinct. A single compatible plan covers either component or a required pair, with its total download size and expandable release notes. Automatic checks default on; download and confirmed restart are explicit. Downloaded targets become installed versions only after successful activation; React simulates that transition after restart. The shared confirmation initially focuses Later, and the active-work fixture disables restart while a turn and terminal are running. Failed targets retain installed version identity and show Update not installed; verification failure never offers restart. The restored-version fixture applies only after safe, supported recovery and does not specify database downgrade or data restoration. SDK repair remains accessible without a running SDK.

The toolbar selects macOS, Windows, or Linux, default 900 × 672 or minimum 720 × 552 window size, and static review scenarios, including unconfigured delivery. Explicit actions animate checks, downloads, verification, and restart in memory. The React panel stays mounted across Settings navigation. Both navigation and content scroll inside the native frame; below a 620px catalog viewport the bounded navigation stacks above the panel. All versions, sizes, and notes are examples; this React review invokes no update service, filesystem installation, or system restart.

The C# client owns an application-wide coordinator, so closing Settings preserves a download. Its checked-in distribution configuration disables delivery and makes no update requests. Production origins/verification keys, publisher and trusted initial-installer receipt integration, and signed installed-package restart/platform release conformance remain release work. See root `DESIGN.md` for the interaction contract and `apps/desktop-avalonia/Infrastructure/Updates/README.md` for client delivery boundaries.

## Provider credentials — review only

Open `#/projects/desktop/settings/provider-credentials` for the interactive API key storage proposal. The toolbar outside the Settings specimen selects macOS, Windows, or Linux wording and normal, locked, denied, unavailable, missing-entry, saving, save-failed, or remove-failed states. Retry returns the fixture to Normal; it performs no OS request.

DeepSeek demonstrates a saved system-keychain key, OpenAI a saved plaintext key, and Zhipu an empty provider that defaults to system-keychain storage. All providers support Save, Cancel, replacement, explicit plaintext confirmation, and confirmed removal. Successful fixture changes survive in-page provider navigation; only masks and mode leave `ProviderCredential.jsx`. No secret is persisted. `styles/provider-credentials.css` owns this feature's styles.

Approval is pending. This page establishes a UI proposal, not an implemented Rust credential contract. No migration flow is planned for the new system.

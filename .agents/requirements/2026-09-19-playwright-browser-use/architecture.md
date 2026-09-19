# Architecture

## Current state

The embedded Rust agent owns tool orchestration, policy, approvals, persistence, MCP, LSP, SDK contracts, and machine operations. The provider-neutral message shape can carry user image inputs, but tool results are currently persisted and reconstructed as text JSON. Avalonia Settings has first-level Network, MCP, Language servers, Logging, Agents, and Model providers destinations. Production currently prohibits Node.js and has no browser process, browser profile, browser tool catalog, or packaged browser runtime.

## Proposed design

Add `suncode-browser` as a Rust protocol and lifecycle package plus a bundled first-party Node.js worker using Playwright. Core owns a project-scoped `BrowserManager`; it validates the installed runtime, lazily starts one worker per project, tracks control ownership and pages, merges browser definitions into primary-session provider requests, and sends every operation through policy and `session_tool_use`.

The Node worker owns only Playwright objects and browser protocol adaptation. It cannot call providers, open SQLite, select policy, create approvals, mutate project files, or load third-party extensions. Worker source is ordinary JavaScript; Node.js is a narrowly accepted production dependency for this fixed browser driver, not a return of the obsolete TypeScript agent path.

## Boundaries and dependencies

```text
Avalonia Browser use Settings
  -> C# SDK -> C ABI -> Rust SDK facade
    -> configuration.browser_use_enabled
    -> core BrowserManager
      -> suncode-browser framed stdio client
        -> bundled Node.js worker
          -> pinned Playwright
            -> bundled Chromium + project profile

provider request
  -> built-in definitions + browser definitions when enabled/healthy
  -> validation -> browser preflight -> policy/approval -> revalidation
  -> BrowserManager operation -> bounded result/artifact -> model context
```

- `suncode-browser` depends only on common contracts, Tokio, Serde/JSON, URL handling, and process primitives.
- Core owns project/session identity, configuration, policy, approvals, generation retirement, tool results, and managed artifacts.
- `suncode-tool` owns static browser model schemas and pure argument validation but no browser state.
- The SDK exposes runtime-management methods; it does not expose model browser calls to Avalonia.
- Avalonia owns presentation and explicit user-control commands only.

## Data and control flow

### Runtime validation

Rust resolves the target-specific packaged runtime relative to the application/native library layout, reads `runtime-manifest.json`, validates critical paths, starts the worker in probe mode, and compares the handshake with the manifest. A successful result is cached for the installed application version; `Verify runtime` requests a complete recheck.

### Project activation

Project open does not start a browser. The first authorized browser tool creates the profile/temp/staging directories, launches a worker with a filtered environment, opens a persistent Chromium context, and installs a runtime generation. One serialized queue prevents interleaved page mutations. Session-owned page handles are opaque and never expose OS process IDs or filesystem paths to the model.

### Observation and action

The worker creates an accessibility-oriented snapshot, assigns references bound to `(page_id, revision)`, and returns bounded text. An action first resolves its target and returns a preflight descriptor. Core evaluates browser-specific policy, persists/suspends approval when required, then asks the worker to execute only if page and revision still match. Navigation, popup, dialog, download, console, and crash events update runtime state or become bounded results.

### Control handoff

The browser normally remains a headed Chromium window outside the foreground. Show/take-control changes the runtime lease to `user`, restores the window where supported, and prevents model actions. Return-control changes the lease to `agent`, clears all snapshot references, and requires a new observation before an action.

### Screenshots and downloads

The worker returns screenshot bytes or writes only to a Rust-provided staging directory. Rust validates size and canonical location, promotes accepted output into managed artifact storage, and records artifact metadata in the tool result. Downloads remain deferred until the authority and artifact contract is complete; the packaged worker contains no generic filesystem path parameter.

## Security and failure handling

- Worker launch clears the environment and supplies only process, locale, display/session, temporary, proxy, and dedicated runtime directory values. Provider keys and dynamic-loader injection variables are excluded.
- The browser is not an OS sandbox. It runs with the user's authority, and the UI states this honestly.
- Browser URLs reject credentials, unsupported schemes, browser-internal pages, and unsafe redirect transitions without a new approval.
- Browser-origin grants are session-scoped and browser-specific. High-consequence actions ignore them and require action-time approval.
- Page text, accessibility names, console messages, downloads, and screenshots are untrusted tool content.
- stdout framing has explicit maximum message sizes. stderr and console data are bounded and redacted.
- Timeouts and cancellation retire the in-flight request. A wedged worker is terminated as a process tree and may restart once with the same profile.
- Browser runtime failure never prevents project/session access, provider use, or ordinary tools.
- HTTPS verification remains browser-native and cannot be disabled by the agent.

## Compatibility and migration

The global boolean `browser_use_enabled` is added to the existing `configuration` table, seeded `false`; no new configuration table is needed. Runtime identity and project state are derived and memory-only. Browser-specific session origin grants require a dedicated table only when session-wide origin approval is implemented; the initial implementation may conservatively use allow-once approvals and add the grant table in a later coherent change.

The C ABI advances for additive Browser Use management methods. Existing sessions, tool rows, messages, projects, and profiles need no conversion. The obsolete production TypeScript path remains removed.

## Risks and rollback

The main risks are package size, platform signing, Linux shared-library coverage, browser prompt injection, fragile UI references, profile corruption, and cross-process cancellation. Exact manifests, offline smoke tests, semantic locators, revision checks, narrow worker commands, runtime generations, and fail-closed policy reduce those risks.

Rollback disables `browser_use_enabled`, removes browser definitions, stops project workers, and leaves profiles inert for later cleanup. Removing bundled binaries in a future release does not require deleting profiles or changing session history.

## Open questions

- Freeze exact runtime versions and the supported Linux distribution matrix after the packaging spike.
- Decide whether screenshot image parts are delivered in tool-role content per provider or reconstructed as an adjacent transient browser-observation message after focused provider compatibility tests.

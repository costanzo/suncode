# Requirement

## Background

SunCode can fetch static web content but cannot operate a real browser. Developers need the agent to exercise local web applications, inspect dynamic pages, reproduce frontend failures, and verify changes through the same audited tool workflow used for project operations. The capability must be first-party and must not use MCP.

The approved direction bundles a fixed Node.js runtime, a fixed Playwright package, and the exact Chromium build required by that Playwright version. Rust remains the authoritative agent and owns process lifecycle, policy, approval, audit, persistence, recovery, and the SDK boundary. The Node worker is a narrow browser driver, not another agent implementation or extension host.

## Goals

- Add first-party Playwright Browser Use to the embedded Rust agent without MCP.
- Ship hermetic browser runtimes for macOS arm64, Windows x64, and Linux x64.
- Pin Node.js, Playwright, and Chromium versions and verify their runtime identity.
- Keep one persistent, project-isolated Chromium profile outside the opened project.
- Run the browser in the background by default and support an explicit user-control handoff.
- Expose bounded semantic browser tools through normal policy, approval, cancellation, and `session_tool_use` audit.
- Add a first-level Browser use Settings destination showing enablement, health, component versions, paths, project profile information, and runtime controls.
- Prevent browser activity from being described as covered by filesystem undo.

## Non-goals

- MCP-based browser tools.
- A second model loop or provider client in Node.js.
- Arbitrary JavaScript, Node.js, Playwright-script, CSS-selector, or XPath execution by the model.
- Reusing or importing the user's ordinary Chrome profile.
- Runtime npm installation or browser download.
- Firefox, WebKit, branded Chrome, Edge, or browser extensions in the first delivery.
- File upload, CAPTCHA handling, password changes, financial transactions, or unattended non-interactive browser operation in the first delivery.
- Claiming OS sandbox isolation around the Node worker or Chromium.

## Requirements

### Distribution and integrity

1. Release artifacts bundle Node.js, the production Playwright package, the regular Playwright Chromium build, Playwright's pinned FFmpeg helper, the worker, licenses, and a target-specific runtime manifest.
2. The initial targets are `darwin-arm64`, `win32-x64`, and `linux-x64`.
3. Exact versions and target-specific SHA-256 values are committed. Semver ranges and `latest` are prohibited.
4. Playwright and Chromium upgrade as one atomic compatibility unit. SunCode never downloads or updates them at runtime.
5. The worker handshake reports protocol, Node.js, Playwright, Chromium version/revision, and target. Rust fails closed on mismatch.
6. Linux support names an explicit Playwright-supported Ubuntu/Debian x64 baseline and reports missing shared dependencies without making the rest of SunCode unavailable.

### Runtime ownership

1. Rust starts one lazy project-scoped worker on first browser use and owns its process tree, timeouts, cancellation, restart budget, and shutdown.
2. Each project has one persistent profile under the agent data directory. Profiles never live in the project and never reuse an external browser profile.
3. Sessions in one project may own separate pages but share the project profile and serialized browser-operation queue.
4. The worker receives a filtered environment and dedicated profile, temporary, and download-staging directories. It receives no provider credentials.
5. stdout is reserved for a framed protocol. stderr is bounded, redacted, and diagnostic-only.
6. Browser and worker crashes become recoverable typed tool failures. Repeated crashes move the project runtime to `failed` until retry.

### Browser behavior

1. Browser Use is globally disabled by default and advertised to the model only when enabled and the bundled runtime is healthy.
2. Enabling does not eagerly launch Node.js or Chromium. Disabling retires tools immediately, cancels active calls, and closes browser processes without deleting profiles.
3. Chromium uses a persistent headed context. Background mode minimizes or removes it from the foreground where the platform permits.
4. `Show browser and take control` transfers an exclusive control lease to the user. Agent browser calls cannot execute until the user returns control.
5. Returning control invalidates prior element references and requires a fresh observation.
6. Linux Wayland limitations are reported honestly when programmatic foreground activation is unavailable.

### Model tools

1. The first delivery exposes narrow built-in tools for opening/navigating, observing, clicking, filling, selecting/checking, pressing keys, scrolling, waiting, screenshots, console messages, tab inspection, and closing.
2. Observations use a bounded accessibility/DOM snapshot with page identity, URL, title, revision, and short-lived element references.
3. Actions use an element reference or a structured role/label/test-id/text locator. Raw selectors are rejected.
4. One call performs one user-visible action; batches that could hide approval boundaries are not accepted.
5. `http` and `https` are supported. `file`, browser-internal, extension, data-navigation, and executable schemes are rejected.
6. Screenshots are bounded managed artifacts. Vision-capable provider delivery is capability-gated and text observation remains the universal fallback.
7. The first delivery exposes browser tools only to primary sessions. Built-in child agents do not receive them.

### Authority and safety

1. Browser enablement grants capability availability, not site or action authority.
2. Page content is untrusted data and cannot grant permission, change policy, or authorize transmission.
3. New origins require an explicit browser-origin approval. Low-risk session grants are browser-specific and never turn on general Full Control.
4. State-changing actions use a preflight descriptor, policy evaluation, approval where required, revision revalidation, and only then execution.
5. Sensitive transmission, account/permission changes, destructive actions, representational communication, and financial actions require action-time confirmation and are not bypassed by Full Control.
6. Non-interactive Browser Use fails closed in the first delivery.
7. Browser changes to remote systems and browser profiles are never included in filesystem undo.
8. HTTPS interstitials are not bypassed. Browser traffic does not inherit the global insecure certificate-verification toggle.

### Settings experience

1. Settings adds a first-level `Browser use` destination after Network.
2. The page uses the existing Settings row/section language and shows no dashboard-style metric cards.
3. It shows the global enable switch, installation health, project runtime state, target, Node.js version/path, Playwright version, Chromium version/revision/path, worker protocol, and integrity state.
4. Runtime paths are read-only, monospace, elided, keyboard reachable, and copyable. External runtime paths cannot be configured.
5. With a current project, the page shows profile scope/path/size, active pages, visibility capability, last safe error, and applicable show, return-control, restart, stop, retry, and clear-data actions.
6. Without a current project, the page explains that project runtime and profile details require a project window.
7. Clearing browser data uses the shared destructive confirmation window and can proceed only after the project runtime is stopped.
8. The design-system specimen and Avalonia implementation cover disabled, ready, starting, background, user-controlled, failed, mismatch, missing dependency, no-project, and destructive-confirmation states in light and dark themes.

## Edge cases

- A settings toggle changes while a browser call or approval is pending.
- A user closes Chromium during user-control handoff.
- A stale snapshot reference resolves after navigation or DOM replacement.
- A popup or redirect changes origin after approval.
- The packaged worker starts but reports the wrong Node.js, Playwright, Chromium, target, or protocol version.
- A project profile is locked by an orphaned Chromium process.
- A user opens several project windows and only some have used Browser Use.
- Chromium is visible but the operating system refuses foreground activation.
- A screenshot exceeds pixel or encoded-size limits.
- Browser console output contains secrets or excessive content.

## Acceptance criteria

- All three release targets pass an offline packaged-runtime smoke test with no network installation.
- Runtime identity shown in Settings matches the launched Node.js, Playwright, and Chromium processes.
- Project profiles are isolated and login state survives a clean application restart.
- Tool definitions disappear on disable and no retired invocation executes.
- Browser actions are audited through `session_tool_use`, cancellable, and recover from stale references without terminating the whole session.
- User takeover pauses agent browser activity and return-control forces a fresh observation.
- External page text cannot bypass approval or cause silent sensitive transmission.
- Runtime corruption or missing Linux dependencies disable only Browser Use.
- Design-system and Avalonia focused tests cover navigation, state copy, keyboard behavior, overflow, themes, and destructive confirmation.

## Open questions

- The exact Node.js LTS patch, Playwright version, Chromium revision, and Linux distribution baseline are selected and frozen during the packaging spike, then recorded in `browser-runtime/runtime-lock.json` before implementation is declared complete.

# Shell settings and PowerShell setup

**Status:** Implemented with focused verification; Windows installation and ConPTY conformance require a Windows host.

Settings → Terminal configures Agent shell and desktop Terminal shell independently. Desktop owns selection and checking. Rust persists `agent_shell` and `terminal_shell`, including host-resolved launch facts, as global configuration objects. System defaults use Windows PowerShell on Windows, POSIX sh for Agent execution on macOS/Linux, and the user's login shell for interactive desktop terminals. No bundled PowerShell runtime or Node-based shell packaging remains.

Desktop detects custom absolute paths with bounded executable probes. Invalid drafts cannot replace saved settings; explicit configured failures never silently fall back to another interpreter. Shell families are PowerShell on Windows, and sh, dash, bash, zsh, or PowerShell on Unix. Agent startup uses non-interactive arguments without user profiles, filtered process environments, UTF-8 PowerShell output, existing policy, and existing process-tree cancellation. Terminal startup retains interactive/login behavior and remains Avalonia/RoyalTerminal-owned.

Each new Agent turn captures the persisted host-supplied resolved path, shell family, version, and launch facts in its durable continuation. Its system context uses that captured version, including after approval/recovery. Shell changes affect subsequent turns only. Older continuations with no captured shell refuse bash execution rather than adopting a newly configured interpreter. Terminal reads its current preference when a tab starts or restarts; running tabs retain their process.

Windows Settings recommends PowerShell 7 and can install the official pinned 7.6.0 x64/ARM64 ZIP into the user's local application runtime directory. Desktop owns architecture selection, expected SHA-256 checks, bounded extraction, staging, version probing, replacement, progress, cancellation, and cleanup. It obtains the stored TLS/proxy transport configuration from the trusted-host SDK accessor; the agent performs no setup work. Managed selection validates the runtime manifest and version. One Desktop setup service and a retained managed SDK reference are shared across project windows; closing one window does not cancel another window’s installation. A filesystem install lock serializes desktop instances, and abandoned owned staging directories are cleaned on the next installation. Optional switching commits both settings atomically after successful installation. Failure/cancellation preserves previous settings. The last Desktop window cancels and joins setup before disposing the service’s retained SDK reference. System PATH and Windows Terminal profiles are not modified.

## Verification

- Rust tests cover persistence without executable probing, structural validation, atomic independent/both preference updates, admission without shell discovery, and approval continuation behavior after settings change.
- Avalonia tests cover executable detection and timeout/output bounds, detection-before-save, stale detection results, dirty tracking, ZIP path/symlink/duplicate rejection, digest checks, cancellation, shared multi-window lifecycle, proxy routing, and localization consistency. Native headless rendering was compared with the design-system specimen in both themes at default and minimum dimensions.
- Real Windows PowerShell 5.1/7 output, official ZIP installation/cancellation/repair, Windows x64/ARM64 compatibility, and ConPTY startup remain release conformance checks; macOS verification cannot establish them.

Contracts and behavior: `contracts/agent-sdk/README.md`, `DESIGN.md`, and `.agents/specs/shell-settings.md`.

## Existing broader verification failures

Strict whole-boundary Clippy is blocked by pre-existing document-crate lints and missing safety documentation on older C ABI functions. Shell-focused SDK tests and no-dependency lint of the affected agent packages pass.


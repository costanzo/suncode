# Test Plan

## Design-system stage

- Build the Vite review browser and run git diff --check.
- Verify route reachability from Settings navigation and catalog sidebar.
- Exercise independent custom paths, empty/checking/available/invalid/timeout states, disabled Save, save persistence, and return to system default.
- Exercise install stages, cancellation, retry after failure, architecture-specific installed path, install-only versus switch-both choices, and installed-runtime reuse.
- Review light/dark themes, long paths, keyboard focus, native minimum width, and constrained viewport overflow.
- Confirm no downloads/process launches or production configuration changes.

## Production stage — pending

Validate real official asset hashes, safe ZIP extraction, failed/cancelled cleanup, offline/proxy conditions, Windows architecture support, PowerShell 5.1/7 encoding/exit/cancellation, turn/approval/recovery shell identity, and terminal-tab lifecycle.

## Commands and results

- `npm run build` in `design-system/`: passed. Vite retains the existing bundle-size advisory.
- `git diff --check`: passed.
- Browser review: Terminal route accessible from the catalog and Settings navigation; independent save survives panel navigation; invalid/empty drafts disable Save; custom executable selection rechecks and permits Save.
- Installation review: cancellation retains settings; install-only retains both defaults; completion with both targets updates both selectors; failure retry succeeds; ARM64 path uses the architecture-specific directory.
- Missing, timeout, unsupported shell, checksum failure, download failure, and directory-permission fixtures show actionable feedback.
- macOS/Linux show the appropriate illustrative default paths and omit the Windows installer.
- Both themes and exact 720 × 552 specimen reviewed; no panel horizontal overflow, content scrolls to installation controls. Keyboard focus has a visible token-based ring. Browser console has no errors/warnings in the final review.

## Residual risks

Browser fixtures do not prove executable availability, download integrity, native PTY behavior, or Windows compatibility.

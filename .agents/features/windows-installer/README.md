# Windows offline installer

Status: Real x64 package build and isolated Inno lifecycle tests pass; ARM64, clean-machine/visual conformance, platform signing and managed-update enrollment remain unverified.

`scripts/package-windows.ps1` publishes the existing Avalonia project with .NET and Rust, validates native architecture, adds app-local Visual C++ redistributable DLLs, and invokes Inno Setup for one offline installer plus a SHA-256 sidecar. The version follows the Desktop project. Native cross-compilation is refused; CEF and the separate CLI are excluded.

Inno owns current-user installation, shortcuts, activation protocol and Windows uninstall. Same-version reinstall repairs missing files. Downgrades (including when the executable is missing) and architecture replacement are refused. Uninstall removes installer-owned files/registrations and preserves unrelated files and the application data directory. It does not access Agent SQLite or force-close running Agent/Terminal work.

The baseline is unmanaged: no trusted component receipt is inferred from publish files. The existing component updater retains its authority and state. Signed initial enrollment remains pending; see `desktop-component-updates`. The definition supports Inno 6.3+; focused build/lifecycle verification uses 7.1.1. `scripts/test-windows-installer.ps1` exercises a unique test identity/protocol/shortcuts under ignored artifacts and uninstalls it in `finally`.

Build instructions, dependency handling and release gaps are in `apps/desktop-avalonia/packaging/windows/README.md`. The standard native wizard boundary is specified in `DESIGN.md` and `design-system/src/platforms/desktop/README.md`.

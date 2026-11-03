# Windows offline installer

Inno Setup turns the self-contained publish directory into one offline `SunCode-<version>-<target>-Setup.exe`. Inno owns installation and uninstall. The installed app keeps separate .NET and Rust binaries; single-file publishing and trimming remain disabled.

## Build

Run on Windows with .NET SDK 10, a matching MSVC Rust host toolchain, Visual C++ build tools including the redistributable CRT, and Inno Setup. Inno Setup 7 is recommended; the definition requires at least 6.3. Validation uses 7.1.1. Build tools are not application runtime dependencies.

From the repository root:

```powershell
./scripts/package-windows.ps1
# Or specify a compiler outside PATH and standard installation directories:
./scripts/package-windows.ps1 -InnoCompilerPath 'C:/build-tools/Inno Setup/ISCC.exe'
```

The command reads the Desktop version from the project, checks prerequisites, and publishes Release with a RID and the .NET runtime. The existing managed SDK target builds/copies Rust; the existing Release publish target clears its publish directory. It validates native PE architecture, copies the matching app-local Visual C++ CRT, compiles Inno, and writes a SHA-256 sidecar. Symbols and Linux desktop integration files are excluded. A checksum detects accidental corruption; it does not authenticate the publisher.

The Rust DLL imports `vcruntime140.dll`, which the .NET self-contained runtime does not supply. The CRT comes from Visual Studio's matching `Microsoft.VC*.CRT` redistributable directory, never System32. Supply `-VcRuntimeDirectory <directory>` when automatic discovery is unavailable.

For Desktop `0.1.0`, the ignored outputs are:

```text
artifacts/windows/0.1.0/win-x64/publish/
artifacts/windows/0.1.0/win-x64/installer/SunCode-0.1.0-win-x64-Setup.exe
artifacts/windows/0.1.0/win-x64/installer/SunCode-0.1.0-win-x64-Setup.exe.sha256
```

Distribute the installer and optional checksum. Build/restore may access NuGet, Cargo and pinned font sources. Installation requires no network download of .NET or Visual C++ prerequisites. Normal provider/update traffic remains application behavior.

`-RuntimeIdentifier win-arm64` requires the ARM64 MSVC Rust host and ARM64 CRT; the managed SDK currently builds the Rust host target, so cross-compiling .NET alone is refused. ARM64 packaging/installation remains unverified. x64 packages allow x64-compatible Windows, including supported emulation. The declared OS floor is Windows 10 build 17763, matching the Desktop API target; clean-machine OS conformance remains release work.

CEF is explicitly excluded, matching the Desktop default. Browser Use and the built-in browser remain unavailable. A CEF-enabled installer needs separate native packaging/dependency verification. The native CLI retains its separate release flow and is not included.

## Installation and uninstall

- Install for the current user without administrator elevation, under `%LOCALAPPDATA%/Programs/SunCode` by default. Directory selection remains available.
- Keep the stable Inno AppId across versions/architectures. Windows Installed apps uses Inno's uninstaller. Create a Start menu shortcut and an optional desktop shortcut, initially unchecked.
- Register the quoted current-user `suncode://` activation command. Uninstall removes the protocol, installer identity, owned shortcuts and files. Notification display/click conformance remains a separate release check.
- Use Inno's standard Restart Manager handling for files in use, without forced process termination or automatic application restart. Close SunCode normally after active Agent/Terminal work finishes. The installer does not query SQLite or resolve approvals.
- Offer launch after interactive installation, initially unchecked. Silent installation never launches SunCode.
- Reject installer downgrades using both uninstall version metadata and the installed executable version; a missing executable does not bypass the metadata check. Architecture replacement requires uninstall first.
- Same-version reinstallation repairs missing package files. There is no recursive wildcard uninstall deletion. Unrelated files in the installation directory survive.
- Preserve sessions, credentials, settings, logs, downloaded shells and updater state under `~/.suncode` or `SUNCODE_DATA_DIRECTORY`. Uninstall does not erase projects, modify PATH or remove system shells.

Unattended setup/uninstall uses Inno's normal `/VERYSILENT /SUPPRESSMSGBOXES /NORESTART` switches. `/DIR="<directory>"` selects the install directory and `/LOG="<file>"` writes a setup log. Treat nonzero exit codes as failure/refusal. Windows may still show security prompts for unsigned builds.

## Component updates and release limits

Inno owns the baseline installation and uninstall. The C# coordinator retains independent Desktop/SDK update authority under [`contracts/updates`](../../../../contracts/updates/README.md). A trusted initial receipt requires the exact signed component archives, manifests and certified pair; do not fabricate one from loose publish files.

This installer currently provisions no receipt. The baseline retains unmanaged informational version checking; it does not enable managed update installation or SDK repair through the updater. Existing managed pointers/state are preserved. Updating the baseline does not replace an already activated managed component pair. Initial-installer enrollment remains pending with the signed release pipeline.

`-DistributionConfiguration <file>` uses the existing Desktop distribution override. It generates no signatures or receipts. Public release still needs Authenticode integration for the executable/native SDK, installer and Inno-generated uninstaller, plus the matching updater publisher fingerprint. This entry point builds unsigned packages and adds no verification bypass. The CLI GitHub release workflow does not publish Desktop installers.

## Verification

After a successful x64 package build, use Inno 7 for the lifecycle test:

```powershell
./scripts/test-windows-installer.ps1 `
  -InnoCompilerPath 'C:/build-tools/Inno Setup/ISCC.exe' `
  -PublishDirectory './artifacts/windows/0.1.0/win-x64/publish'
git diff --check
```

The test compiles the real payload with a fresh installer identity, unique protocol/shortcut names, and an artifacts install directory. It does not launch SunCode or use the user's data directory. It verifies payload hashes, registrations/activation quoting, reinstall repair, downgrade refusal including a missing executable, architecture refusal, uninstall cleanup, and preservation of an unrelated file. A `finally` block uninstalls the test package after assertion failures. Ignored test archives/logs remain available for local inspection.

Real x64 packaging and isolated lifecycle checks pass with Inno Setup 7.1.1. Clean-machine coverage, Windows 10/11 visual wizard and notification behavior, ARM64, platform signing/SmartScreen reputation and trusted update enrollment remain release checks.

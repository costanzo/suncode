# macOS and Linux desktop packaging

Status: Packaging source implemented and statically inspected on Windows. Native builds, installed behavior and signing/notarization have not been executed or verified.

`scripts/package-macos.sh` and `scripts/package-linux.sh` share Bash 3.2-compatible host/version/publish/checksum validation. They require native OS/architecture and matching Rust host toolchains, include .NET and the Rust SDK, exclude CEF/CLI, check native payload architecture and write ignored version/RID-scoped artifacts.

macOS generates `SunCode.app` and a compressed DMG with an Applications link. Assembly metadata and copied plist versions follow the Desktop project. Final ad hoc signing depends on completed bundle/resource/symbol work for both macOS RIDs. Optional Developer ID signing handles Mach-O leaves before the hardened-runtime/JIT-entitled app envelope; keychain-profile notarization requires Accepted status and staples the app and DMG. No credentials enter source or packages. Finder owns installation/removal.

Linux generates Debian and portable tar.gz packages. Debian uses `/opt/suncode`, `/usr/bin/suncode`, an absolute desktop entry, existing icons, desktop/MIME/icon cache hooks, ELF-derived ABI dependencies and explicit dynamic prerequisites. dpkg owns package files; removal/purge leaves user data untouched. Portable archives use a directory-relative launcher and register nothing machine-wide. RPM/AppImage/musl support is not implemented.

Neither installer creates trusted component receipts, resets managed update pointers, accesses SQLite, launches a desktop session automatically or resolves Agent approvals. C# retains component-update ownership. The manually triggered `.github/workflows/desktop-packaging.yml` defines native ARM64/x64 jobs for both OSes; it has not been run and publishes no release.

Commands, dependencies, release identity options and native verification gaps are documented in `apps/desktop-avalonia/packaging/README.md` and its platform guides. Standard OS installation behavior is specified in `DESIGN.md` and `design-system/src/platforms/desktop/README.md`.

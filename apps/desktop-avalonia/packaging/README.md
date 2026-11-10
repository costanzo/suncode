# Desktop packaging

| Platform | Entry point | Distribution |
| --- | --- | --- |
| Windows | `scripts/package-windows.ps1` | Inno Setup offline installer |
| macOS | `bash scripts/package-macos.sh` | App bundle and compressed DMG |
| Linux | `bash scripts/package-linux.sh` | Debian package and portable tar.gz |

Run each on its native OS with a matching Rust MSVC/GNU/Apple host toolchain, .NET SDK 10 and platform build dependencies. Unix scripts detect the architecture; `--runtime` asserts that choice and does not enable cross-compilation. Versions follow `SunCode.Desktop.csproj`; outputs stay under ignored `artifacts/`. See [Windows](windows/README.md), [macOS](macos/README.md) and [Linux](linux/README.md) for commands and verification limits. Bash entry points support macOS's system Bash 3.2. Packages include the built-in managed CEF binding. Native Chromium is installed separately through Settings on supported targets. The Rust CLI keeps its separate release flow.

These are full **baseline distributions** containing Desktop and SDK together, not the independent component archives in `contracts/updates`. Installation/removal preserves the data directory and managed component pointers; reinstalling a baseline does not reset an activated pair. No trusted updater receipts are fabricated. Initial update enrollment remains separate release work.

`.github/workflows/desktop-packaging.yml` provides manually triggered native macOS ARM64/x64, Ubuntu 22.04 x64 and Ubuntu 24.04 ARM64/x64 jobs. Artifact names and Cargo cache keys include the runner OS so distribution baselines do not collide. It uploads packages/checksums, uses ad hoc macOS signing and does not publish releases. The workflow has not been executed in this Windows session. Complete platform packaging, installed behavior, official signing and deployment-OS compatibility remain unverified.

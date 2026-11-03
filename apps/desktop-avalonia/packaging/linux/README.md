# Linux packaging

Run on native GNU/glibc Linux x64 or ARM64 with .NET SDK 10, the matching Rust GNU host toolchain, C/C++ compiler, pkg-config and native development libraries. Default output is both a Debian package and portable tar.gz:

```sh
bash scripts/package-linux.sh
bash scripts/package-linux.sh --format deb --runtime linux-x64
bash scripts/package-linux.sh --format tar
```

Ubuntu 24.04 build/packaging dependencies (also installed by the native CI job):

```sh
sudo apt-get install build-essential pkg-config libssl-dev libdbus-1-dev \
  libx11-dev libxext-dev libxi-dev libxcursor-dev libxrandr-dev libxtst-dev \
  libxdo-dev libxkbcommon-dev libfontconfig1-dev libfreetype6-dev libicu-dev \
  zlib1g-dev libunwind-dev liblttng-ust-dev dpkg-dev desktop-file-utils hicolor-icon-theme
```

Other distributions use their corresponding dependencies and `--format tar`; `deb`/`all` requires Debian tools. ARM64 maps to the native `aarch64-unknown-linux-gnu` host and Debian `arm64`; x64 uses `x86_64-unknown-linux-gnu` and `amd64`. Mismatched hosts/RIDs are refused.

For version `0.1.0` on x64, outputs under `artifacts/linux/0.1.0/linux-x64/` are `publish/`, `SunCode-0.1.0-linux-x64.deb`, `SunCode-0.1.0-linux-x64.tar.gz` and their `.sha256` sidecars. .NET and Rust are included; native ELF architectures are checked. The build stages package roots without installing them. CEF and CLI are excluded.

## Debian installation and uninstall

```sh
sudo apt install ./SunCode-0.1.0-linux-x64.deb
suncode
sudo apt remove suncode
```

The package manager owns installation/upgrades/removal. Quit SunCode after Agent/Terminal work finishes before replacing/removing binaries; the scripts neither force-close it nor inspect Agent activity. Package paths:

- `/opt/suncode/`: Desktop, .NET and native SDK payload.
- `/usr/bin/suncode`: absolute launcher forwarding all arguments.
- `/usr/share/applications/dev.suncode.desktop.desktop`: `/usr/bin/suncode %u`, `TryExec` and the `suncode://` handler declaration.
- `/usr/share/icons/hicolor/`: existing multi-size/scalable icons.
- `/usr/share/doc/suncode/copyright`: project license.

Maintainer scripts refresh desktop/MIME and icon caches when tools are available, without changing each user's preferred handler or launching a desktop session. Removal/purge touches no user profile, project, credentials, downloaded shell or managed update state. No custom policy blocks an explicitly requested package-manager downgrade.

`dpkg-shlibdeps` scans every ELF and derives ABI dependency versions from the build distribution. Missing libraries remain fatal; missing dpkg ownership metadata for private bundled libraries is ignored. Explicit dependencies cover dynamically loaded ICU/OpenSSL and X11/font/D-Bus/desktop services. Build on the intended distribution baseline: Ubuntu-built packages are not automatically compatible with every Debian/Ubuntu release. `SUNCODE_DEB_MAINTAINER` can provide the release maintainer's actual `Name <email>`; local builds use the non-routing placeholder `SunCode <noreply@localhost>`.

## Portable archive and release limits

Extract the tar.gz and run `./suncode` inside `SunCode-<version>-<rid>/`. Its directory-relative wrapper forwards arguments and supports spaces in the extracted path. It installs no global launcher, desktop entry, icons or protocol handler. Remove the extracted folder to remove binaries; the separate data directory survives.

Self-contained .NET still needs compatible native system libraries: glibc/libstdc++, ICU/OpenSSL and font/X11/D-Bus dependencies. The archive installs none and is not a musl/Alpine or universal Linux package. RPM/AppImage output is not implemented.

`SUNCODE_UPDATE_DISTRIBUTION_FILE` selects existing update configuration. These full baseline packages are not independent component archives and create no trusted receipt. SHA-256 sidecars detect corruption rather than authenticate the publisher; repository/package signing and managed-update enrollment remain release work.

Only static Bash/XML/workflow/whitespace inspection was performed on Windows. Native builds, dependency resolution, package installation/upgrades/removal, extracted startup, desktop/protocol/notification activation and retention checks were not run. Manual native CI was not triggered; both architectures and clean-machine behavior require release validation.

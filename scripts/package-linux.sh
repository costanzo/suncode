#!/usr/bin/env bash
set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/packaging-common.sh"

runtime=''
format=all
while [[ $# -gt 0 ]]; do
  case "$1" in
    --runtime|--format)
      [[ $# -ge 2 && -n "$2" ]] || packaging_fail "$1 requires a value"
      case "$1" in --runtime) runtime="$2" ;; --format) format="$2" ;; esac
      shift 2 ;;
    --help|-h)
      printf '%s\n' 'Usage: bash scripts/package-linux.sh [--runtime linux-x64|linux-arm64] [--format all|deb|tar]'
      exit 0 ;;
    *) packaging_fail "unknown option: $1" ;;
  esac
done
case "$format" in all|deb|tar) ;; *) packaging_fail '--format must be all, deb or tar' ;; esac
packaging_init linux "$runtime"
for tool in readelf tar gzip sha256sum; do packaging_require "$tool"; done
if [[ "$format" != tar ]]; then
  for tool in dpkg-deb dpkg-shlibdeps dpkg-query desktop-file-validate; do packaging_require "$tool"; done
fi
packaging_publish_app
machine_pattern='Machine:.*Advanced Micro Devices X86-64'
[[ "$packaging_arch" != arm64 ]] || machine_pattern='Machine:.*AArch64'
for binary in SunCode libsuncode_agent.so libcoreclr.so libhostfxr.so; do
  readelf -h "$packaging_publish/$binary" | grep -q "$machine_pattern" || packaging_fail "unexpected architecture in $binary"
done
linux_assets="$packaging_root/apps/desktop-avalonia/packaging/linux"

if [[ "$format" != deb ]]; then
  archive_name="SunCode-$packaging_version-$packaging_rid"
  portable="$packaging_stage/portable/$archive_name"
  mkdir -p "$portable"
  cp -a "$packaging_publish/." "$portable/"
  # Portable archives have no machine-wide desktop/protocol registration.
  rm -f "$portable/share/applications/dev.suncode.desktop.desktop"
  install -m 755 "$linux_assets/portable-launcher.sh" "$portable/suncode"
  install -m 644 "$packaging_root/LICENSE" "$portable/LICENSE.txt"
  chmod 755 "$portable/SunCode"
  tar -C "$packaging_stage/portable" -czf "$packaging_stage/package.tar.gz" "$archive_name"
  archive="$packaging_output/$archive_name.tar.gz"
  mv -f "$packaging_stage/package.tar.gz" "$archive"
  packaging_checksum "$archive"
  printf 'Portable archive: %s\n' "$archive"
fi

if [[ "$format" != tar ]]; then
  deb_arch=amd64
  [[ "$packaging_arch" != arm64 ]] || deb_arch=arm64
  deb_root="$packaging_stage/deb"
  app_dir="$deb_root/opt/suncode"
  mkdir -p "$app_dir" "$deb_root/DEBIAN" "$deb_root/usr/bin" \
    "$deb_root/usr/share/applications" "$deb_root/usr/share/icons" "$deb_root/usr/share/doc/suncode"
  cp -a "$packaging_publish/." "$app_dir/"
  install -m 755 "$linux_assets/debian-launcher.sh" "$deb_root/usr/bin/suncode"
  install -m 644 "$linux_assets/dev.suncode.desktop.desktop" "$deb_root/usr/share/applications/dev.suncode.desktop.desktop"
  cp -a "$linux_assets/icons/hicolor" "$deb_root/usr/share/icons/"
  install -m 644 "$packaging_root/LICENSE" "$deb_root/usr/share/doc/suncode/copyright"
  install -m 755 "$linux_assets/postinst" "$deb_root/DEBIAN/postinst"
  install -m 755 "$linux_assets/postrm" "$deb_root/DEBIAN/postrm"
  chmod -R u+rwX,go+rX,go-w "$deb_root"
  chmod 755 "$app_dir/SunCode"
  desktop-file-validate "$deb_root/usr/share/applications/dev.suncode.desktop.desktop"

  # Scan every ELF payload, not just the managed apphost. Missing libraries
  # remain fatal. Private bundled libraries have no dpkg ownership metadata.
  mkdir -p "$packaging_stage/dependency-scan/debian"
  printf 'Source: suncode\n\nPackage: suncode\nArchitecture: %s\nDescription: SunCode desktop\n' "$deb_arch" > "$packaging_stage/dependency-scan/debian/control"
  elf_arguments=()
  while IFS= read -r -d '' binary; do
    if readelf -h "$binary" >/dev/null 2>&1; then elf_arguments+=("-e$binary"); fi
  done < <(find "$app_dir" -type f -print0)
  [[ ${#elf_arguments[@]} -gt 0 ]] || packaging_fail 'no native ELF payloads found'
  dependency_output="$(cd "$packaging_stage/dependency-scan" && dpkg-shlibdeps -O --ignore-missing-info -l"$app_dir" "${elf_arguments[@]}")"
  abi_dependencies="$(printf '%s\n' "$dependency_output" | sed -n 's/^shlibs:Depends=//p')"
  [[ -n "$abi_dependencies" ]] || packaging_fail 'could not derive Debian ABI dependencies'
  # ICU, OpenSSL and X11/font services can also be loaded through dlopen, so
  # ELF metadata alone does not describe the complete Desktop prerequisites.
  icu_package="$(dpkg-query -W -f='${Package} ${db:Status-Status}\n' 'libicu[0-9]*' | awk '$2 == "installed" { print $1 }' | sort -V | tail -n 1)"
  [[ "$icu_package" =~ ^libicu[0-9]+$ ]] || packaging_fail 'install the build distribution ICU runtime package'
  maintainer="${SUNCODE_DEB_MAINTAINER:-SunCode <noreply@localhost>}"
  [[ "$maintainer" != *$'\n'* && "$maintainer" != *$'\r'* ]] || packaging_fail 'Debian maintainer must be one line'
  cat > "$deb_root/DEBIAN/control" <<EOF
Package: suncode
Version: $packaging_version
Section: devel
Priority: optional
Architecture: $deb_arch
Maintainer: $maintainer
Homepage: https://www.suncode.ai
Depends: $abi_dependencies, $icu_package, libssl3 | libssl3t64, zlib1g, libfontconfig1, libfreetype6, libx11-6, libice6, libsm6, libxext6, libxi6, libxcursor1, libxrandr2, libxtst6, libxdo3, libxkbcommon0, libdbus-1-3, desktop-file-utils, hicolor-icon-theme
Description: SunCode desktop coding agent
 Native Avalonia desktop with the embedded Rust Agent SDK and .NET runtime.
EOF
  deb="$packaging_output/SunCode-$packaging_version-$packaging_rid.deb"
  dpkg-deb --build --root-owner-group "$deb_root" "$packaging_stage/package.deb"
  mv -f "$packaging_stage/package.deb" "$deb"
  packaging_checksum "$deb"
  printf 'Debian package: %s\n' "$deb"
fi

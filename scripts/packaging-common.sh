#!/usr/bin/env bash
# Shared by the native platform packaging entry points (Bash 3.2 compatible).
set -euo pipefail

packaging_fail() { printf 'packaging: %s\n' "$*" >&2; exit 1; }
packaging_require() {
  command -v "$1" >/dev/null 2>&1 || packaging_fail "$1 is required"
}

packaging_init() {
  local platform="$1" requested="${2:-}" machine rust_target rust_host part
  case "$platform:$(uname -s)" in
    macos:Darwin|linux:Linux) ;;
    *) packaging_fail "$platform packages must be built on $platform" ;;
  esac
  machine="$(uname -m)"
  case "$machine" in
    arm64|aarch64) packaging_arch=arm64 ;;
    x86_64) packaging_arch=x64 ;;
    *) packaging_fail "unsupported host architecture: $machine" ;;
  esac
  if [[ "$platform" == macos ]]; then
    packaging_rid="osx-$packaging_arch"
    if [[ "$packaging_arch" == arm64 ]]; then rust_target=aarch64-apple-darwin; else rust_target=x86_64-apple-darwin; fi
  else
    packaging_rid="linux-$packaging_arch"
    if [[ "$packaging_arch" == arm64 ]]; then rust_target=aarch64-unknown-linux-gnu; else rust_target=x86_64-unknown-linux-gnu; fi
  fi
  [[ -z "$requested" || "$requested" == "$packaging_rid" ]] || packaging_fail "native builds only: this host requires $packaging_rid"
  for part in dotnet cargo rustc; do packaging_require "$part"; done
  rust_host="$(rustc -vV | sed -n 's/^host: //p')"
  [[ "$rust_host" == "$rust_target" ]] || packaging_fail "Rust host must be $rust_target (found $rust_host)"
  [[ -z "${CARGO_BUILD_TARGET:-}" || "$CARGO_BUILD_TARGET" == "$rust_target" ]] || packaging_fail 'CARGO_BUILD_TARGET conflicts with the native build'
  packaging_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)"
  packaging_project="$packaging_root/apps/desktop-avalonia/SunCode.Desktop.csproj"
  packaging_version="$(DOTNET_NOLOGO=1 dotnet msbuild "$packaging_project" -nologo -getProperty:Version)"
  local version_pattern='^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$'
  [[ "$packaging_version" =~ $version_pattern ]] || packaging_fail 'Desktop Version must be MAJOR.MINOR.PATCH'
  for part in ${packaging_version//./ }; do
    [[ ${#part} -le 5 ]] && (( part <= 65535 )) || packaging_fail 'Desktop version components must be between 0 and 65535'
  done
  packaging_output="$packaging_root/artifacts/$platform/$packaging_version/$packaging_rid"
  packaging_publish="$packaging_output/publish"
  mkdir -p "$packaging_output"
  packaging_stage="$(mktemp -d "$packaging_output/stage.XXXXXX")"
  trap packaging_cleanup EXIT
  trap 'exit 130' INT
  trap 'exit 143' TERM
}

packaging_cleanup() {
  # Only this invocation's mktemp directory is disposable. Keep publish inputs
  # and completed artifacts; never accept a caller-supplied recursive target.
  if [[ -n "${packaging_stage:-}" && "$packaging_stage" == "$packaging_output"/stage.* ]]; then
    rm -rf -- "$packaging_stage"
  fi
}

packaging_publish_app() {
  local args=(dotnet publish "$packaging_project" -c Release -r "$packaging_rid"
    --self-contained true -o "$packaging_publish" -p:PublishSingleFile=false
    -p:PublishTrimmed=false -p:SunCodeBundleCef=false)
  if [[ -n "${SUNCODE_UPDATE_DISTRIBUTION_FILE:-}" ]]; then
    [[ -f "$SUNCODE_UPDATE_DISTRIBUTION_FILE" ]] || packaging_fail 'distribution configuration does not exist'
    args+=("-p:SunCodeUpdateDistributionFile=$(cd "$(dirname "$SUNCODE_UPDATE_DISTRIBUTION_FILE")" && pwd -P)/$(basename "$SUNCODE_UPDATE_DISTRIBUTION_FILE")")
  fi
  (cd "$packaging_root" && "${args[@]}")
}

packaging_checksum() {
  local directory filename
  directory="$(dirname "$1")"
  filename="$(basename "$1")"
  if command -v sha256sum >/dev/null 2>&1; then
    (cd "$directory" && sha256sum "$filename") > "$1.sha256"
  else
    (cd "$directory" && shasum -a 256 "$filename") > "$1.sha256"
  fi
}

#!/usr/bin/env bash
set -euo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/packaging-common.sh"

runtime=''
identity="${SUNCODE_MACOS_SIGN_IDENTITY:-}"
notary_profile="${SUNCODE_MACOS_NOTARY_PROFILE:-}"
while [[ $# -gt 0 ]]; do
  case "$1" in
    --runtime|--sign-identity|--notary-profile)
      [[ $# -ge 2 && -n "$2" ]] || packaging_fail "$1 requires a value"
      case "$1" in
        --runtime) runtime="$2" ;;
        --sign-identity) identity="$2" ;;
        --notary-profile) notary_profile="$2" ;;
      esac
      shift 2 ;;
    --help|-h)
      printf '%s\n' 'Usage: bash scripts/package-macos.sh [--runtime osx-arm64|osx-x64] [--sign-identity "Developer ID Application: ..."] [--notary-profile keychain-profile]'
      exit 0 ;;
    *) packaging_fail "unknown option: $1" ;;
  esac
done
[[ -z "$notary_profile" || ( -n "$identity" && "$identity" != - ) ]] || packaging_fail 'notarization requires a Developer ID signing identity'
packaging_init macos "$runtime"
for tool in xcrun codesign hdiutil ditto file lipo plutil shasum; do packaging_require "$tool"; done
[[ -z "$notary_profile" ]] || xcrun --find notarytool >/dev/null

packaging_publish_app
app="$packaging_publish/SunCode.app"
[[ -d "$app" ]] || packaging_fail 'publish did not create SunCode.app'
native_arch=x86_64
[[ "$packaging_arch" != arm64 ]] || native_arch=arm64
for binary in SunCode libsuncode_agent.dylib libcoreclr.dylib libhostfxr.dylib libsuncode_notifications.dylib; do
  # -verify_arch consumes all following arguments as architecture names.
  # Put the input first so its path cannot be parsed as another architecture.
  lipo "$app/Contents/MacOS/$binary" -verify_arch "$native_arch"
done
for key in CFBundleShortVersionString CFBundleVersion; do
  [[ "$(/usr/libexec/PlistBuddy -c "Print :$key" "$app/Contents/Info.plist")" == "$packaging_version" ]] || packaging_fail "bundle $key does not match Desktop version"
done

notarize() {
  local artifact="$1" report="$packaging_stage/notary-result.json"
  xcrun notarytool submit "$artifact" --keychain-profile "$notary_profile" --wait --output-format json > "$report"
  [[ "$(plutil -extract status raw -o - "$report")" == Accepted ]] || packaging_fail "notarization was not accepted for $(basename "$artifact")"
}

if [[ -n "$identity" && "$identity" != - ]]; then
  # Sign every native leaf first, then the app envelope/main executable with
  # the JIT entitlement. CEF/framework packaging is deliberately disabled.
  while IFS= read -r -d '' binary; do
    if file -b "$binary" | grep -q 'Mach-O'; then
      codesign --force --options runtime --timestamp --sign "$identity" "$binary"
    fi
  done < <(find "$app/Contents" -type f -print0)
  codesign --force --options runtime --timestamp --sign "$identity" \
    --entitlements "$packaging_root/apps/desktop-avalonia/packaging/macos/SunCode.entitlements" "$app"
  signature="$(codesign --display --verbose=4 "$app" 2>&1)"
  grep -q '^Authority=Developer ID Application:' <<< "$signature" || packaging_fail 'release signing requires a Developer ID Application certificate'
fi
codesign --verify --deep --strict "$app"
if [[ -n "$notary_profile" ]]; then
  ditto -c -k --keepParent "$app" "$packaging_stage/notarize-app.zip"
  notarize "$packaging_stage/notarize-app.zip"
  xcrun stapler staple "$app"
  xcrun stapler validate "$app"
  /usr/sbin/spctl --assess --type execute "$app"
fi

mkdir -p "$packaging_stage/image"
ditto "$app" "$packaging_stage/image/SunCode.app"
ln -s /Applications "$packaging_stage/image/Applications"
hdiutil create -volname SunCode -srcfolder "$packaging_stage/image" -fs HFS+ -format UDZO "$packaging_stage/SunCode.dmg"
if [[ -n "$identity" && "$identity" != - ]]; then
  codesign --force --timestamp --sign "$identity" "$packaging_stage/SunCode.dmg"
  codesign --verify --strict "$packaging_stage/SunCode.dmg"
fi
if [[ -n "$notary_profile" ]]; then
  notarize "$packaging_stage/SunCode.dmg"
  xcrun stapler staple "$packaging_stage/SunCode.dmg"
  xcrun stapler validate "$packaging_stage/SunCode.dmg"
fi
dmg="$packaging_output/SunCode-$packaging_version-$packaging_rid.dmg"
mv -f "$packaging_stage/SunCode.dmg" "$dmg"
packaging_checksum "$dmg"
printf 'App: %s\nDMG: %s\n' "$app" "$dmg"
if [[ -z "$identity" || "$identity" == - ]]; then
  printf '%s\n' 'Signing: ad hoc (local builds only; not Developer ID signed or notarized)'
elif [[ -z "$notary_profile" ]]; then
  printf '%s\n' 'Signing: Developer ID signed; notarization was not requested'
else
  printf '%s\n' 'Signing: Developer ID signed, notarized and stapled'
fi

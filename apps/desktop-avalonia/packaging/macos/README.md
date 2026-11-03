# macOS packaging

Run on native Apple Silicon or Intel macOS with .NET SDK 10, the matching Rust Apple host toolchain and Xcode Command Line Tools (including Swift, `codesign`, `hdiutil`, `ditto`, `lipo`, `plutil`). The script detects `osx-arm64` or `osx-x64` and refuses mismatched Rust/RID targets; it does not produce universal binaries or cross-compile.

```sh
bash scripts/package-macos.sh
# Optional native host assertion:
bash scripts/package-macos.sh --runtime osx-arm64
```

For version `0.1.0` on Apple Silicon, outputs are:

```text
artifacts/macos/0.1.0/osx-arm64/publish/SunCode.app
artifacts/macos/0.1.0/osx-arm64/SunCode-0.1.0-osx-arm64.dmg
artifacts/macos/0.1.0/osx-arm64/SunCode-0.1.0-osx-arm64.dmg.sha256
```

Publishing includes .NET, Rust and the notification bridge. The MSBuild target updates the copied plist version fields from the Desktop project before signing; source plist values are template placeholders. Assembly metadata follows the same version, and both macOS RIDs receive final ad hoc signing. The script checks bundle versions and apphost/SDK/.NET/notification-bridge architectures before making the DMG. CEF remains off and needs a separate framework/signing review before inclusion.

The DMG contains `SunCode.app` and an Applications link. Quit SunCode after active work finishes, drag the app into Applications, eject the image and start the installed app. Remove the bundle through Finder to uninstall. Data under `~/.suncode` or the configured data directory survives. This is a Finder installation flow, without a `.pkg` or Windows-style downgrade guard.

## Developer ID and notarization

Default builds are ad hoc signed for local development. They are not Developer ID/notarized public releases and may be rejected by Gatekeeper; no security-policy bypass is installed.

With a Developer ID Application certificate/private key already in the build user's keychain:

```sh
bash scripts/package-macos.sh \
  --sign-identity 'Developer ID Application: Your Company (TEAMID)' \
  --notary-profile suncode-notary
```

Create the profile separately through Apple's interactive `xcrun notarytool store-credentials suncode-notary` or an existing organizational keychain workflow. The script accepts a profile name, not raw credentials. Equivalent environment options are `SUNCODE_MACOS_SIGN_IDENTITY` and `SUNCODE_MACOS_NOTARY_PROFILE`. Omitting the profile signs without notarizing; requesting notarization without a real identity is refused.

Sign native Mach-O leaves before the app envelope, applying hardened runtime and the .NET JIT entitlement from `SunCode.entitlements` to the main executable. Verify the Developer ID authority and complete signature, notarize a temporary app ZIP, require status `Accepted`, staple/validate the app, then build/sign/notarize/staple the DMG. Checksums are calculated after stapling. Failures abort; credentials and private keys never enter packages or tracked files.

`SUNCODE_UPDATE_DISTRIBUTION_FILE` selects an existing Desktop update configuration. Managed updates additionally require the correct `macTeamId`, signed independent archives and a trusted initial receipt; a signed DMG does not enroll an unmanaged baseline.

Only static Bash/XML/workflow/whitespace inspection was performed on Windows. No native build, notarization request, DMG mount, installed startup/JIT, notification activation or uninstall was executed. The manual CI workflow was not triggered. Native release validation remains required.

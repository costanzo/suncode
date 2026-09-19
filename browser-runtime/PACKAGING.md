# Browser Runtime Packaging

Browser runtime artifacts are built on their target operating system and are never cross-packaged:

| SunCode target | Build host | Runtime output |
| --- | --- | --- |
| macOS arm64 | macOS arm64 | `artifacts/browser-runtime/darwin-arm64` |
| Windows x64 | Windows x64 | `artifacts/browser-runtime/win32-x64` |
| Linux x64 | Ubuntu 22.04 or 24.04 x64 | `artifacts/browser-runtime/linux-x64` |

Run:

```sh
npm --prefix browser-runtime run package -- --target <target>
npm --prefix browser-runtime run verify -- artifacts/browser-runtime/<target>
```

Release CI must additionally run the Rust probe and smoke executables against the assembled directory before publishing the desktop package.

```sh
cargo run --manifest-path agent/Cargo.toml -p suncode-browser --example probe -- artifacts/browser-runtime/<target>
cargo run --manifest-path agent/Cargo.toml -p suncode-browser --example smoke -- artifacts/browser-runtime/<target>
```

## macOS signing order

1. Sign executables, frameworks, libraries, and helper applications nested inside Playwright Chromium.
2. Sign the bundled Node.js executable and SunCode native library.
3. Sign the outer `SunCode.app` with hardened runtime enabled.
4. Verify with `codesign --verify --deep --strict` and Gatekeeper assessment.
5. Notarize and staple the final distribution artifact.

The macOS publish target copies the runtime with `ditto` so Chromium hard links are preserved. File-by-file MSBuild copying expands Chromium from roughly 300 MiB to roughly 900 MiB and is prohibited.
The runtime packager also uses target-native archive-preserving copies when reusing a browser download. Chromium framework symlinks must remain relative; absolute or tree-escaping symlinks fail manifest generation and verification.

## Windows signing

Sign SunCode-owned executables, native libraries, and the installer. Do not modify upstream Chromium files after the runtime tree hash has been created. Run the installed package offline and verify that no npm or Playwright download is attempted.

## Linux baseline

The initial Linux x64 baseline is Ubuntu 22.04 and 24.04. Release CI installs the Playwright Chromium dependency set during image construction, then validates the packaged runtime in an offline test container. Other distributions are unsupported until separately verified.

## Release evidence

Retain the runtime lock, generated target manifest, SBOM, license notices, signing verification, package size, and smoke-test output with each release. These files must not contain local absolute paths, credentials, process IDs, or developer cache paths.

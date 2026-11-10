# SunCode Avalonia Desktop

The Phase 1 desktop client targets .NET 10 and Avalonia. It embeds the Rust agent through the method-oriented C ABI and owns presentation, navigation, transient interaction state, and desktop-local capabilities such as update coordination.

The Avalonia client owns the desktop presentation and embeds the Rust agent through the native SDK boundary.

Settings → Updates and About → Software updates use one C# coordinator for independent Desktop/native SDK packages. The client implements signed metadata checks, compatible plans, verified staging, SDK repair, and confirmed restart with immutable payloads. The checked-in build enables catalog checks against `https://www.suncode.ai`, with apex and www archive origins allowed. Current catalog package values are samples; real archives, initial-installer receipts, platform signing identities, and signed installed-platform verification are still required; see the [updater guide](Infrastructure/Updates/README.md) for configuration, package layout, and limits. No Rust update implementation or C# SQLite access is introduced.

## Requirements

- .NET SDK 10
- Rust stable and Cargo

## Build and run

Prepare the bundled Browser Use runtime once before the first Debug build or after changing its lock or worker:

```sh
```

The Avalonia client starts CEF with remote debugging on localhost port 9222. Rust connects to CEF through CDP; no Node.js or Playwright runtime is packaged.

All desktop browser code lives in this project under `Infrastructure/Browser/`, `Infrastructure/CefPreviewRuntime.cs`, and the existing browser views. The managed CEF binding is built directly into Desktop and its managed dependencies use normal application assembly resolution. Chromium is installed explicitly in Settings → Browser use from NuGet.org or Huawei Cloud, with the same application-pinned SHA-256 for either source. Builds no longer need a local CefGlue checkout or a CEF bundle flag. The adapter uses Costanzo.CefGlue.Avalonia 1.0.0-preview.1, CefGlue.Next 152.7977.831 and CEF 152.0.6. Runtime payloads remain in the user's application data directory and are excluded from build/publish output. Windows x64/ARM64 and Linux x64/ARM64 have pinned packages. External macOS Framework loading requires a binding change and currently reports unsupported.

```sh
dotnet build apps/desktop-avalonia/SunCode.Desktop.csproj
dotnet run --project apps/desktop-avalonia/SunCode.Desktop.csproj
```

The managed SDK runs Cargo from the repository root, uses the shared root `target/`, and copies the native library from `target/debug` or `target/release`. This also works when `dotnet` is invoked from another directory. To override the output, use `CARGO_TARGET_DIR` or `-p:SunCodeCargoTargetDirectory=...`; relative overrides resolve from the repository root. Development builds retain line-table Rust debug information and incremental compilation.

Run the Avalonia test project from its colocated `tests/` directory:

```bash
dotnet test apps/desktop-avalonia/tests/SunCode.Desktop.Tests.csproj
```

Create the macOS app bundle with the .NET 10 runtime included. Release publishes with a RID are self-contained, untrimmed and keep native libraries separate. Each Release publish clears its publish directory first. Assembly metadata and the copied bundle plist follow the project version. Direct publish produces `SunCode.app`; use the packaging command below for a distributable DMG.

```sh
dotnet publish apps/desktop-avalonia/SunCode.Desktop.csproj -c Release -r osx-arm64 --self-contained true
open apps/desktop-avalonia/bin/Release/net10.0/osx-arm64/publish/SunCode.app
```

The build compiles the `sdks/c` native library (`suncode_agent`, which embeds the `suncode-core` harness) and copies it beside the managed executable. The client does not access SQLite or providers directly; Explorer/project file reads use the SDK. Source Control is an explicit desktop exception: a narrow C# service performs bounded read-only inspection through installed system Git, with no Git library or Git SDK methods. Git must be executable and support porcelain v2 (Git 2.11+); otherwise its controls, drawer, and footer data are hidden. Detection repeats when a project window becomes active.

Implemented workflows include the project hub, independent draggable project windows over one shared agent handle, project/session navigation, ordered conversation streaming, completed-response Markdown rendering, model selection, Enter-to-send and Shift+Enter newline handling, turn submission and cancellation, approvals, checkpoint undo, touched-file review, agent diagnostics, Git status and structured file diffs, provider credentials, default model selection, dark/light appearance, dialogs and traffic lights, full-screen geometry, the IDE-style native menu hierarchy, the macOS Dock menu and Windows taskbar Jump List for open and recent projects, and background session-attention notifications with notification-click navigation. Notification activation uses a desktop-only Named Pipe on Windows and Unix Domain Socket on macOS/Linux; it never exposes the embedded agent over IPC.

### Windows installer

Build one offline Inno Setup installer with `./scripts/package-windows.ps1` on Windows. It bundles .NET, the Rust SDK and the app-local Visual C++ CRT, registers current-user shortcuts/activation and uninstall, and preserves session data on uninstall. See the [Windows packaging guide](packaging/windows/README.md) for prerequisites, outputs, lifecycle checks and signing/update-enrollment limits.

### macOS and Linux packages

Run `bash scripts/package-macos.sh` on macOS for an app bundle and DMG, or `bash scripts/package-linux.sh` on GNU Linux for a Debian package and portable tar.gz. Both require matching native toolchains. macOS provides optional Developer ID signing/keychain-profile notarization; Debian packages own launcher/desktop/protocol/icon integration and removal. These new flows have only static verification in this Windows session. See the [packaging guide](packaging/README.md) for native prerequisites and the manual `Desktop macOS and Linux Packaging` workflow.

### Bundled fonts

The desktop client bundles static Noto Sans and Noto Sans SC
regular/medium/semibold/bold faces, JetBrains Mono, and Seti UI without
committing font binaries to Git. Build and publish targets run
`scripts/prepare-fonts.sh` on macOS/Linux or `scripts/prepare-fonts.ps1` on
Windows. The scripts download immutable upstream revisions into the ignored
`apps/desktop-avalonia/Assets/fonts/` directory and verify the SHA-256 values in
`scripts/fonts.lock` before Avalonia embeds the files in the application.

UI text, Markdown, tooltips, and shared selectors use `SansFontFamily`: bundled
Noto Sans first, then installed Noto Sans CJK SC or bundled Noto Sans SC for
Chinese glyphs. Code, paths, model identifiers, and pairing URLs use the shared
`MonoFontFamily` resource backed by JetBrains Mono.

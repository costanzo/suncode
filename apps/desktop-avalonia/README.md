# SunCode Avalonia Desktop

The Phase 1 desktop client targets .NET 10 and Avalonia. It embeds the Rust agent through the method-oriented C ABI and owns presentation, navigation, and transient interaction state only.

The Avalonia client owns the desktop presentation and embeds the Rust agent through the native SDK boundary.

## Requirements

- .NET SDK 10
- Rust stable and Cargo

## Build and run

Prepare the bundled Browser Use runtime once before the first Debug build or after changing its lock or worker:

```sh
```

The Avalonia client starts CEF with remote debugging on localhost port 9222. Rust connects to CEF through CDP; no Node.js or Playwright runtime is packaged.

```sh
dotnet build apps/desktop-avalonia/SunCode.Desktop.csproj
dotnet run --project apps/desktop-avalonia/SunCode.Desktop.csproj
```

The managed SDK runs Cargo from the repository root, uses the shared root `target/`, and copies the native library from `target/debug` or `target/release`. This also works when `dotnet` is invoked from another directory. To override the output, use `CARGO_TARGET_DIR` or `-p:SunCodeCargoTargetDirectory=...`; relative overrides resolve from the repository root. Development builds retain line-table Rust debug information and incremental compilation.

The referenced CefGlue fork's NuGet configuration must use `nuget.org` as both the package-source key and the package-source-mapping key for `https://api.nuget.org/v3/index.json`. Naming the same URL `Nuget` in that fork can conflict with the standard user-level `nuget.org` source and cause NU1100 during normal restore, especially after clearing package caches.

Run the Avalonia test project from its colocated `tests/` directory:

```bash
dotnet test apps/desktop-avalonia/tests/SunCode.Desktop.Tests.csproj
```

Create the macOS app bundle with the .NET 10 runtime included. Release publishes that specify a runtime identifier are self-contained, are not trimmed, and are not packed into a single file, so the Rust agent library and CEF stay as separate native binaries. Each Release publish clears its publish directory first, so assemblies left by an older dependency set cannot remain beside the new app. The current desktop version is 0.1.0. Give users `SunCode.app`; the files next to the bundle are the publish inputs that the bundle copies.

```sh
dotnet publish apps/desktop-avalonia/SunCode.Desktop.csproj -c Release -r osx-arm64 --self-contained true
open apps/desktop-avalonia/bin/Release/net10.0/osx-arm64/publish/SunCode.app
```

The build compiles the `sdks/c` native library (`suncode_agent`, which embeds the `suncode-core` harness) and copies it beside the managed executable. The client does not access SQLite or providers directly; Explorer/project file reads use the SDK. Source Control is an explicit desktop exception: a narrow C# service performs bounded read-only inspection through installed system Git, with no Git library or Git SDK methods. Git must be executable and support porcelain v2 (Git 2.11+); otherwise its controls, drawer, and footer data are hidden. Detection repeats when a project window becomes active.

Implemented workflows include the project hub, independent draggable project windows over one shared agent handle, project/session navigation, ordered conversation streaming, completed-response Markdown rendering, model selection, Enter-to-send and Shift+Enter newline handling, turn submission and cancellation, approvals, checkpoint undo, touched-file review, agent diagnostics, Git status and structured file diffs, provider credentials, default model selection, dark/light appearance, dialogs and traffic lights, full-screen geometry, the IDE-style native menu hierarchy, the macOS Dock menu for open and recent projects, and background session-attention notifications with notification-click navigation. Notification activation uses a desktop-only Named Pipe on Windows and Unix Domain Socket on macOS/Linux; it never exposes the embedded agent over IPC.

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

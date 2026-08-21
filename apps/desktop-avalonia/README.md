# SunCode Avalonia Desktop

The Phase 1 desktop client targets .NET 10 and Avalonia. It embeds the Rust runtime through the method-oriented C ABI and owns presentation, navigation, and transient interaction state only.

The Avalonia client owns the desktop presentation and embeds the Rust runtime through the native SDK boundary.

## Requirements

- .NET SDK 10
- Rust stable and Cargo

## Build and run

```sh
dotnet build apps/desktop-avalonia/SunCode.Desktop.csproj
dotnet run --project apps/desktop-avalonia/SunCode.Desktop.csproj
```

Create the macOS app bundle with:

```sh
dotnet publish apps/desktop-avalonia/SunCode.Desktop.csproj -c Release -r osx-arm64 --self-contained false
open apps/desktop-avalonia/bin/Release/net10.0/osx-arm64/publish/SunCode.app
```

The build compiles `suncode-runtime` as a dynamic native library and copies it beside the managed executable. The client does not access SQLite, providers, Git, or project files directly.

Implemented workflows include the project hub, independent draggable project windows over one shared runtime handle, project/session navigation, ordered conversation streaming, completed-response Markdown rendering, model selection, Enter-to-send and Shift+Enter newline handling, turn submission and cancellation, approvals, checkpoint undo, touched-file review, runtime diagnostics, Git status and structured file diffs, provider credentials, default model selection, dark/light appearance, dialogs and traffic lights, full-screen geometry, and the native macOS project menu.

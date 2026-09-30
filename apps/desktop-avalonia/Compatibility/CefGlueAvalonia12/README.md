# CefGlue Avalonia 12 compatibility layer

This is the MIT-licensed `CefGlue.Avalonia` adapter from Deon-Berlin/CefGlue commit `9666a8171c61e20e3f822d2ec7196a19354f9ce3`, plus its `BaseCefBrowser` source. It is built locally because the published `CefGlue.Next.Avalonia` assembly targets Avalonia 11 and fails at runtime under SunCode's Avalonia 12.1.1. The native CEF 152.0.6 and CefGlue core/common packages remain pinned NuGet dependencies.

The local changes replace Avalonia 11 focus, window, scaling, drag/drop, and popup APIs with Avalonia 12 equivalents. Browser drag/drop file transfer is intentionally disabled in the compatibility layer until a reviewed preview upload flow exists.

The upstream source and local modifications are governed by the included MIT license. When updating CEF or Avalonia, compare this layer against upstream before changing the package pins.

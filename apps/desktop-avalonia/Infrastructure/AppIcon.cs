using Avalonia.Controls;
using Avalonia.Platform;

namespace SunCode.Desktop.Infrastructure;

/// <summary>Shared window icon so every window loads the bundled logo from one place.</summary>
internal static class AppIcon
{
    private static readonly Uri IconUri = new("avares://SunCode/Assets/logo/suncode-logo-128.png");
    private static WindowIcon? _windowIcon;

    internal static WindowIcon Window => _windowIcon ??= new WindowIcon(AssetLoader.Open(IconUri));
}

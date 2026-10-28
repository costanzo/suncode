using Avalonia;
using Avalonia.Controls;
using SunCode.Desktop.Infrastructure;

namespace SunCode.Desktop.Views.ProjectWorkspace;

internal readonly record struct WorkspaceWindowGeometry(Size Size, PixelPoint? Position, WindowState State)
{
    internal static WorkspaceWindowGeometry Resolve(UiProjectState saved, PixelRect? workingArea, double scaling)
    {
        // Window dimensions are DIP; screen areas and saved positions are physical pixels.
        var maxWidth = Math.Max(620, Math.Min(2400, workingArea?.Width / scaling ?? 2400));
        var maxHeight = Math.Max(620, Math.Min(1600, workingArea?.Height / scaling ?? 1600));
        var width = Math.Clamp(saved.WindowWidth ?? 1440, 620, maxWidth);
        var height = Math.Clamp(saved.WindowHeight ?? 900, 620, maxHeight);
        PixelPoint? position = saved.WindowX is { } x && saved.WindowY is { } y
            ? new PixelPoint((int)Math.Round(x), (int)Math.Round(y))
            : null;
        if (workingArea is { } area)
        {
            var pixelWidth = (int)Math.Round(width * scaling);
            var pixelHeight = (int)Math.Round(height * scaling);
            position = position is { } point
                ? new PixelPoint(
                    Math.Clamp(point.X, area.X, Math.Max(area.X, area.Right - pixelWidth)),
                    Math.Clamp(point.Y, area.Y, Math.Max(area.Y, area.Bottom - pixelHeight)))
                : new PixelPoint(
                    area.X + Math.Max(0, (area.Width - pixelWidth) / 2),
                    area.Y + Math.Max(0, (area.Height - pixelHeight) / 2));
        }
        var state = saved.WindowState switch
        {
            "maximized" => WindowState.Maximized,
            "fullscreen" => WindowState.FullScreen,
            _ => WindowState.Normal
        };
        return new WorkspaceWindowGeometry(new Size(width, height), position, state);
    }
}

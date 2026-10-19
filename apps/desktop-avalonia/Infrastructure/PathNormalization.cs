namespace SunCode.Desktop.Infrastructure;

internal static class PathNormalization
{
    public static string FullPath(string path) => NormalizeWindowsExtendedPath(Path.GetFullPath(path));

    public static string NormalizeWindowsExtendedPath(string path)
    {
        if (!OperatingSystem.IsWindows() || !path.StartsWith(@"\\?\", StringComparison.Ordinal)) return path;
        return path.StartsWith(@"\\?\UNC\", StringComparison.OrdinalIgnoreCase)
            ? @"\" + path[7..]
            : path[4..];
    }
}

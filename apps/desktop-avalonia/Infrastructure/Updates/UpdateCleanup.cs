namespace SunCode.Desktop.Infrastructure.Updates;

internal static class UpdateCleanup
{
    public static void Prune(UpdateStateStore store)
    {
        var state = store.Read();
        if (state.Activation is not null) return;
        var protectedPaths = new[] { state.Installation?.Desktop.Directory, state.Installation?.Sdk?.Directory,
            state.PreviousInstallation?.Desktop.Directory, state.PreviousInstallation?.Sdk?.Directory }
            .Where(path => path is not null).Select(path => Path.GetFullPath(path!)).ToArray();
        var versions = Path.Combine(store.DirectoryPath,"versions");
        if (Directory.Exists(versions))
        {
            var directories = Directory.GetDirectories(versions).Where(path => Guid.TryParseExact(Path.GetFileName(path),"N",out _))
                .OrderByDescending(Directory.GetCreationTimeUtc).ToArray();
            foreach (var directory in directories.Skip(2))
                if (!protectedPaths.Any(path => path.StartsWith(directory + Path.DirectorySeparatorChar, StringComparison.OrdinalIgnoreCase))) DeleteOwned(directory);
        }
        var staging = Path.Combine(store.DirectoryPath,"staging");
        if (Directory.Exists(staging))
            foreach (var directory in Directory.GetDirectories(staging))
                if (Guid.TryParseExact(Path.GetFileName(directory),"N",out _) && directory != state.Staged?.Directory) DeleteOwned(directory);
    }
    private static void DeleteOwned(string path)
    {
        if ((File.GetAttributes(path) & FileAttributes.ReparsePoint) != 0) return;
        Directory.Delete(path, recursive: true);
    }
}

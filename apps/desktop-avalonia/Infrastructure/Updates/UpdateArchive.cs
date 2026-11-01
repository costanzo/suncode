using System.Formats.Tar;
using System.IO.Compression;

namespace SunCode.Desktop.Infrastructure.Updates;

internal static class UpdateArchive
{
    private const long MaximumExpandedBytes = 32L * 1024 * 1024 * 1024;
    private const int MaximumEntries = 100000;
    internal static string Inside(string root, string relative)
    {
        if (string.IsNullOrEmpty(relative) || relative.Length > 4096 || relative.Contains('\\') || relative.Contains(':')
            || relative.StartsWith('/') || relative.Split('/').Any(p => p is ".." or "." || p.EndsWith(' ') || p.EndsWith('.')))
            throw Invalid();
        var result = Path.GetFullPath(Path.Combine(root, relative));
        var prefix = Path.GetFullPath(root).TrimEnd(Path.DirectorySeparatorChar) + Path.DirectorySeparatorChar;
        if (!result.StartsWith(prefix, OperatingSystem.IsWindows() ? StringComparison.OrdinalIgnoreCase : StringComparison.Ordinal)) throw Invalid();
        if (OperatingSystem.IsWindows() && relative.Split('/').Any(p =>
            System.Text.RegularExpressions.Regex.IsMatch(p, "^(CON|PRN|AUX|NUL|COM[1-9]|LPT[1-9])($|\\.)", System.Text.RegularExpressions.RegexOptions.IgnoreCase))) throw Invalid();
        return result;
    }

    public static async Task ExtractAsync(string archive, string format, string destination, CancellationToken token)
    {
        if (Directory.Exists(destination)) throw Invalid();
        Directory.CreateDirectory(destination);
        var seen = new HashSet<string>(StringComparer.OrdinalIgnoreCase);
        var links = new List<(string Path, string Target)>();
        long expanded = 0;
        async Task Entry(string name, bool directory, bool link, string? linkTarget, UnixFileMode mode, Stream? data, long length)
        {
            token.ThrowIfCancellationRequested();
            while (name.StartsWith("./", StringComparison.Ordinal)) name = name[2..];
            name = name.TrimEnd('/');
            if (name.Length == 0 || name == ".") return;
            var path = Inside(destination, name);
            if (!seen.Add(name) || seen.Count > MaximumEntries || length < 0) throw Invalid();
            if (directory) { Directory.CreateDirectory(path); return; }
            Directory.CreateDirectory(Path.GetDirectoryName(path)!);
            if (link)
            {
                if (linkTarget is null || Path.IsPathRooted(linkTarget) || linkTarget.Contains('\\') || linkTarget.Contains(':') || linkTarget.Split('/').Contains("..")) throw Invalid();
                var resolved = Path.GetFullPath(Path.Combine(Path.GetDirectoryName(path)!, linkTarget));
                var relative = Path.GetRelativePath(destination, resolved).Replace(Path.DirectorySeparatorChar, '/');
                _ = Inside(destination, relative);
                links.Add((path, linkTarget)); return;
            }
            if (data is null) throw Invalid();
            expanded = checked(expanded + length);
            if (expanded > MaximumExpandedBytes) throw Invalid();
            await using (var output = new FileStream(path, FileMode.CreateNew, FileAccess.Write, FileShare.None, 81920, true))
            {
                var buffer = new byte[81920]; long written = 0; int count;
                while ((count = await data.ReadAsync(buffer, token)) != 0)
                {
                    written = checked(written + count);
                    if (written > length) throw Invalid();
                    await output.WriteAsync(buffer.AsMemory(0, count), token);
                }
                if (written != length) throw Invalid();
            }
            if (!OperatingSystem.IsWindows()) File.SetUnixFileMode(path, mode & (UnixFileMode)0x1ff);
        }
        try
        {
            if (format == "zip")
            {
                using var zip = ZipFile.OpenRead(archive);
                foreach (var entry in zip.Entries)
                {
                    var mode = (entry.ExternalAttributes >> 16) & 0xffff;
                    var type = mode & 0xf000;
                    if (type is not (0 or 0x8000 or 0x4000 or 0xa000)) throw Invalid();
                    var isDirectory = entry.FullName.EndsWith('/') || type == 0x4000;
                    await using var stream = entry.Open();
                    string? target = null;
                    if (type == 0xa000)
                    {
                        if (entry.Length > 4096) throw Invalid();
                        using var reader = new StreamReader(stream, new System.Text.UTF8Encoding(false, true));
                        target = await reader.ReadToEndAsync(token);
                    }
                    var permissions = mode == 0 ? (UnixFileMode)0x1a4 : (UnixFileMode)(mode & 0x1ff);
                    await Entry(entry.FullName, isDirectory, type == 0xa000, target, permissions, stream, entry.Length);
                }
            }
            else if (format == "tar.gz")
            {
                await using var file = File.OpenRead(archive);
                await using var gzip = new GZipStream(file, CompressionMode.Decompress);
                using var tar = new TarReader(gzip);
                TarEntry? entry;
                while ((entry = await tar.GetNextEntryAsync(copyData: false, token)) is not null)
                {
                    if (entry.EntryType is not (TarEntryType.RegularFile or TarEntryType.V7RegularFile or TarEntryType.Directory or TarEntryType.SymbolicLink)) throw Invalid();
                    await Entry(entry.Name, entry.EntryType == TarEntryType.Directory, entry.EntryType == TarEntryType.SymbolicLink,
                        entry.EntryType == TarEntryType.SymbolicLink ? entry.LinkName : null, entry.Mode, entry.DataStream, entry.Length);
                }
            }
            else throw Invalid();
            // Links are created only after all file writes, so no archive entry can
            // traverse a previously extracted link. Reject chains/cycles and escapes.
            string ResolveLink(string path, HashSet<string> visited)
            {
                if (visited.Count > 64) throw Invalid();
                foreach (var next in links)
                {
                    if (path != next.Path && !path.StartsWith(next.Path + Path.DirectorySeparatorChar, StringComparison.Ordinal)) continue;
                    if (!visited.Add(next.Path)) throw Invalid();
                    return ResolveLink(Path.GetFullPath(Path.Combine(Path.GetDirectoryName(next.Path)!, next.Target)) + path[next.Path.Length..], visited);
                }
                return path;
            }
            foreach (var link in links)
            {
                var target = ResolveLink(Path.GetFullPath(Path.Combine(Path.GetDirectoryName(link.Path)!, link.Target)), new(StringComparer.Ordinal) { link.Path });
                if (!File.Exists(target) && !Directory.Exists(target) || link.Path.StartsWith(target + Path.DirectorySeparatorChar, StringComparison.Ordinal)) throw Invalid();
                if (Directory.Exists(target)) Directory.CreateSymbolicLink(link.Path, link.Target); else File.CreateSymbolicLink(link.Path, link.Target);
            }
        }
        catch
        {
            if (Directory.Exists(destination)) Directory.Delete(destination, recursive: true);
            throw;
        }
    }
    public static Dictionary<string, string> Manifest(string root)
    {
        var result = new Dictionary<string, string>(StringComparer.Ordinal);
        void Scan(string directory)
        {
            foreach (var path in Directory.EnumerateFileSystemEntries(directory))
            {
                var relative = Path.GetRelativePath(root, path).Replace(Path.DirectorySeparatorChar, '/');
                FileSystemInfo info = Directory.Exists(path) ? new DirectoryInfo(path) : new FileInfo(path);
                if (info.LinkTarget is { } target)
                    result[relative] = "link:" + Convert.ToHexStringLower(System.Security.Cryptography.SHA256.HashData(System.Text.Encoding.UTF8.GetBytes(target)));
                else if (Directory.Exists(path)) Scan(path);
                else result[relative] = UpdateStateStore.HashFile(path);
            }
        }
        Scan(root);
        return result;
    }
    private static UpdateException Invalid() => new("archive_invalid", "The update archive could not be safely extracted.");
}

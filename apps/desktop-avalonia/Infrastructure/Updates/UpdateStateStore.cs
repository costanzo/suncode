using System.Security.Cryptography;
using System.Runtime.InteropServices;

namespace SunCode.Desktop.Infrastructure.Updates;

internal sealed record CatalogCache(long Revision, string SignedSha256, string? ETag, byte[] Envelope);
internal sealed record StagedUpdate(byte[] Envelope, bool Desktop, bool Sdk, bool Repair, string Directory, UpdateInstallation Baseline);
internal sealed record UpdateState
{
    public int SchemaVersion { get; init; } = 1;
    public bool AutomaticChecks { get; set; } = true;
    public Dictionary<string, CatalogCache> Catalogs { get; init; } = new(StringComparer.Ordinal);
    public UpdateInstallation? Installation { get; set; }
    public UpdateInstallation? PreviousInstallation { get; set; }
    public StagedUpdate? Staged { get; set; }
    public ActivationJournal? Activation { get; set; }
    public string? LastResult { get; set; }
}

internal sealed class UpdateStateStore(string directory)
{
    public string DirectoryPath { get; } = Path.GetFullPath(directory);
    private string StatePath => Path.Combine(DirectoryPath, "state.json");
    public UpdateState Read()
    {
        using var fileLock = Lock();
        return ReadUnlocked();
    }
    public void Change(Action<UpdateState> change)
    {
        using var fileLock = Lock();
        var state = ReadUnlocked();
        change(state);
        AtomicWrite(StatePath, UpdateJson.Write(state));
    }
    public CatalogCache? Cached(string url) => Read().Catalogs.GetValueOrDefault(url);
    public void Observe(string url, VerifiedCatalog verified, string? etag)
        => ObserveAuthenticated(url, verified.Catalog.Revision, verified.SignedSha256, verified.Envelope, etag);
    public void ObserveAuthenticated(string url, long revision, string hash, byte[] envelope, string? etag)
    {
        Change(state =>
        {
            if (state.Catalogs.TryGetValue(url, out var previous))
            {
                if (revision < previous.Revision)
                    throw new UpdateException("catalog_rollback", "An older update catalog was refused.");
                if (revision == previous.Revision && hash != previous.SignedSha256)
                    throw new UpdateException("publication_conflict", "The update publication is inconsistent.");
            }
            state.Catalogs[url] = new(revision, hash, etag, envelope);
        });
    }
    private UpdateState ReadUnlocked()
    {
        if (!File.Exists(StatePath)) return new();
        if (new FileInfo(StatePath).Length > 16 * 1024 * 1024) throw new UpdateException("state_invalid", "Update state is damaged. Reinstall SunCode to repair it.");
        try
        {
            var state = UpdateJson.Read<UpdateState>(File.ReadAllBytes(StatePath));
            if (state.SchemaVersion != 1 || state.Catalogs is null) throw new InvalidDataException();
            return state;
        }
        catch (Exception e) when (e is System.Text.Json.JsonException or InvalidDataException)
        { throw new UpdateException("state_invalid", "Update state is unsupported or damaged. Reinstall SunCode to repair it.", e); }
    }
    private FileStream Lock()
    {
        Directory.CreateDirectory(DirectoryPath);
        var watch = System.Diagnostics.Stopwatch.StartNew();
        while (true)
        {
            try { return new FileStream(Path.Combine(DirectoryPath, "state.lock"), FileMode.OpenOrCreate, FileAccess.ReadWrite, FileShare.None); }
            catch (IOException e)
            {
                if (watch.Elapsed >= TimeSpan.FromSeconds(2)) throw new UpdateException("update_locked", "Another update operation is running. Try again shortly.", e);
                Thread.Sleep(10);
            }
        }
    }
    internal static void AtomicWrite(string path, byte[] bytes)
    {
        if (bytes.Length > 16 * 1024 * 1024) throw new UpdateException("state_too_large", "The update state exceeds the supported size.");
        Directory.CreateDirectory(Path.GetDirectoryName(path)!);
        var temporary = path + "." + Guid.NewGuid().ToString("N") + ".tmp";
        try
        {
            using (var stream = new FileStream(temporary, FileMode.CreateNew, FileAccess.Write, FileShare.None))
            { stream.Write(bytes); stream.Flush(flushToDisk: true); }
            if (OperatingSystem.IsWindows())
            {
                if (!MoveFileEx(temporary, path, 0x1 | 0x8)) throw new IOException("Could not durably save update state.", new System.ComponentModel.Win32Exception(Marshal.GetLastPInvokeError()));
            }
            else
            {
                File.Move(temporary, path, overwrite: true);
                var descriptor = OpenDirectory(Path.GetDirectoryName(path)!, 0);
                if (descriptor < 0) throw new IOException("Could not open update state directory for synchronization.");
                try { if (SyncDirectory(descriptor) != 0) throw new IOException("Could not synchronize update state directory."); }
                finally { CloseDirectory(descriptor); }
            }
        }
        finally { if (File.Exists(temporary)) File.Delete(temporary); }
    }
    internal static string HashFile(string path)
    {
        using var stream = File.OpenRead(path);
        return Convert.ToHexStringLower(SHA256.HashData(stream));
    }
    [DllImport("kernel32.dll", EntryPoint = "MoveFileExW", CharSet = CharSet.Unicode, SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool MoveFileEx(string source, string destination, uint flags);
    [DllImport("libc", EntryPoint = "open", SetLastError = true)] private static extern int OpenDirectory(string path, int flags);
    [DllImport("libc", EntryPoint = "fsync", SetLastError = true)] private static extern int SyncDirectory(int descriptor);
    [DllImport("libc", EntryPoint = "close")] private static extern int CloseDirectory(int descriptor);
}

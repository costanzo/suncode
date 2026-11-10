using System.Security.Cryptography;
using System.Text.Json;
using SunCode.Sdk.Models;
using SunCode.Desktop.Infrastructure.Updates;

namespace SunCode.Desktop.Infrastructure;

internal sealed record BrowserComponentStatus(string State, long Downloaded = 0, long? Total = null, string? Error = null);
internal sealed record BrowserComponentInspection(string State, string? Directory = null);

// One owner across every Settings/project window. Closing Settings retains the download.
internal sealed class BrowserComponentService
{
    internal static BrowserComponentService Current { get; } = new(Path.Combine(AppDataPaths.DataDirectory, "browser-components"));
    private readonly string _root;
    private readonly BrowserComponentPackage? _package;
    private readonly Func<DownloadNetworkConfiguration, HttpMessageHandler> _handler;
    private readonly object _gate = new();
    private CancellationTokenSource? _cancellation;
    private Task? _operation;
    private BrowserComponentStatus? _status;
    private string? _loadedDirectory;
    private string? _invalidDirectory;
    private bool _closed;
    internal string Source { get; private set; } = "official";
    internal bool Busy { get { lock (_gate) return _operation is { IsCompleted: false }; } }
    internal BrowserComponentService(string root, BrowserComponentPackage? package = null,
        Func<DownloadNetworkConfiguration, HttpMessageHandler>? handler = null)
    {
        _root = Path.GetFullPath(root);
        _package = package ?? BrowserComponentCatalog.Current;
        _handler = handler ?? (network => PowerShellInstaller.CreateHandler(network with { VerifyCertificates = true }));
        try
        {
            var path = Path.Combine(_root, "source.json");
            if (File.Exists(path) && new FileInfo(path).Length < 256)
            {
                var saved = JsonSerializer.Deserialize<string>(File.ReadAllText(path));
                if (saved is "official" or "mirror") Source = saved;
            }
        }
        catch (IOException) { }
        catch (UnauthorizedAccessException) { }
        catch (JsonException) { }
    }

    internal BrowserComponentStatus Status
    {
        get
        {
            lock (_gate)
            {
                if (_status is { State: not ("ready" or "missing" or "restart_required") }) return _status;
                return new(Inspect().State, Error: _status?.Error);
            }
        }
    }
    internal void SelectSource(string source)
    {
        _ = BrowserComponentCatalog.SourceUrl(source);
        lock (_gate)
        {
            if (Busy || _closed) return;
            EnsureRoot();
            WriteAtomic(Path.Combine(_root, "source.json"), JsonSerializer.Serialize(source));
            Source = source;
        }
    }
    private void EnsureRoot()
    {
        Directory.CreateDirectory(_root);
        RejectLink(_root);
    }
    private static void RejectLink(string path)
    {
        if ((File.GetAttributes(path) & FileAttributes.ReparsePoint) != 0) throw new BrowserComponentException("corrupt");
    }
    private string Inside(string root, string relative)
    {
        string result;
        try { result = UpdateArchive.Inside(root, relative); }
        catch (UpdateException) { throw new BrowserComponentException("corrupt"); }
        for (var current = result; current != root; current = Path.GetDirectoryName(current)!)
            if (File.Exists(current) || Directory.Exists(current)) RejectLink(current);
        return result;
    }
    internal BrowserComponentInspection Inspect()
    {
        var package = _package;
        if (package is null) return new("unsupported");
        var pointer = Path.Combine(_root, "active.json");
        try
        {
            if (!File.Exists(pointer)) return new("missing");
            RejectLink(_root);
            RejectLink(pointer);
            if (new FileInfo(pointer).Length > 256) return new("corrupt");
            var id = JsonSerializer.Deserialize<string>(File.ReadAllText(pointer));
            if (!Guid.TryParseExact(id, "N", out _)) return new("corrupt");
            var directory = Inside(_root, "installed/" + id);
            if (Volatile.Read(ref _invalidDirectory) == directory) return new("corrupt");
            var manifest = ReadManifest(directory, package);
            foreach (var file in manifest.Files)
            {
                var path = Inside(directory, file.Path);
                if (!File.Exists(path) || new FileInfo(path).Length != file.Size) return new("corrupt");
            }
            if (_loadedDirectory is not null && _loadedDirectory != directory) return new("restart_required", directory);
            return new("ready", directory);
        }
        catch (BrowserComponentException exception) { return new(exception.Code); }
        catch (Exception exception) when (exception is IOException or UnauthorizedAccessException or JsonException or ArgumentException) { return new("corrupt"); }
    }
    private BrowserInstallation ReadManifest(string directory, BrowserComponentPackage package)
    {
        var path = Inside(directory, "installation.json");
        if (!File.Exists(path) || new FileInfo(path).Length > 1024 * 1024) throw new BrowserComponentException("corrupt");
        var manifest = JsonSerializer.Deserialize<BrowserInstallation>(File.ReadAllText(path)) ?? throw new BrowserComponentException("corrupt");
        if (manifest.Target != package.Target || manifest.Version != package.Version || manifest.PackageSha256 != package.Sha256)
            throw new BrowserComponentException("incompatible");
        if (manifest.Files is null || manifest.Files.Length is 0 or > 10000) throw new BrowserComponentException("corrupt");
        var paths = new HashSet<string>(StringComparer.OrdinalIgnoreCase);
        foreach (var file in manifest.Files)
            if (file is null || string.IsNullOrEmpty(file.Path) || file.Size < 0 || file.Sha256?.Length != 64
                || !file.Sha256.All(Uri.IsHexDigit) || !paths.Add(file.Path)) throw new BrowserComponentException("corrupt");
        foreach (var required in new[] { package.Library, "icudtl.dat", "resources.pak", "locales/en-US.pak" })
            if (!paths.Contains(required)) throw new BrowserComponentException("corrupt");
        return manifest;
    }
    internal string RequireReady()
    {
        if (Busy) throw new BrowserComponentException("installing");
        var inspection = Inspect();
        if (inspection.State != "ready" || inspection.Directory is null) throw new BrowserComponentException(inspection.State);
        return inspection.Directory;
    }
    internal void MarkLoaded(string directory) { lock (_gate) _loadedDirectory = directory; }

    internal Task InstallAsync(DownloadNetworkConfiguration network)
    {
        lock (_gate)
        {
            if (_closed || Busy) return _operation ?? Task.CompletedTask;
            var package = _package;
            if (package is null) { _status = new("unsupported"); return Task.CompletedTask; }
            if (CefPreviewRuntime.ActiveViews != 0) { _status = new(Inspect().State, Error: "browser_active"); return Task.CompletedTask; }
            _cancellation?.Dispose();
            _cancellation = new();
            var source = Source;
            var token = _cancellation.Token;
            _status = new("downloading");
            _operation = Task.Run(() => InstallCoreAsync(network, source, package, token));
            return _operation;
        }
    }
    private async Task InstallCoreAsync(DownloadNetworkConfiguration network, string source, BrowserComponentPackage package, CancellationToken token)
    {
        string? staging = null;
        string? destination = null;
        bool committed = false;
        try
        {
            EnsureRoot();
            if (File.Exists(Path.Combine(_root, "install.lock"))) RejectLink(Path.Combine(_root, "install.lock"));
            using var installLock = new FileStream(Path.Combine(_root, "install.lock"), FileMode.OpenOrCreate, FileAccess.ReadWrite, FileShare.None);
            foreach (var abandoned in Directory.EnumerateDirectories(_root, ".staging-*"))
                if (Guid.TryParseExact(Path.GetFileName(abandoned)[9..], "N", out _) && (File.GetAttributes(abandoned) & FileAttributes.ReparsePoint) == 0) TryDelete(abandoned);
            staging = Inside(_root, ".staging-" + Guid.NewGuid().ToString("N"));
            Directory.CreateDirectory(staging);
            using var timeout = CancellationTokenSource.CreateLinkedTokenSource(token);
            timeout.CancelAfter(TimeSpan.FromMinutes(30));
            // Installation never inherits an insecure-certificate preference.
            using var handler = _handler(network with { VerifyCertificates = true });
            using var client = new HttpClient(handler) { Timeout = TimeSpan.FromSeconds(30) };
            var uri = await BrowserComponentInstaller.ResolvePackageAsync(client, source, package, timeout.Token).ConfigureAwait(false);
            var archive = Path.Combine(staging, "runtime.nupkg");
            await BrowserComponentInstaller.DownloadAsync(client, uri, archive, package,
                (bytes, total) => Set(new("downloading", bytes, total)), timeout.Token).ConfigureAwait(false);
            Set(new("verifying"));
            timeout.Token.ThrowIfCancellationRequested();
            Set(new("installing"));
            var payload = Path.Combine(staging, "payload");
            await BrowserComponentInstaller.PrepareAsync(archive, Path.Combine(staging, "raw"), payload, package, timeout.Token).ConfigureAwait(false);
            var id = Guid.NewGuid().ToString("N");
            destination = Inside(_root, "installed/" + id);
            Directory.CreateDirectory(Path.GetDirectoryName(destination)!);
            timeout.Token.ThrowIfCancellationRequested();
            Directory.Move(payload, destination);
            WriteAtomic(Path.Combine(_root, "active.json"), JsonSerializer.Serialize(id));
            committed = true;
            Set(new(Inspect().State));
            CleanupInactiveInstallations(destination);
        }
        catch (OperationCanceledException) { Set(token.IsCancellationRequested ? new(Inspect().State) : new("download_failed", Error: "download_timeout")); }
        catch (Exception exception)
        {
            var code = exception is BrowserComponentException component ? component.Code : "download_failed";
            var state = code is "hash_mismatch" or "source_missing" or "corrupt" or "incompatible" ? code : "download_failed";
            Set(new(state, Error: code));
            DiagnosticLog.Error("browser.install", exception, "component_install_failed=true");
        }
        finally
        {
            if (staging is not null) TryDelete(staging);
            if (!committed && destination is not null) TryDelete(destination);
        }
    }
    // Called by the primary Desktop process after it acquires the data-directory
    // instance lock. CEF subprocesses must never collect their parent's runtime.
    internal void CleanupUnusedInstallations()
    {
        try
        {
            lock (_gate)
            {
                if (_closed || Busy || !Directory.Exists(_root)) return;
                RejectLink(_root);
                var lockPath = Path.Combine(_root, "install.lock");
                if (File.Exists(lockPath)) RejectLink(lockPath);
                using var installLock = new FileStream(lockPath, FileMode.OpenOrCreate, FileAccess.ReadWrite, FileShare.None);
                var active = Inspect();
                if (active.State is "ready" or "restart_required" && active.Directory is not null)
                    CleanupInactiveInstallations(active.Directory);
            }
        }
        catch (Exception exception) when (exception is IOException or UnauthorizedAccessException or BrowserComponentException)
        {
            DiagnosticLog.Error("browser.cleanup", exception, "unused_installation_cleanup_deferred=true");
        }
    }

    private void CleanupInactiveInstallations(string activeDirectory)
    {
        try
        {
            var installedRoot = Inside(_root, "installed");
            foreach (var candidate in Directory.EnumerateDirectories(installedRoot))
            {
                var id = Path.GetFileName(candidate);
                if (!Guid.TryParseExact(id, "N", out _)) continue;
                var directory = Inside(_root, "installed/" + id);
                if (directory == activeDirectory || directory == Volatile.Read(ref _loadedDirectory)) continue;
                TryDelete(directory);
            }
        }
        catch (Exception exception) when (exception is IOException or UnauthorizedAccessException or BrowserComponentException)
        {
            DiagnosticLog.Error("browser.cleanup", exception, "unused_installation_cleanup_deferred=true");
        }
    }

    internal Task VerifyAsync()
    {
        lock (_gate)
        {
            if (_closed || Busy || CefPreviewRuntime.ActiveViews != 0) return Task.CompletedTask;
            var inspection = Inspect();
            if (inspection.Directory is null) { _status = new(inspection.State); return Task.CompletedTask; }
            _cancellation?.Dispose();
            _cancellation = new();
            var token = _cancellation.Token;
            _status = new("verifying_files");
            _operation = Task.Run(async () =>
            {
                try { await VerifyFilesAsync(inspection.Directory, token).ConfigureAwait(false); Set(new(Inspect().State)); }
                catch (OperationCanceledException) { Set(new(Inspect().State)); }
                catch (Exception exception) { Set(new("corrupt", Error: "corrupt")); DiagnosticLog.Error("browser.verify", exception, "file_verification_failed=true"); }
            });
            return _operation;
        }
    }
    internal async Task VerifyFilesAsync(string directory, CancellationToken token)
    {
        try
        {
            var package = _package ?? throw new BrowserComponentException("unsupported");
            var manifest = ReadManifest(directory, package);
            foreach (var file in manifest.Files)
            {
                await using var input = File.OpenRead(Inside(directory, file.Path));
                BrowserComponentInstaller.VerifyDigest(await SHA256.HashDataAsync(input, token).ConfigureAwait(false), file.Sha256);
            }
        }
        catch (OperationCanceledException) { throw; }
        catch { Volatile.Write(ref _invalidDirectory, directory); throw; }
    }
    private static void WriteAtomic(string path, string json)
    {
        var temp = path + "." + Guid.NewGuid().ToString("N") + ".tmp";
        try { File.WriteAllText(temp, json); File.Move(temp, path, true); }
        finally { if (File.Exists(temp)) File.Delete(temp); }
    }
    private void Set(BrowserComponentStatus status) { lock (_gate) _status = status; }
    internal void Cancel() { lock (_gate) _cancellation?.Cancel(); }
    internal async Task ShutdownAsync()
    {
        Task? operation;
        lock (_gate) { _closed = true; _cancellation?.Cancel(); operation = _operation; }
        if (operation is not null) await operation.ConfigureAwait(false);
        _cancellation?.Dispose();
    }
    private static void TryDelete(string path)
    {
        try { if (Directory.Exists(path)) Directory.Delete(path, true); }
        catch (IOException) { }
        catch (UnauthorizedAccessException) { }
    }
}

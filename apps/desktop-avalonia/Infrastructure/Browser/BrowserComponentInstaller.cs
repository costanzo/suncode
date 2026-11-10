using System.Net;
using System.Security.Cryptography;
using System.Text.Json;

namespace SunCode.Desktop.Infrastructure;

internal sealed record BrowserInstalledFile(string Path, long Size, string Sha256);
internal sealed record BrowserInstallation(string Target, string Version, string PackageSha256, BrowserInstalledFile[] Files);

internal static class BrowserComponentInstaller
{
    private const long MaximumDownload = 512L * 1024 * 1024;
    internal static async Task<Uri> ResolvePackageAsync(HttpClient client, string source, BrowserComponentPackage package, CancellationToken token)
    {
        using var response = await client.GetAsync(BrowserComponentCatalog.SourceUrl(source), HttpCompletionOption.ResponseHeadersRead, token).ConfigureAwait(false);
        response.EnsureSuccessStatusCode();
        await using var body = await response.Content.ReadAsStreamAsync(token).ConfigureAwait(false);
        using var limited = new MemoryStream();
        var buffer = new byte[8192];
        while (true)
        {
            var count = await body.ReadAsync(buffer, token).ConfigureAwait(false);
            if (count == 0) break;
            if (limited.Length + count > 1024 * 1024) throw new BrowserComponentException("source_invalid");
            limited.Write(buffer, 0, count);
        }
        using var index = JsonDocument.Parse(limited.ToArray());
        foreach (var resource in index.RootElement.GetProperty("resources").EnumerateArray())
        {
            if (!resource.GetProperty("@type").GetString()!.StartsWith("PackageBaseAddress/3.", StringComparison.Ordinal)) continue;
            var baseAddress = resource.GetProperty("@id").GetString()!;
            if (!Uri.TryCreate(baseAddress, UriKind.Absolute, out var uri) || uri.Scheme != "https" || uri.UserInfo.Length != 0)
                throw new BrowserComponentException("source_invalid");
            return new Uri(uri.AbsoluteUri.TrimEnd('/') + $"/{package.Id}/{package.Version}/{package.Id}.{package.Version}.nupkg");
        }
        throw new BrowserComponentException("source_invalid");
    }

    internal static async Task DownloadAsync(HttpClient client, Uri uri, string path, BrowserComponentPackage package, Action<long, long?> progress, CancellationToken token)
    {
        using var response = await client.GetAsync(uri, HttpCompletionOption.ResponseHeadersRead, token).ConfigureAwait(false);
        if (response.StatusCode == HttpStatusCode.NotFound) throw new BrowserComponentException("source_missing");
        response.EnsureSuccessStatusCode();
        if (response.RequestMessage?.RequestUri?.Scheme != "https") throw new BrowserComponentException("source_invalid");
        var total = response.Content.Headers.ContentLength;
        if (total > MaximumDownload) throw new BrowserComponentException("download_too_large");
        await using var input = await response.Content.ReadAsStreamAsync(token).ConfigureAwait(false);
        await using var output = new FileStream(path, FileMode.CreateNew, FileAccess.Write, FileShare.None, 81920, true);
        using var digest = IncrementalHash.CreateHash(HashAlgorithmName.SHA256);
        var buffer = new byte[81920];
        long received = 0;
        while (true)
        {
            using var idle = CancellationTokenSource.CreateLinkedTokenSource(token);
            idle.CancelAfter(TimeSpan.FromSeconds(30));
            var count = await input.ReadAsync(buffer, idle.Token).ConfigureAwait(false);
            if (count == 0) break;
            received += count;
            if (received > MaximumDownload) throw new BrowserComponentException("download_too_large");
            digest.AppendData(buffer, 0, count);
            await output.WriteAsync(buffer.AsMemory(0, count), token).ConfigureAwait(false);
            progress(received, total);
        }
        if (total is not null && received != total) throw new BrowserComponentException("download_failed");
        VerifyDigest(digest.GetHashAndReset(), package.Sha256);
    }

    internal static void VerifyDigest(byte[] digest, string expected)
    {
        if (!CryptographicOperations.FixedTimeEquals(digest, Convert.FromHexString(expected)))
            throw new BrowserComponentException("hash_mismatch");
    }

    internal static async Task PrepareAsync(string archive, string raw, string destination, BrowserComponentPackage package, CancellationToken token)
    {
        // Reuse the bounded ZIP parser's path, duplicate, symlink and size checks.
        await PowerShellInstaller.ExtractAsync(archive, raw, token).ConfigureAwait(false);
        Directory.CreateDirectory(destination);
        if (package.Target.StartsWith("win-", StringComparison.Ordinal))
        {
            CopyTree(Path.Combine(raw, "runtimes", package.Target, "native"), destination);
            CopyTree(Path.Combine(raw, "CEF", package.Target), destination);
        }
        else
        {
            CopyTree(Path.Combine(raw, "CEF", package.Target, "Release"), destination);
            CopyTree(Path.Combine(raw, "CEF", package.Target, "Resources"), destination);
        }
        File.Copy(Path.Combine(raw, "LICENSE.txt"), Path.Combine(destination, "LICENSE.txt"), false);
        foreach (var required in new[] { package.Library, "icudtl.dat", "resources.pak", "locales/en-US.pak" })
            if (!File.Exists(Path.Combine(destination, required))) throw new BrowserComponentException("package_invalid");
        var files = new List<BrowserInstalledFile>();
        foreach (var file in Directory.EnumerateFiles(destination, "*", SearchOption.AllDirectories))
        {
            token.ThrowIfCancellationRequested();
            await using var input = File.OpenRead(file);
            var hash = await SHA256.HashDataAsync(input, token).ConfigureAwait(false);
            files.Add(new(Path.GetRelativePath(destination, file).Replace('\\', '/'), input.Length, Convert.ToHexStringLower(hash)));
        }
        var manifest = new BrowserInstallation(package.Target, package.Version, package.Sha256, files.ToArray());
        await File.WriteAllTextAsync(Path.Combine(destination, "installation.json"), JsonSerializer.Serialize(manifest), token).ConfigureAwait(false);
    }

    private static void CopyTree(string source, string target)
    {
        if (!Directory.Exists(source)) throw new BrowserComponentException("package_invalid");
        foreach (var file in Directory.EnumerateFiles(source, "*", SearchOption.AllDirectories))
        {
            var output = Path.Combine(target, Path.GetRelativePath(source, file));
            Directory.CreateDirectory(Path.GetDirectoryName(output)!);
            File.Copy(file, output, false);
        }
    }
}

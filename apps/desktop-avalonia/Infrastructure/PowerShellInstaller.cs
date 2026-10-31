using System.IO.Compression;
using System.Net;
using System.Net.Security;
using System.Security.Cryptography;
using System.Security.Cryptography.X509Certificates;
using System.Text.Json;
using System.Text.RegularExpressions;
using SunCode.Sdk;
using SunCode.Sdk.Models;

namespace SunCode.Desktop.Infrastructure;

internal static class PowerShellInstaller
{
    private const long MaxArchive = 200L * 1024 * 1024;
    private const long MaxExpanded = 1024L * 1024 * 1024;
    internal static async Task InstallAsync(DownloadNetworkConfiguration network, Action<string, int> progress, CancellationToken token)
    {
        var directory = ShellResolver.ManagedDirectory;
        var parent = Path.GetDirectoryName(directory)!;
        Directory.CreateDirectory(parent);
        // FileShare.None also serializes independent desktop processes.
        using var installLock = new FileStream(Path.Combine(parent, ".install.lock"), FileMode.OpenOrCreate, FileAccess.ReadWrite, FileShare.None);
        foreach (var abandoned in Directory.EnumerateDirectories(parent, "suncode-powershell-*"))
            if ((File.GetAttributes(abandoned) & FileAttributes.ReparsePoint) == 0 && !Directory.Exists(Path.Combine(abandoned, "previous"))) TryDelete(abandoned);
        var staging = Path.Combine(parent, "suncode-powershell-" + Guid.NewGuid().ToString("N"));
        Directory.CreateDirectory(staging);
        var committed = false;
        try
        {
            var archive = Path.Combine(staging, "powershell.zip");
            var runtime = Path.Combine(staging, "runtime");
            using var handler = CreateHandler(network);
            using var client = new HttpClient(handler) { Timeout = TimeSpan.FromMinutes(5) };
            var arch = ShellResolver.Architecture;
            var url = $"https://github.com/PowerShell/PowerShell/releases/download/v{ShellResolver.PowerShellVersion}/PowerShell-{ShellResolver.PowerShellVersion}-win-{arch}.zip";
            progress("downloading", 0);
            using var response = await client.GetAsync(url, HttpCompletionOption.ResponseHeadersRead, token).ConfigureAwait(false);
            response.EnsureSuccessStatusCode();
            var size = response.Content.Headers.ContentLength;
            if (size > MaxArchive) throw new SdkException("shell_download_too_large", "PowerShell archive exceeds the download limit");
            using var hash = IncrementalHash.CreateHash(HashAlgorithmName.SHA256);
            // HeadersRead needs an explicit timeout covering the response stream too.
            using var downloadTimeout = CancellationTokenSource.CreateLinkedTokenSource(token);
            downloadTimeout.CancelAfter(TimeSpan.FromMinutes(5));
            await using (var source = await response.Content.ReadAsStreamAsync(downloadTimeout.Token).ConfigureAwait(false))
            await using (var target = new FileStream(archive, FileMode.CreateNew, FileAccess.Write, FileShare.None, 81920, true))
            {
                var buffer = new byte[81920]; long received = 0;
                while (true)
                {
                    var count = await source.ReadAsync(buffer, downloadTimeout.Token).ConfigureAwait(false);
                    if (count == 0) break;
                    received += count;
                    if (received > MaxArchive) throw new SdkException("shell_download_too_large", "PowerShell archive exceeds the download limit");
                    hash.AppendData(buffer, 0, count);
                    await target.WriteAsync(buffer.AsMemory(0, count), downloadTimeout.Token).ConfigureAwait(false);
                    progress("downloading", size > 0 ? (int)Math.Min(65, received * 65 / size.Value) : 0);
                }
            }
            token.ThrowIfCancellationRequested(); progress("verifying", 70);
            VerifyDigest(hash.GetHashAndReset(), ShellResolver.ReleaseDigest(arch));
            progress("extracting", 80);
            await ExtractAsync(archive, runtime, token).ConfigureAwait(false);
            progress("checking", 95);
            var shell = await ShellResolver.ResolveAsync(new("custom", Path.Combine(runtime, "pwsh.exe")), false, token).ConfigureAwait(false);
            if (shell.Version != $"PowerShell {ShellResolver.PowerShellVersion}") throw new SdkException("shell_install_invalid", "The archive did not contain the expected PowerShell release");
            await File.WriteAllTextAsync(Path.Combine(runtime, "suncode-runtime.json"), JsonSerializer.Serialize(new { version = ShellResolver.PowerShellVersion, architecture = arch, sha256 = ShellResolver.ReleaseDigest(arch) }), token).ConfigureAwait(false);
            token.ThrowIfCancellationRequested();
            var backup = Path.Combine(staging, "previous");
            if (Directory.Exists(directory)) Directory.Move(directory, backup);
            try { Directory.Move(runtime, directory); committed = true; }
            catch { if (Directory.Exists(backup)) Directory.Move(backup, directory); throw; }
        }
        finally { if (committed || !Directory.Exists(Path.Combine(staging, "previous"))) TryDelete(staging); }
    }
    internal static void VerifyDigest(byte[] actual, string expected)
    {
        if (!CryptographicOperations.FixedTimeEquals(actual, Convert.FromHexString(expected))) throw new SdkException("shell_download_integrity_failed", "The downloaded archive did not match the official release. Retry the download");
    }
    internal static async Task ExtractAsync(string archive, string target, CancellationToken token)
    {
        using var zip = ZipFile.OpenRead(archive);
        if (zip.Entries.Count > 20_000) throw new SdkException("shell_install_invalid", "The archive contains too many entries");
        Directory.CreateDirectory(target);
        long total = 0;
        var paths = new HashSet<string>(StringComparer.OrdinalIgnoreCase);
        foreach (var entry in zip.Entries)
        {
            token.ThrowIfCancellationRequested();
            var name = entry.FullName;
            var components = name.TrimEnd('/').Split('/');
            var mode = (entry.ExternalAttributes >> 16) & 0xF000;
            if (name.Length == 0 || name.Contains('\\') || name.StartsWith('/') || mode == 0xA000 || (entry.ExternalAttributes & (int)FileAttributes.ReparsePoint) != 0
                || components.Any(InvalidComponent) || !paths.Add(name.TrimEnd('/')))
                throw new SdkException("shell_install_invalid", "The archive contains an unsafe or duplicate path");
            total = checked(total + entry.Length);
            if (total > MaxExpanded) throw new SdkException("shell_install_invalid", "The expanded archive exceeds the installation limit");
            var path = Path.Combine(target, Path.Combine(components));
            if (name.EndsWith('/')) { Directory.CreateDirectory(path); continue; }
            Directory.CreateDirectory(Path.GetDirectoryName(path)!);
            await using var input = entry.Open();
            await using var output = new FileStream(path, FileMode.CreateNew, FileAccess.Write, FileShare.None, 81920, true);
            var buffer = new byte[81920]; long written = 0;
            while (true)
            {
                var count = await input.ReadAsync(buffer, token).ConfigureAwait(false);
                if (count == 0) break;
                written += count;
                if (written > entry.Length) throw new SdkException("shell_install_invalid", "An archive entry exceeded its declared size");
                await output.WriteAsync(buffer.AsMemory(0, count), token).ConfigureAwait(false);
            }
            if (written != entry.Length) throw new SdkException("shell_install_invalid", "An archive entry was incomplete");
        }
    }
    private static bool InvalidComponent(string component)
    {
        if (component.Length == 0 || component is "." or ".." || component.EndsWith('.') || component.EndsWith(' ') || component.Any(c => char.IsControl(c) || "<>:\"|?*".Contains(c))) return true;
        var basename = component.Split('.')[0].ToUpperInvariant();
        return basename is "CON" or "PRN" or "AUX" or "NUL" or "CLOCK$" or "CONIN$" or "CONOUT$" || Regex.IsMatch(basename, "^(COM|LPT)[1-9¹²³]$");
    }
    internal static SocketsHttpHandler CreateHandler(DownloadNetworkConfiguration network)
    {
        var handler = new SocketsHttpHandler { ConnectTimeout = TimeSpan.FromSeconds(20) };
        handler.UseProxy = network.Proxy.Mode != "no_proxy";
        if (network.Proxy.Mode == "custom") handler.Proxy = new DownloadProxy(network.Proxy);
        if (!network.VerifyCertificates) handler.SslOptions.RemoteCertificateValidationCallback = (_, _, _, _) => true;
        else if (!network.UseSystemCertificates || !string.IsNullOrWhiteSpace(network.CertificatePath))
        {
            var roots = new X509Certificate2Collection();
            if (!string.IsNullOrWhiteSpace(network.CertificatePath)) roots.ImportFromPemFile(network.CertificatePath);
            handler.SslOptions.RemoteCertificateValidationCallback = (_, certificate, existing, errors) =>
            {
                if (certificate is null || (errors & (SslPolicyErrors.RemoteCertificateNameMismatch | SslPolicyErrors.RemoteCertificateNotAvailable)) != 0) return false;
                if (network.UseSystemCertificates && errors == SslPolicyErrors.None) return true;
                using var chain = new X509Chain();
                chain.ChainPolicy.TrustMode = X509ChainTrustMode.CustomRootTrust;
                chain.ChainPolicy.CustomTrustStore.AddRange(roots);
                chain.ChainPolicy.RevocationMode = X509RevocationMode.NoCheck;
                if (existing is not null) foreach (var element in existing.ChainElements) chain.ChainPolicy.ExtraStore.Add(element.Certificate);
                using var leaf = new X509Certificate2(certificate);
                return chain.Build(leaf);
            };
        }
        return handler;
    }
    private sealed class DownloadProxy(DownloadProxyConfiguration configuration) : IWebProxy
    {
        private readonly Uri _proxy = new(configuration.Url);
        public ICredentials? Credentials { get; set; } = string.IsNullOrEmpty(configuration.Username) ? null : new NetworkCredential(configuration.Username, configuration.Password);
        public Uri GetProxy(Uri destination) => IsBypassed(destination) ? destination : _proxy;
        public bool IsBypassed(Uri destination) => configuration.Bypass.Any(rule =>
            rule == "*" || rule.StartsWith('.') && (destination.Host.Equals(rule[1..], StringComparison.OrdinalIgnoreCase) || destination.Host.EndsWith(rule, StringComparison.OrdinalIgnoreCase))
            || destination.Host.Equals(rule, StringComparison.OrdinalIgnoreCase)
            || IPNetwork.TryParse(rule, out var network) && IPAddress.TryParse(destination.Host, out var address) && network.Contains(address));
    }
    private static void TryDelete(string directory) { try { Directory.Delete(directory, true); } catch (IOException) { } catch (UnauthorizedAccessException) { } }
}

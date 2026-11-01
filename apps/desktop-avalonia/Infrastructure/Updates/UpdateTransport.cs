using System.Net;
using System.Net.Http.Headers;
using System.Security.Cryptography;

namespace SunCode.Desktop.Infrastructure.Updates;

internal sealed class UpdateTransport : IDisposable
{
    private readonly HttpClient _client;
    private readonly UpdateDistribution _distribution;
    private readonly CatalogVerifier _verifier;
    private readonly UpdateStateStore _store;
    private readonly TimeProvider _time;
    public DateTimeOffset? RetryNotBefore { get; private set; }
    public string CatalogUrl { get; }

    public UpdateTransport(UpdateDistribution distribution, UpdateStateStore store, UpdateEnvironment environment, HttpMessageHandler? handler = null, TimeProvider? time = null)
    {
        _distribution = distribution; _store = store;
        _time = time ?? TimeProvider.System;
        _verifier = new(distribution, environment.Target, time);
        CatalogUrl = distribution.Enabled ? $"{distribution.CatalogOrigin?.TrimEnd('/')}/updates/v1/{distribution.Channel}/{environment.Target}/latest.json" : "";
        // No certificate-validation override, cookies, credentials, or automatic redirect.
        _client = new(handler ?? new SocketsHttpHandler { AllowAutoRedirect = false, UseCookies = false, AutomaticDecompression = DecompressionMethods.GZip | DecompressionMethods.Deflate })
        { Timeout = TimeSpan.FromMinutes(30) };
    }

    public async Task<VerifiedCatalog> CheckAsync(CancellationToken cancellationToken)
    {
        _distribution.Validate();
        if (RetryNotBefore > _time.GetUtcNow()) throw new UpdateException("retry_later", "The update service asked to retry later.");
        var cached = _store.Cached(CatalogUrl);
        for (var attempt = 0; attempt < 2; attempt++)
        {
            using var timeout = CancellationTokenSource.CreateLinkedTokenSource(cancellationToken);
            timeout.CancelAfter(TimeSpan.FromSeconds(30));
            using var request = new HttpRequestMessage(HttpMethod.Get, CatalogUrl);
            request.Headers.Accept.Add(new("application/json"));
            request.Headers.CacheControl = new() { NoCache = true };
            if (attempt == 0 && cached?.ETag is { } etag && EntityTagHeaderValue.TryParse(etag, out var parsed)) request.Headers.IfNoneMatch.Add(parsed);
            using var response = await _client.SendAsync(request, HttpCompletionOption.ResponseHeadersRead, timeout.Token);
            if (response.StatusCode == HttpStatusCode.NotModified)
            {
                if (attempt == 0 && cached is not null)
                {
                    try { return Accept(cached.Envelope, response.Headers.ETag?.ToString() ?? cached.ETag); }
                    catch (UpdateException) { }
                }
                if (attempt == 0) continue;
                throw new UpdateException("check_failed", "The update service returned no usable catalog.");
            }
            if (!response.IsSuccessStatusCode)
            {
                if (response.StatusCode is HttpStatusCode.TooManyRequests or HttpStatusCode.ServiceUnavailable
                    && response.Headers.RetryAfter?.Delta is { } delay)
                    RetryNotBefore = _time.GetUtcNow().AddSeconds(Math.Clamp(delay.TotalSeconds, 0, 86400));
                var error = response.StatusCode is HttpStatusCode.TooManyRequests or HttpStatusCode.ServiceUnavailable ? "retry_later" : "check_failed";
                throw new UpdateException(error, "The update service is unavailable. Try again later.");
            }
            if (response.StatusCode != HttpStatusCode.OK || response.Content.Headers.ContentType?.MediaType != "application/json")
                throw new UpdateException("check_failed", "The update service returned an invalid response.");
            var bytes = await ReadBoundedAsync(response.Content, CatalogVerifier.MaximumCatalogBytes, timeout.Token);
            return Accept(bytes, response.Headers.ETag?.ToString());
        }
        throw new UpdateException("check_failed", "The update service returned no usable catalog.");
    }

    private VerifiedCatalog Accept(byte[] bytes, string? etag)
    {
        var verified = _verifier.Authenticate(bytes, (revision, hash) => _store.ObserveAuthenticated(CatalogUrl, revision, hash, bytes, etag));
        _verifier.Validate(verified);
        return verified;
    }

    public VerifiedCatalog Revalidate(byte[] envelope)
    {
        var cached = _store.Cached(CatalogUrl);
        var etag = cached is not null && cached.Envelope.AsSpan().SequenceEqual(envelope) ? cached.ETag : null;
        var verified = _verifier.Authenticate(envelope, (revision, hash) => _store.ObserveAuthenticated(CatalogUrl, revision, hash, envelope, etag));
        _verifier.Validate(verified);
        return verified;
    }

    public async Task DownloadAsync(UpdatePackage package, string destination, Action<long>? progress, CancellationToken token)
    {
        _distribution.Validate();
        const long maximumArchiveBytes = 8L * 1024 * 1024 * 1024;
        if (package.SizeBytes > maximumArchiveBytes) throw new UpdateException("package_too_large", "The update package exceeds the supported size.");
        var uri = UpdateDistribution.ValidateHttps(package.Url);
        for (var hop = 0; hop < 6; hop++)
        {
            if (!_distribution.AllowsDownload(uri)) throw new UpdateException("download_origin", "The download location is not approved.");
            using var request = new HttpRequestMessage(HttpMethod.Get, uri);
            request.Headers.AcceptEncoding.Add(new("identity"));
            using var response = await _client.SendAsync(request, HttpCompletionOption.ResponseHeadersRead, token);
            if ((int)response.StatusCode is >= 300 and < 400)
            {
                if (response.Headers.Location is null) throw new UpdateException("download_failed", "The download redirect is invalid.");
                uri = UpdateDistribution.ValidateHttps(new Uri(uri, response.Headers.Location).AbsoluteUri);
                continue;
            }
            if (response.StatusCode != HttpStatusCode.OK || response.Content.Headers.ContentLength is { } length && length != package.SizeBytes)
                throw new UpdateException("download_failed", "The update download was interrupted.");
            Directory.CreateDirectory(Path.GetDirectoryName(destination)!);
            var partial = destination + ".partial";
            try
            {
                using var digest = IncrementalHash.CreateHash(HashAlgorithmName.SHA256);
                await using var output = new FileStream(partial, FileMode.Create, FileAccess.Write, FileShare.None, 81920, true);
                await using var input = await response.Content.ReadAsStreamAsync(token);
                var buffer = new byte[81920]; long count = 0; int read;
                while ((read = await input.ReadAsync(buffer, token)) != 0)
                {
                    count = checked(count + read);
                    if (count > package.SizeBytes) throw new UpdateException("integrity_failed", "The update package has an unexpected size.");
                    await output.WriteAsync(buffer.AsMemory(0, read), token);
                    digest.AppendData(buffer, 0, read); progress?.Invoke(count);
                }
                if (count != package.SizeBytes || Convert.ToHexStringLower(digest.GetHashAndReset()) != package.Sha256)
                    throw new UpdateException("integrity_failed", "The update package could not be verified.");
                await output.FlushAsync(token); output.Close();
                File.Move(partial, destination, overwrite: true);
                return;
            }
            finally { if (File.Exists(partial)) File.Delete(partial); }
        }
        throw new UpdateException("download_failed", "The update download redirected too many times.");
    }
    private static async Task<byte[]> ReadBoundedAsync(HttpContent content, int maximum, CancellationToken token)
    {
        if (content.Headers.ContentLength > maximum) throw new UpdateException("invalid_catalog", "The update catalog is too large.");
        await using var source = await content.ReadAsStreamAsync(token);
        using var memory = new MemoryStream();
        var buffer = new byte[16384]; int count;
        while ((count = await source.ReadAsync(buffer, token)) != 0)
        {
            if (memory.Length + count > maximum) throw new UpdateException("invalid_catalog", "The update catalog is too large.");
            memory.Write(buffer, 0, count);
        }
        return memory.ToArray();
    }
    public void Dispose() => _client.Dispose();
}

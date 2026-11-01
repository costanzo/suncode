using System.IO.Compression;
using System.Net;
using System.Text;
using System.Text.Json;
using System.Text.Json.Nodes;
using Org.BouncyCastle.Crypto.Parameters;
using Org.BouncyCastle.Crypto.Signers;
using Org.BouncyCastle.Security;
using SunCode.Desktop.Infrastructure.Updates;

namespace SunCode.Desktop.Tests.Updates;

internal sealed class UpdateFixture : IDisposable
{
    public string Root { get; } = Path.Combine(Path.GetTempPath(), "suncode-update-tests", Guid.NewGuid().ToString("N"));
    private readonly Ed25519PrivateKeyParameters _key = new(new SecureRandom());
    public UpdateDistribution Distribution { get; }
    public UpdateEnvironment Environment { get; } = new("win-x64", "10.0.22631", null);
    public FixedUpdateTime Time { get; } = new();
    public UpdateStateStore Store { get; }
    public UpdateInstallation Installed { get; }
    public UpdateCatalog Catalog { get; set; }
    public Dictionary<string, byte[]> Packages { get; } = [];
    public StubUpdateHandler Handler { get; } = new();

    public UpdateFixture()
    {
        Directory.CreateDirectory(Root);
        Store = new(Path.Combine(Root, "updates"));
        Distribution = new() { Enabled = true, CatalogOrigin = "https://updates.suncode.example", DownloadOrigins = ["https://downloads.suncode.example"], PublicKeys = [new() { Id = "test", PublicKey = Base64(_key.GeneratePublicKey().GetEncoded()) }] };
        var desktop = Files("installed-desktop", ("SunCode.dll", "old-desktop"));
        var sdk = Files("installed-sdk", (UpdateStager.SdkLibraryName, "old-sdk"));
        Installed = new(Environment.Target, new("0.1.0", 27, new string('a', 64), desktop, UpdateArchive.Manifest(desktop)), new("0.1.0", 27, new string('b', 64), sdk, UpdateArchive.Manifest(sdk)));
        var archive = Zip((UpdateStager.SdkLibraryName, "new-sdk"));
        var package = Package("sdk", archive);
        Catalog = new()
        {
            SchemaVersion = 1, Revision = 42, Channel = "stable", Target = Environment.Target,
            PublishedAt = "2026-11-01T02:00:00Z", ExpiresAt = "2026-11-08T02:00:00Z",
            Components = new()
            {
                Desktop = new() { Version = "0.1.0", RequiredSdkAbi = 27, PublishedAt = "2026-10-30T02:00:00Z", Package = new() { Url = "https://downloads.suncode.example/desktop.zip", Format = "zip", SizeBytes = 123, Sha256 = Installed.Desktop.ArchiveSha256 }, ReleaseNotes = new() { ["en-US"] = "Desktop unchanged." } },
                Sdk = new() { Version = "0.1.1", Abi = 27, PublishedAt = "2026-11-01T02:00:00Z", Package = package, ReleaseNotes = new() { ["en-US"] = "SDK improvements.", ["zh-CN"] = "改进 SDK。" } }
            },
            CompatiblePairs = [new() { Desktop = "0.1.0", Sdk = "0.1.0" }, new() { Desktop = "0.1.0", Sdk = "0.1.1" }]
        };
        Store.Change(s => s.Installation = Installed);
        Handler.Respond = (request, _) => Task.FromResult(request.RequestUri!.Host == "updates.suncode.example"
            ? JsonResponse(Sign()) : new HttpResponseMessage(HttpStatusCode.OK) { Content = new ByteArrayContent(Packages[request.RequestUri.AbsolutePath]) });
    }
    public UpdatePackage Package(string name, byte[] bytes)
    {
        Packages["/" + name + ".zip"] = bytes;
        return new() { Url = $"https://downloads.suncode.example/{name}.zip", Format = "zip", SizeBytes = bytes.Length, Sha256 = Convert.ToHexStringLower(System.Security.Cryptography.SHA256.HashData(bytes)) };
    }
    public string Files(string name, params (string Path, string Contents)[] files)
    {
        var root = Path.Combine(Root, name);
        Directory.CreateDirectory(root);
        foreach (var file in files) { var path = Path.Combine(root, file.Path); Directory.CreateDirectory(Path.GetDirectoryName(path)!); File.WriteAllText(path, file.Contents); }
        return root;
    }
    public CatalogVerifier Verifier => new(Distribution, Environment.Target, Time);
    public VerifiedCatalog Verified() { var value = Verifier.Authenticate(Sign()); Verifier.Validate(value); return value; }
    public byte[] Sign(Action<JsonObject>? change = null)
    {
        var signed = JsonSerializer.SerializeToNode(Catalog, UpdateJson.Options)!.AsObject();
        change?.Invoke(signed);
        using var doc = JsonDocument.Parse(signed.ToJsonString());
        var canonical = CanonicalJson.Encode(doc.RootElement);
        var message = Encoding.UTF8.GetBytes("SunCode Update Catalog v1\n").Concat(canonical).ToArray();
        var signer = new Ed25519Signer(); signer.Init(true, _key); signer.BlockUpdate(message, 0, message.Length);
        return JsonSerializer.SerializeToUtf8Bytes(new { signed, signatures = new[] { new { keyId = "test", algorithm = "Ed25519", value = Base64(signer.GenerateSignature()) } } }, UpdateJson.Options);
    }
    public static string Base64(byte[] bytes) => Convert.ToBase64String(bytes).TrimEnd('=').Replace('+', '-').Replace('/', '_');
    public static HttpResponseMessage JsonResponse(byte[] bytes)
    {
        var response = new HttpResponseMessage(HttpStatusCode.OK) { Content = new ByteArrayContent(bytes) };
        response.Content.Headers.ContentType = new("application/json"); response.Headers.ETag = new("\"revision42\"");
        return response;
    }
    public static byte[] Zip(params (string Path, string Contents)[] files)
    {
        using var buffer = new MemoryStream();
        using (var zip = new ZipArchive(buffer, ZipArchiveMode.Create, leaveOpen: true))
            foreach (var file in files) { var entry = zip.CreateEntry(file.Path); using var writer = new StreamWriter(entry.Open(), new UTF8Encoding(false)); writer.Write(file.Contents); }
        return buffer.ToArray();
    }
    public void Dispose() { Handler.Dispose(); if (Directory.Exists(Root)) Directory.Delete(Root, recursive: true); }
}
internal sealed class FixedUpdateTime : TimeProvider
{
    public DateTimeOffset Now { get; set; } = new(2026, 11, 1, 3, 0, 0, TimeSpan.Zero);
    public override DateTimeOffset GetUtcNow() => Now;
}
internal sealed class StubUpdateHandler : HttpMessageHandler
{
    public int Requests { get; private set; }
    public Func<HttpRequestMessage, CancellationToken, Task<HttpResponseMessage>> Respond { get; set; } = (_, _) => throw new InvalidOperationException();
    protected override Task<HttpResponseMessage> SendAsync(HttpRequestMessage request, CancellationToken cancellationToken) { Requests++; return Respond(request, cancellationToken); }
}
internal sealed class StubPlatformVerifier : IUpdatePlatformVerifier
{
    public int Checks { get; private set; }
    public Task VerifyAsync(string directory, bool desktop, CancellationToken token) { token.ThrowIfCancellationRequested(); Checks++; return Task.CompletedTask; }
}

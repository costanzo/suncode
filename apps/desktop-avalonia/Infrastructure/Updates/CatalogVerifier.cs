using System.Globalization;
using System.Security.Cryptography;
using System.Text;
using System.Text.Json;
using System.Text.RegularExpressions;
using Org.BouncyCastle.Crypto.Parameters;
using Org.BouncyCastle.Crypto.Signers;

namespace SunCode.Desktop.Infrastructure.Updates;

internal sealed class CatalogVerifier(UpdateDistribution distribution, string target, TimeProvider? time = null)
{
    public const int MaximumCatalogBytes = 1024 * 1024;
    private readonly TimeProvider _time = time ?? TimeProvider.System;
    public VerifiedCatalog Authenticate(byte[] bytes, Action<long, string>? authenticated = null)
    {
        distribution.Validate();
        if (bytes.Length == 0 || bytes.Length > MaximumCatalogBytes) throw Invalid();
        try
        {
            using var document = JsonDocument.Parse(bytes, new JsonDocumentOptions { MaxDepth = 48 });
            // Also checks duplicate properties and I-JSON strings in the unsigned envelope.
            _ = CanonicalJson.Encode(document.RootElement);
            var root = document.RootElement;
            var signed = root.GetProperty("signed");
            if (signed.ValueKind != JsonValueKind.Object) throw Invalid();
            var signatures = root.GetProperty("signatures");
            if (signatures.ValueKind != JsonValueKind.Array || signatures.GetArrayLength() is < 1 or > 8) throw Invalid();
            var canonical = CanonicalJson.Encode(signed);
            var message = Encoding.UTF8.GetBytes("SunCode Update Catalog v1\n").Concat(canonical).ToArray();
            var verified = false;
            foreach (var signature in signatures.EnumerateArray())
            {
                var id = signature.GetProperty("keyId").GetString()!;
                var algorithm = signature.GetProperty("algorithm").GetString();
                var value = DecodeBase64Url(signature.GetProperty("value").GetString()!);
                if (!Regex.IsMatch(id, "^[A-Za-z0-9._-]{1,64}$") || algorithm != "Ed25519" || value.Length != 64) throw Invalid();
                var key = distribution.PublicKeys.SingleOrDefault(k => k.Id == id && !k.Revoked);
                if (key is null) continue;
                var verifier = new Ed25519Signer();
                verifier.Init(false, new Ed25519PublicKeyParameters(DecodeBase64Url(key.PublicKey), 0));
                verifier.BlockUpdate(message, 0, message.Length);
                verified |= verifier.VerifySignature(value);
            }
            if (!verified) throw new UpdateException("signature_invalid", "The update catalog could not be verified.");
            if (signed.GetProperty("channel").GetString() != distribution.Channel || signed.GetProperty("target").GetString() != target) throw Invalid();
            var revision = signed.GetProperty("revision").GetInt64();
            if (revision is < 1 or > 9007199254740991) throw Invalid();
            var hash = Convert.ToHexStringLower(SHA256.HashData(canonical));
            authenticated?.Invoke(revision, hash);
            var now = _time.GetUtcNow();
            var published = Timestamp(signed.GetProperty("publishedAt").GetString()!);
            var expires = Timestamp(signed.GetProperty("expiresAt").GetString()!);
            if (published > now.AddMinutes(5)) throw new UpdateException("clock_invalid", "Check your system clock before checking for updates.");
            if (expires <= published || expires > published.AddDays(7)) throw Invalid();
            if (expires <= now) throw new UpdateException("catalog_expired", "The update catalog has expired. Check again.");
            var catalog = UpdateJson.Read<UpdateCatalog>(Encoding.UTF8.GetBytes(signed.GetRawText()));
            return new(catalog, bytes, hash);
        }
        catch (UpdateException) { throw; }
        catch (Exception e) when (e is JsonException or InvalidOperationException or KeyNotFoundException or ArgumentException or OverflowException)
        { throw new UpdateException("invalid_catalog", "The update catalog is invalid.", e); }
    }

    // Called after persisting the authenticated publication high-water mark.
    public void Validate(VerifiedCatalog verified)
    {
        var c = verified.Catalog;
        if (c.SchemaVersion != 1) throw new UpdateException("schema_unsupported", "This update catalog requires a newer installer.");
        var d = c.Components?.Desktop ?? throw Invalid();
        var s = c.Components.Sdk ?? throw Invalid();
        if (d.RequiredSdkAbi < 1 || s.Abi < 1 || d.RequiredSdkAbi != s.Abi) throw Invalid();
        if (c.CompatiblePairs is null || c.CompatiblePairs.Length is < 1 or > 256
            || c.CompatiblePairs.Distinct().Count() != c.CompatiblePairs.Length
            || !c.CompatiblePairs.Contains(new CompatiblePair { Desktop = d.Version, Sdk = s.Version })) throw Invalid();
        using var document = JsonDocument.Parse(verified.Envelope);
        if (document.RootElement.GetProperty("signed").GetProperty("compatiblePairs").EnumerateArray()
            .Any(pair => pair.EnumerateObject().Any(property => property.Name is not ("desktop" or "sdk")))) throw Invalid();
        foreach (var pair in c.CompatiblePairs) { ReleaseVersion.Parse(pair.Desktop); ReleaseVersion.Parse(pair.Sdk); }
        foreach (var release in new ComponentRelease[] { d, s })
        {
            ReleaseVersion.Parse(release.Version);
            if (Timestamp(release.PublishedAt) > Timestamp(c.PublishedAt)) throw Invalid();
            var p = release.Package ?? throw Invalid();
            var uri = UpdateDistribution.ValidateHttps(p.Url);
            if (!distribution.AllowsDownload(uri) || p.Format is not ("zip" or "tar.gz") || p.SizeBytes is < 1 or > 9007199254740991
                || !Regex.IsMatch(p.Sha256, "^[0-9a-f]{64}$")) throw Invalid();
            if (p.MinimumOsVersion is { } os && !Regex.IsMatch(os, "^[0-9]+(\\.[0-9]+){0,3}$")) throw Invalid();
            if (p.MinimumGlibcVersion is { } glibc && (!target.StartsWith("linux-", StringComparison.Ordinal)
                || !Regex.IsMatch(glibc, "^[0-9]+\\.[0-9]+$"))) throw Invalid();
            if (release.ReleaseNotes is null || release.ReleaseNotes.Count is < 1 or > 16 || !release.ReleaseNotes.ContainsKey("en-US")) throw Invalid();
            foreach (var note in release.ReleaseNotes)
                if (!Regex.IsMatch(note.Key, "^[A-Za-z]{2,8}(-[A-Za-z0-9]{1,8})*$") || note.Value is null || note.Value.Length is < 1 or > 16000) throw Invalid();
        }
    }

    internal static DateTimeOffset Timestamp(string text)
    {
        if (text is null || text.Length > 64 || !Regex.IsMatch(text, "^[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}(\\.[0-9]+)?Z$") || !DateTimeOffset.TryParse(text, CultureInfo.InvariantCulture,
                DateTimeStyles.AssumeUniversal | DateTimeStyles.AdjustToUniversal, out var result)) throw Invalid();
        return result;
    }
    internal static byte[] DecodeBase64Url(string text)
    {
        if (text is null || text.Length > 256 || !Regex.IsMatch(text, "^[A-Za-z0-9_-]+$")) throw Invalid();
        try
        {
            var value = Convert.FromBase64String(text.Replace('-', '+').Replace('_', '/').PadRight((text.Length + 3) / 4 * 4, '='));
            if (Convert.ToBase64String(value).TrimEnd('=').Replace('+', '-').Replace('/', '_') != text) throw Invalid();
            return value;
        }
        catch (FormatException) { throw Invalid(); }
    }
    private static UpdateException Invalid() => new("invalid_catalog", "The update catalog is invalid.");
}

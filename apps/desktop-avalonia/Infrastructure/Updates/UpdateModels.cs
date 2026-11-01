using System.Globalization;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace SunCode.Desktop.Infrastructure.Updates;

internal sealed class UpdateException(string code, string message, Exception? inner = null) : Exception(message, inner)
{
    public string Code { get; } = code;
}

internal readonly record struct ReleaseVersion(int Major, int Minor, int Patch) : IComparable<ReleaseVersion>
{
    public static ReleaseVersion Parse(string text)
    {
        var parts = text.Split('.');
        if (parts.Length != 3 || parts.Any(p => p.Length == 0 || (p.Length > 1 && p[0] == '0')
            || p.Any(c => c is < '0' or > '9') || !int.TryParse(p, NumberStyles.None, CultureInfo.InvariantCulture, out _)))
            throw new UpdateException("invalid_catalog", "A release version is invalid.");
        return new(int.Parse(parts[0], CultureInfo.InvariantCulture), int.Parse(parts[1], CultureInfo.InvariantCulture), int.Parse(parts[2], CultureInfo.InvariantCulture));
    }
    public int CompareTo(ReleaseVersion other)
    {
        var major = Major.CompareTo(other.Major);
        if (major != 0) return major;
        var minor = Minor.CompareTo(other.Minor);
        return minor != 0 ? minor : Patch.CompareTo(other.Patch);
    }
    public override string ToString() => $"{Major}.{Minor}.{Patch}";
}

internal static class UpdateJson
{
    public static readonly JsonSerializerOptions Options = new(JsonSerializerDefaults.Web)
    {
        PropertyNameCaseInsensitive = false,
        MaxDepth = 48,
        WriteIndented = true
    };
    public static T Read<T>(ReadOnlySpan<byte> bytes) => JsonSerializer.Deserialize<T>(bytes, Options)
        ?? throw new UpdateException("invalid_catalog", "Update metadata is empty.");
    public static byte[] Write<T>(T value) => JsonSerializer.SerializeToUtf8Bytes(value, Options);
}

internal sealed record UpdateCatalog
{
    public required int SchemaVersion { get; init; }
    public required long Revision { get; init; }
    public required string Channel { get; init; }
    public required string Target { get; init; }
    public required string PublishedAt { get; init; }
    public required string ExpiresAt { get; init; }
    public required CatalogComponents Components { get; init; }
    public required CompatiblePair[] CompatiblePairs { get; init; }
}

internal sealed record CatalogComponents
{
    public required DesktopRelease Desktop { get; init; }
    public required SdkRelease Sdk { get; init; }
}

internal record ComponentRelease
{
    public required string Version { get; init; }
    public required string PublishedAt { get; init; }
    public required UpdatePackage Package { get; init; }
    public required Dictionary<string, string> ReleaseNotes { get; init; }
}
internal sealed record DesktopRelease : ComponentRelease
{
    public required int RequiredSdkAbi { get; init; }
}
internal sealed record SdkRelease : ComponentRelease
{
    public required int Abi { get; init; }
}
internal sealed record UpdatePackage
{
    public required string Url { get; init; }
    public required string Format { get; init; }
    public required long SizeBytes { get; init; }
    public required string Sha256 { get; init; }
    public string? MinimumOsVersion { get; init; }
    public string? MinimumGlibcVersion { get; init; }
}
internal sealed record CompatiblePair
{
    public required string Desktop { get; init; }
    public required string Sdk { get; init; }
}

internal sealed record TrustedUpdateKey
{
    public required string Id { get; init; }
    public required string PublicKey { get; init; }
    public bool Revoked { get; init; }
}

internal sealed record UpdateDistribution
{
    public int SchemaVersion { get; init; } = 1;
    public bool Enabled { get; init; }
    public string? CatalogOrigin { get; init; }
    public string[] DownloadOrigins { get; init; } = [];
    public TrustedUpdateKey[] PublicKeys { get; init; } = [];
    public string? MacTeamId { get; init; }
    public string? WindowsPublisherSha256 { get; init; }
    public string Channel { get; init; } = "stable";

    public void Validate()
    {
        if (SchemaVersion != 1) throw new UpdateException("configuration_invalid", "The update distribution configuration is unsupported.");
        if (!Enabled) throw new UpdateException("configuration_missing", "Updates are not configured for this build.");
        if (Channel != "stable" || PublicKeys.Length is < 1 or > 8 || DownloadOrigins.Length is < 1 or > 16)
            throw new UpdateException("configuration_invalid", "The update distribution configuration is incomplete.");
        ValidateOrigin(CatalogOrigin!);
        foreach (var origin in DownloadOrigins) ValidateOrigin(origin);
        if (PublicKeys.Select(k => k.Id).Distinct(StringComparer.Ordinal).Count() != PublicKeys.Length)
            throw new UpdateException("configuration_invalid", "Update signing key identifiers are duplicated.");
        foreach (var key in PublicKeys)
        {
            if (!System.Text.RegularExpressions.Regex.IsMatch(key.Id, "^[A-Za-z0-9._-]{1,64}$") || CatalogVerifier.DecodeBase64Url(key.PublicKey).Length != 32)
                throw new UpdateException("configuration_invalid", "An update verification key is invalid.");
        }
        if (PublicKeys.All(k => k.Revoked)) throw new UpdateException("configuration_invalid", "No active update verification key is configured.");
    }
    internal static Uri ValidateOrigin(string text)
    {
        var uri = ValidateHttps(text);
        if (uri.AbsolutePath != "/" || uri.Query.Length != 0) throw new UpdateException("configuration_invalid", "An update origin is invalid.");
        return uri;
    }
    internal static Uri ValidateHttps(string text)
    {
        if (text is null || text.Length > 2048 || !Uri.TryCreate(text, UriKind.Absolute, out var uri)
            || uri.Scheme != "https" || uri.UserInfo.Length != 0 || uri.Fragment.Length != 0)
            throw new UpdateException("invalid_catalog", "An update URL is invalid.");
        return uri;
    }
    public bool AllowsDownload(Uri uri) => DownloadOrigins.Any(origin =>
        string.Equals(new Uri(origin).GetLeftPart(UriPartial.Authority), uri.GetLeftPart(UriPartial.Authority), StringComparison.OrdinalIgnoreCase));
}

internal sealed record VerifiedCatalog(UpdateCatalog Catalog, byte[] Envelope, string SignedSha256);
internal sealed record RunningUpdateVersions(string DesktopVersion, uint DesktopAbi, string? SdkVersion = null, uint? SdkAbi = null);
internal sealed record InstalledComponent(string Version, int Abi, string ArchiveSha256, string Directory, Dictionary<string, string> Files);
internal sealed record UpdateInstallation(string Target, InstalledComponent Desktop, InstalledComponent? Sdk);
internal sealed record UpdatePlan(VerifiedCatalog Verified, UpdateInstallation Installed, bool Desktop, bool Sdk, bool Repair)
{
    public long SizeBytes => (Desktop ? Verified.Catalog.Components.Desktop.Package.SizeBytes : 0)
        + (Sdk ? Verified.Catalog.Components.Sdk.Package.SizeBytes : 0);
    public string DesktopVersion => Desktop ? Verified.Catalog.Components.Desktop.Version : Installed.Desktop.Version;
    public string SdkVersion => Sdk ? Verified.Catalog.Components.Sdk.Version : Installed.Sdk!.Version;
}

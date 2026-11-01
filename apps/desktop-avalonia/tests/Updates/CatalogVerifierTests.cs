using System.Text;
using System.Text.Json;
using System.Text.Json.Nodes;
using SunCode.Desktop.Infrastructure.Updates;

namespace SunCode.Desktop.Tests.Updates;

public sealed class CatalogVerifierTests
{
    [Fact]
    public void VerifiesCompleteSignedObjectAndRejectsTampering()
    {
        using var f = new UpdateFixture();
        var valid = f.Sign(json => json["futureInformationalField"] = "signed too");
        var verified = f.Verifier.Authenticate(valid); f.Verifier.Validate(verified);
        var changed = JsonNode.Parse(valid)!;
        changed["signed"]!["futureInformationalField"] = "tampered";
        Assert.Equal("signature_invalid", Assert.Throws<UpdateException>(() => f.Verifier.Authenticate(Encoding.UTF8.GetBytes(changed.ToJsonString()))).Code);
    }
    [Fact]
    public void RejectsUnknownRevokedAndInvalidKeys()
    {
        using var f = new UpdateFixture();
        var unknown = f.Distribution with { PublicKeys = [f.Distribution.PublicKeys[0] with { Id = "other" }] };
        Assert.Throws<UpdateException>(() => new CatalogVerifier(unknown, f.Environment.Target, f.Time).Authenticate(f.Sign()));
        var revoked = f.Distribution with { PublicKeys = [f.Distribution.PublicKeys[0] with { Revoked = true }] };
        Assert.Throws<UpdateException>(() => new CatalogVerifier(revoked, f.Environment.Target, f.Time).Authenticate(f.Sign()));
    }
    [Theory]
    [InlineData("-0", "0")]
    [InlineData("1e30", "1e+30")]
    [InlineData("0.000001", "0.000001")]
    [InlineData("0.0000001", "1e-7")]
    [InlineData("1e20", "100000000000000000000")]
    [InlineData("333333333.33333329", "333333333.3333333")]
    public void CanonicalNumbersUseEcmaScriptLayout(string input, string expected)
    {
        using var doc = JsonDocument.Parse(input);
        Assert.Equal(expected, Encoding.UTF8.GetString(CanonicalJson.Encode(doc.RootElement)));
    }
    [Fact]
    public void CanonicalStringsSortUtf16AndDoNotNormalizeUnicodeOrEscapeSeparators()
    {
        using var doc = JsonDocument.Parse("{\"z\":\"é/\\n\",\"a\":\"\u2028\",\"😀\":1,\"\uE000\":2}");
        Assert.Equal("{\"a\":\"\u2028\",\"z\":\"é/\\n\",\"😀\":1,\"\uE000\":2}", Encoding.UTF8.GetString(CanonicalJson.Encode(doc.RootElement)));
    }
    [Theory]
    [InlineData("{\"a\":1,\"a\":2}")]
    [InlineData("{\"a\":1,\"\\u0061\":2}")]
    [InlineData("{\"value\":\"\\ud800\"}")]
    public void RejectsDuplicateKeysAndUnpairedSurrogates(string json)
    {
        using var doc = JsonDocument.Parse(json);
        Assert.ThrowsAny<Exception>(() => CanonicalJson.Encode(doc.RootElement));
    }
    [Fact]
    public void EnforcesSignedFreshnessTargetAndAbi()
    {
        using var f = new UpdateFixture();
        Assert.Throws<UpdateException>(() => f.Verifier.Authenticate(f.Sign(j => j["target"] = "osx-arm64")));
        Assert.Throws<UpdateException>(() => f.Verifier.Authenticate(f.Sign(j => j["publishedAt"] = "2026-11-01T04:00:00Z")));
        Assert.Throws<UpdateException>(() => f.Verifier.Authenticate(f.Sign(j => j["expiresAt"] = "2026-11-10T02:00:00Z")));
        f.Time.Now = f.Time.Now.AddDays(8);
        Assert.Equal("catalog_expired", Assert.Throws<UpdateException>(() => f.Verifier.Authenticate(f.Sign())).Code);
        f.Time.Now = f.Time.Now.AddDays(-8);
        var mismatch = f.Verifier.Authenticate(f.Sign(j => j["components"]!["sdk"]!["abi"] = 28));
        Assert.Throws<UpdateException>(() => f.Verifier.Validate(mismatch));
    }
    [Theory]
    [InlineData("version")]
    [InlineData("digest")]
    [InlineData("pair")]
    [InlineData("notes")]
    [InlineData("url")]
    [InlineData("glibc")]
    public void RejectsStructurallyOrSemanticallyInvalidReleases(string mutation)
    {
        using var f = new UpdateFixture();
        var envelope = f.Sign(j =>
        {
            var sdk = j["components"]!["sdk"]!;
            switch (mutation)
            {
                case "version": sdk["version"] = "2147483648.0.0"; break;
                case "digest": sdk["package"]!["sha256"] = "bad"; break;
                case "pair": j["compatiblePairs"] = new JsonArray(); break;
                case "notes": sdk["releaseNotes"] = new JsonObject(); break;
                case "url": sdk["package"]!["url"] = "http://downloads.suncode.example/a.zip"; break;
                case "glibc": sdk["package"]!["minimumGlibcVersion"] = "2.35"; break;
            }
        });
        var verified = f.Verifier.Authenticate(envelope);
        Assert.Throws<UpdateException>(() => f.Verifier.Validate(verified));
    }
    [Fact]
    public void PersistsHighWaterMarkBeforeRejectingNewSemanticContent()
    {
        using var f = new UpdateFixture();
        var old = f.Verified();
        f.Store.Observe("catalog", old, "\"42\"");
        f.Catalog = f.Catalog with { Revision = 43, Components = f.Catalog.Components with { Sdk = f.Catalog.Components.Sdk with { Abi = 28 } } };
        var invalid = f.Verifier.Authenticate(f.Sign());
        f.Store.Observe("catalog", invalid, "\"43\"");
        Assert.Throws<UpdateException>(() => f.Verifier.Validate(invalid));
        Assert.Equal("catalog_rollback", Assert.Throws<UpdateException>(() => f.Store.Observe("catalog", old, null)).Code);
        var conflict = invalid with { SignedSha256 = "different" };
        Assert.Equal("publication_conflict", Assert.Throws<UpdateException>(() => f.Store.Observe("catalog", conflict, null)).Code);
        Assert.Equal(43, new UpdateStateStore(f.Store.DirectoryPath).Cached("catalog")!.Revision);
    }
}

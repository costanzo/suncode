using System.Net;
using SunCode.Desktop.Infrastructure.Updates;

namespace SunCode.Desktop.Tests.Updates;

public sealed class UpdateTransportTests
{
    [Theory]
    [InlineData("win-x64")]
    [InlineData("win-arm64")]
    [InlineData("osx-x64")]
    [InlineData("osx-arm64")]
    [InlineData("linux-x64")]
    [InlineData("linux-arm64")]
    public async Task PackagedDistributionUsesSunCodeUpdatePathForEachTarget(string target)
    {
        using var f = new UpdateFixture();
        var configured = UpdateJson.Read<UpdateDistribution>(File.ReadAllBytes(Path.Combine(AppContext.BaseDirectory, "update-distribution.json")));
        Assert.True(configured.Enabled);
        Assert.True(configured.AllowsDownload(new Uri("https://suncode.ai/updates/releases/0.1.0/sdk.zip")));
        Assert.True(configured.AllowsDownload(new Uri("https://www.suncode.ai/updates/releases/0.1.0/sdk.zip")));
        Assert.False(configured.AllowsDownload(new Uri("https://other.example/sdk.zip")));
        using var client = new UpdateTransport(configured, f.Store, new(target, "99.0", null), f.Handler, f.Time);
        f.Handler.Respond = (request, _) =>
        {
            Assert.Equal($"https://www.suncode.ai/updates/v1/stable/{target}/latest.json", request.RequestUri!.AbsoluteUri);
            return Task.FromResult(new HttpResponseMessage(HttpStatusCode.NotFound));
        };
        Assert.Equal("check_failed", (await Assert.ThrowsAsync<UpdateException>(() => client.CheckAsync(default))).Code);
        Assert.Equal(1, f.Handler.Requests);
    }
    [Fact]
    public async Task PackagedDistributionAllowsVerifiedArchiveRedirectFromApexToWww()
    {
        using var f = new UpdateFixture();
        var configured = UpdateJson.Read<UpdateDistribution>(File.ReadAllBytes(Path.Combine(AppContext.BaseDirectory, "update-distribution.json")));
        using var client = new UpdateTransport(configured, f.Store, f.Environment, f.Handler, f.Time);
        f.Handler.Respond = (request, _) =>
        {
            if (request.RequestUri!.Host == "suncode.ai")
            {
                var redirect = new HttpResponseMessage(HttpStatusCode.PermanentRedirect);
                redirect.Headers.Location = new("https://www.suncode.ai/sdk.zip");
                return Task.FromResult(redirect);
            }
            Assert.Equal("https://www.suncode.ai/sdk.zip", request.RequestUri.AbsoluteUri);
            return Task.FromResult(new HttpResponseMessage(HttpStatusCode.OK) { Content = new ByteArrayContent(f.Packages["/sdk.zip"]) });
        };
        var destination = Path.Combine(f.Root, "download.zip");
        await client.DownloadAsync(f.Catalog.Components.Sdk.Package with { Url = "https://suncode.ai/sdk.zip" }, destination, null, default);
        Assert.Equal(f.Catalog.Components.Sdk.Package.Sha256, UpdateStateStore.HashFile(destination));
        Assert.Equal(2, f.Handler.Requests);
    }
    [Theory]
    [InlineData(false, "catalog_expired")]
    [InlineData(true, "clock_invalid")]
    public async Task UnusableAuthenticatedTimesStillPersistRevisionBeforeRejectingReplay(bool future, string code)
    {
        using var f = new UpdateFixture();
        using var client = new UpdateTransport(f.Distribution, f.Store, f.Environment, f.Handler, f.Time);
        await client.CheckAsync(default);
        var newer = f.Sign(j =>
        {
            j["revision"] = 43;
            j["publishedAt"] = future ? "2026-11-02T02:00:00Z" : "2026-10-24T02:00:00Z";
            j["expiresAt"] = future ? "2026-11-08T02:00:00Z" : "2026-10-31T02:00:00Z";
        });
        f.Handler.Respond = (_, _) => Task.FromResult(UpdateFixture.JsonResponse(newer));
        Assert.Equal(code, (await Assert.ThrowsAsync<UpdateException>(() => client.CheckAsync(default))).Code);
        Assert.Equal(43, new UpdateStateStore(f.Store.DirectoryPath).Cached(client.CatalogUrl)!.Revision);
        f.Handler.Respond = (_, _) => Task.FromResult(UpdateFixture.JsonResponse(f.Sign()));
        Assert.Equal("catalog_rollback", (await Assert.ThrowsAsync<UpdateException>(() => client.CheckAsync(default))).Code);
    }
    [Fact]
    public async Task ConditionalCheckRecomputesAgainstAuthenticatedCacheWithoutSendingInstalledIdentity()
    {
        using var f = new UpdateFixture();
        using var client = new UpdateTransport(f.Distribution, f.Store, f.Environment, f.Handler, f.Time);
        var first = await client.CheckAsync(default);
        f.Handler.Respond = (request, _) =>
        {
            Assert.Equal("\"revision42\"", Assert.Single(request.Headers.IfNoneMatch).ToString());
            Assert.Null(request.Content); Assert.Null(request.Headers.Authorization);
            Assert.Empty(request.RequestUri!.Query);
            Assert.True(request.Headers.CacheControl!.NoCache);
            return Task.FromResult(new HttpResponseMessage(HttpStatusCode.NotModified));
        };
        var second = await client.CheckAsync(default);
        Assert.Equal(first.SignedSha256, second.SignedSha256);
        Assert.Equal(2, f.Handler.Requests);
    }
    [Fact]
    public async Task MissingOrExpired304CacheTriggersOneUnconditionalRetry()
    {
        using var f = new UpdateFixture();
        var calls = 0;
        f.Handler.Respond = (request, _) =>
        {
            calls++; Assert.Empty(request.Headers.IfNoneMatch);
            return Task.FromResult(calls == 1 ? new HttpResponseMessage(HttpStatusCode.NotModified) : UpdateFixture.JsonResponse(f.Sign()));
        };
        using var client = new UpdateTransport(f.Distribution, f.Store, f.Environment, f.Handler, f.Time);
        await client.CheckAsync(default);
        Assert.Equal(2, calls);
        f.Time.Now = f.Time.Now.AddDays(8);
        calls = 0;
        f.Handler.Respond = (request, _) => { calls++; if(calls==2) Assert.Empty(request.Headers.IfNoneMatch); return Task.FromResult(new HttpResponseMessage(HttpStatusCode.NotModified)); };
        await Assert.ThrowsAsync<UpdateException>(() => client.CheckAsync(default));
        Assert.Equal(2, calls);
    }
    [Theory]
    [InlineData(404)] [InlineData(429)] [InlineData(503)] [InlineData(302)]
    public async Task HttpFailuresNeverBecomeUpToDate(int status)
    {
        using var f = new UpdateFixture();
        f.Handler.Respond = (_,_) => Task.FromResult(new HttpResponseMessage((HttpStatusCode)status) { Content = new StringContent("<html>error</html>") });
        using var client = new UpdateTransport(f.Distribution, f.Store, f.Environment, f.Handler, f.Time);
        await Assert.ThrowsAsync<UpdateException>(()=>client.CheckAsync(default));
    }
    [Fact]
    public async Task DisabledDistributionMakesNoRequest()
    {
        using var f = new UpdateFixture();
        using var client = new UpdateTransport(new(), f.Store, f.Environment, f.Handler, f.Time);
        Assert.Equal("configuration_missing", (await Assert.ThrowsAsync<UpdateException>(()=>client.CheckAsync(default))).Code);
        Assert.Equal(0, f.Handler.Requests);
    }
    [Fact]
    public async Task DownloadVerifiesLengthAndHashAndRejectsUnapprovedRedirects()
    {
        using var f = new UpdateFixture();
        using var client = new UpdateTransport(f.Distribution, f.Store, f.Environment, f.Handler, f.Time);
        var path = Path.Combine(f.Root,"download.zip");
        await client.DownloadAsync(f.Catalog.Components.Sdk.Package,path,null,default);
        Assert.Equal(f.Catalog.Components.Sdk.Package.Sha256, UpdateStateStore.HashFile(path));
        File.Delete(path);
        await Assert.ThrowsAsync<UpdateException>(()=>client.DownloadAsync(f.Catalog.Components.Sdk.Package with { Sha256 = new string('e',64) },path,null,default));
        Assert.False(File.Exists(path)); Assert.False(File.Exists(path+".partial"));
        f.Handler.Respond=(_,_)=>
        {
            var response=new HttpResponseMessage(HttpStatusCode.Redirect);response.Headers.Location=new("https://other.suncode.example/payload.zip");return Task.FromResult(response);
        };
        Assert.Equal("download_origin",(await Assert.ThrowsAsync<UpdateException>(()=>client.DownloadAsync(f.Catalog.Components.Sdk.Package,path,null,default))).Code);
    }
    [Fact]
    public async Task OversizedCatalogDoesNotOverwriteVerifiedCache()
    {
        using var f = new UpdateFixture();
        using var client = new UpdateTransport(f.Distribution, f.Store, f.Environment, f.Handler, f.Time);
        var good=await client.CheckAsync(default);
        f.Handler.Respond=(_,_)=>Task.FromResult(UpdateFixture.JsonResponse(new byte[CatalogVerifier.MaximumCatalogBytes+1]));
        await Assert.ThrowsAsync<UpdateException>(()=>client.CheckAsync(default));
        Assert.Equal(good.SignedSha256,f.Store.Cached(client.CatalogUrl)!.SignedSha256);
    }
    [Fact]
    public async Task AuthenticatedButMalformedNewerBodyStillAdvancesRevisionAndRetryAfterIsRespected()
    {
        using var f=new UpdateFixture();
        using var client=new UpdateTransport(f.Distribution,f.Store,f.Environment,f.Handler,f.Time);
        f.Handler.Respond=(_,_)=>Task.FromResult(UpdateFixture.JsonResponse(f.Sign(j=> { j["revision"]=43;j["components"]!["sdk"]!["package"]!["sizeBytes"]="not an integer"; })));
        await Assert.ThrowsAsync<UpdateException>(()=>client.CheckAsync(default));
        Assert.Equal(43,f.Store.Cached(client.CatalogUrl)!.Revision);
        f.Handler.Respond=(_,_)=>
        {
            var response=new HttpResponseMessage(HttpStatusCode.TooManyRequests);
            response.Headers.RetryAfter=new(TimeSpan.FromDays(3));return Task.FromResult(response);
        };
        await Assert.ThrowsAsync<UpdateException>(()=>client.CheckAsync(default));
        var requests=f.Handler.Requests;
        await Assert.ThrowsAsync<UpdateException>(()=>client.CheckAsync(default));
        Assert.Equal(requests,f.Handler.Requests);
        Assert.Equal(f.Time.Now.AddDays(1),client.RetryNotBefore);
    }
}

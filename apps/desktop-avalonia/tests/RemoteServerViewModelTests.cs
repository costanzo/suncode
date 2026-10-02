using SunCode.Desktop.ViewModels;
using SunCode.Sdk.Models;

namespace SunCode.Desktop.Tests;

public sealed class RemoteServerViewModelTests
{
    [Fact]
    public async Task OperationsReturnUnavailableStatusWithoutSdk()
    {
        using var viewModel = new RemoteServerViewModel(new FakeViewModelHost());

        var loaded = await viewModel.LoadAsync();
        var saved = await viewModel.SaveAsync("https://remote.example", "code");
        var disconnected = await viewModel.DisconnectAsync();
        var cleared = await viewModel.ClearAsync();

        Assert.Equal(string.Empty, loaded.Configuration.ServerUrl);
        Assert.Equal("Local agent unavailable", loaded.Status.Error);
        Assert.Equal("Local agent unavailable", saved.Error);
        Assert.False(disconnected.Connected);
        Assert.False(cleared.Configured);
        Assert.False(viewModel.Configured);
        Assert.False(viewModel.Connected);
    }

    [Fact]
    public void ApplyStatusUpdatesFlagsAndStatusText()
    {
        using var viewModel = new RemoteServerViewModel(new FakeViewModelHost());
        var changed = new List<string?>();
        viewModel.PropertyChanged += (_, args) => changed.Add(args.PropertyName);

        viewModel.ApplyStatus(new RemoteServerStatus(true, false, true, null, null, null));
        Assert.True(viewModel.Configured);
        Assert.True(viewModel.Connecting);
        Assert.Equal("Remote connecting", viewModel.StatusText);

        viewModel.ApplyStatus(new RemoteServerStatus(true, true, false, null, null, null));
        Assert.True(viewModel.Connected);
        Assert.Equal("Remote connected", viewModel.StatusText);
        Assert.Contains(nameof(RemoteServerViewModel.StatusText), changed);
    }

    [Fact]
    public void StatusCarriesPairingMetadataAndExpiry()
    {
        using var viewModel = new RemoteServerViewModel(new FakeViewModelHost());
        var status = new RemoteServerStatus(
            true, true, false, "host-1", "https://remote.example?code=once&hostId=host-1&k=aes", null,
            "2026-10-02T03:47:22.778Z", "once", "https://remote.example?code=once&hostId=host-1&k=aes");

        viewModel.ApplyStatus(status);

        Assert.True(viewModel.Configured);
        Assert.True(viewModel.Connected);
        Assert.Equal("Remote connected", viewModel.StatusText);
    }
}

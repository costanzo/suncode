using System.Collections.ObjectModel;
using SunCode.Desktop.Infrastructure;
using SunCode.Desktop.Models;
using SunCode.Sdk.Models;

namespace SunCode.Desktop.ViewModels;

public sealed class LanguageServersViewModel : ObservableObject, IPolledPage
{
    // Activation starts the project runtimes and loads in parallel, as before.
    Task IPolledPage.ActivateAsync() => Task.WhenAll(StartProjectAsync(), LoadServersAsync());
    Task IPolledPage.PollAsync() => LoadServersAsync();
    void IPolledPage.Deactivate() { }

    private readonly IViewModelHost _host;
    private bool _loading;
    private string _statusText = string.Empty;

    internal LanguageServersViewModel(IViewModelHost host) => _host = host;

    public ObservableCollection<LanguageServerItem> Servers { get; } = [];
    public bool HasServers => Servers.Count > 0;
    public bool HasNoServers => !HasServers;
    public string StatusText
    {
        get => _statusText;
        private set => SetProperty(ref _statusText, value);
    }

    public async Task StartProjectAsync()
    {
        if (_host.SelectedProject is null || !await _host.EnsureSdkReadyAsync()) return;
        try
        {
            await _host.Sdk!.StartLanguageServerProjectAsync(_host.SelectedProject.ProjectId);
        }
        catch (Exception exception)
        {
            StatusText = exception.Message;
            _host.ReportError(exception);
        }
    }

    public async Task LoadServersAsync()
    {
        if (_loading || !await _host.EnsureSdkReadyAsync()) return;
        _loading = true;
        try
        {
            var result = await _host.Sdk!.GetLanguageServersAsync(_host.SelectedProject?.ProjectId);
            ReplaceServers(result.Servers);
            StatusText = string.Empty;
        }
        catch (Exception exception)
        {
            StatusText = exception.Message;
            _host.ReportError(exception);
        }
        finally
        {
            _loading = false;
        }
    }

    public async Task<bool> CreateServerAsync(LanguageServerWriteRequest request)
    {
        if (!await _host.EnsureSdkReadyAsync()) return false;
        return await RunMutationAsync(null, async () =>
        {
            var server = await _host.Sdk!.CreateLanguageServerAsync(new(
                _host.SelectedProject?.ProjectId,
                Guid.NewGuid().ToString("N"),
                request));
            UpsertServer(server);
        }, request.Enabled ? "Language server saved and started." : "Language server saved as disabled.");
    }

    public async Task<bool> UpdateServerAsync(
        LanguageServerItem item,
        LanguageServerWriteRequest request)
    {
        if (!await _host.EnsureSdkReadyAsync()) return false;
        return await RunMutationAsync(item, async () =>
        {
            var server = await _host.Sdk!.UpdateLanguageServerAsync(new(
                _host.SelectedProject?.ProjectId,
                item.ServerId,
                item.Revision,
                Guid.NewGuid().ToString("N"),
                request));
            UpsertServer(server);
        }, "Language server changes saved and applied.");
    }

    public async Task<bool> SetServerEnabledAsync(LanguageServerItem item, bool enabled)
    {
        if (!await _host.EnsureSdkReadyAsync()) return false;
        return await RunMutationAsync(item, async () =>
        {
            var server = await _host.Sdk!.SetLanguageServerEnabledAsync(new(
                _host.SelectedProject?.ProjectId,
                item.ServerId,
                item.Revision,
                Guid.NewGuid().ToString("N"),
                enabled));
            UpsertServer(server);
        }, enabled ? "Language server enabled." : "Language server disabled.");
    }

    public async Task<bool> RetryServerAsync(LanguageServerItem item)
    {
        if (!await _host.EnsureSdkReadyAsync() || _host.SelectedProject is null) return false;
        return await RunMutationAsync(item, async () =>
        {
            var server = await _host.Sdk!.RetryLanguageServerAsync(_host.SelectedProject.ProjectId, item.ServerId);
            UpsertServer(server);
        }, "Language server restarted.");
    }

    public async Task<bool> DeleteServerAsync(LanguageServerItem item)
    {
        if (!await _host.EnsureSdkReadyAsync()) return false;
        return await RunMutationAsync(item, async () =>
        {
            var result = await _host.Sdk!.DeleteLanguageServerAsync(new(
                item.ServerId,
                item.Revision,
                Guid.NewGuid().ToString("N")));
            if (result.Removed) Servers.Remove(item);
            NotifyCollectionChanged();
        }, "Language server deleted.");
    }

    private async Task<bool> RunMutationAsync(
        LanguageServerItem? item,
        Func<Task> operation,
        string success)
    {
        item?.IsPending = true;
        try
        {
            await operation();
            StatusText = success;
            return true;
        }
        catch (Exception exception)
        {
            StatusText = exception.Message;
            _host.ReportError(exception);
            return false;
        }
        finally
        {
            if (item is not null) item.IsPending = false;
        }
    }

    private void ReplaceServers(IReadOnlyList<LanguageServer> servers)
    {
        for (var targetIndex = 0; targetIndex < servers.Count; targetIndex++)
        {
            var server = servers[targetIndex];
            var existing = Servers.FirstOrDefault(item => item.ServerId == server.LanguageServerId);
            if (existing is null)
            {
                Servers.Insert(targetIndex, new LanguageServerItem(server));
                continue;
            }
            existing.Replace(server);
            var currentIndex = Servers.IndexOf(existing);
            if (currentIndex != targetIndex) Servers.Move(currentIndex, targetIndex);
        }
        var ids = servers.Select(server => server.LanguageServerId).ToHashSet(StringComparer.Ordinal);
        for (var index = Servers.Count - 1; index >= 0; index--)
        {
            if (!ids.Contains(Servers[index].ServerId)) Servers.RemoveAt(index);
        }
        NotifyCollectionChanged();
    }

    private void UpsertServer(LanguageServer server)
    {
        var existing = Servers.FirstOrDefault(item => item.ServerId == server.LanguageServerId);
        if (existing is null) Servers.Add(new LanguageServerItem(server));
        else existing.Replace(server);
        NotifyCollectionChanged();
    }

    private void NotifyCollectionChanged()
    {
        OnPropertyChanged(nameof(HasServers));
        OnPropertyChanged(nameof(HasNoServers));
    }
}

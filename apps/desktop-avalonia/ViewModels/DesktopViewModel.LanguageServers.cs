using System.Collections.ObjectModel;
using SunCode.Desktop.Models;
using SunCode.Sdk.Models;

namespace SunCode.Desktop.ViewModels;

public sealed partial class DesktopViewModel
{
    private bool _languageServersLoading;
    private string _languageServerStatusText = string.Empty;

    public ObservableCollection<LanguageServerItem> LanguageServers { get; } = [];
    public bool HasLanguageServers => LanguageServers.Count > 0;
    public bool HasNoLanguageServers => !HasLanguageServers;
    public string LanguageServerStatusText
    {
        get => _languageServerStatusText;
        private set => SetProperty(ref _languageServerStatusText, value);
    }

    public async Task StartLanguageServerProjectAsync()
    {
        if (SelectedProject is null || !await EnsureSdkReadyAsync()) return;
        try
        {
            await _sdk!.StartLanguageServerProjectAsync(SelectedProject.ProjectId);
        }
        catch (Exception exception)
        {
            LanguageServerStatusText = exception.Message;
            ReportError(exception);
        }
    }

    public async Task LoadLanguageServersAsync()
    {
        if (_languageServersLoading || !await EnsureSdkReadyAsync()) return;
        _languageServersLoading = true;
        try
        {
            var result = await _sdk!.GetLanguageServersAsync(SelectedProject?.ProjectId);
            ReplaceLanguageServers(result.Servers);
            LanguageServerStatusText = string.Empty;
        }
        catch (Exception exception)
        {
            LanguageServerStatusText = exception.Message;
            ReportError(exception);
        }
        finally
        {
            _languageServersLoading = false;
        }
    }

    public async Task<bool> CreateLanguageServerAsync(LanguageServerWriteRequest request)
    {
        if (!await EnsureSdkReadyAsync()) return false;
        return await RunLanguageServerMutationAsync(null, async () =>
        {
            var server = await _sdk!.CreateLanguageServerAsync(new(
                SelectedProject?.ProjectId,
                Guid.NewGuid().ToString("N"),
                request));
            UpsertLanguageServer(server);
        }, request.Enabled ? "Language server saved and started." : "Language server saved as disabled.");
    }

    public async Task<bool> UpdateLanguageServerAsync(
        LanguageServerItem item,
        LanguageServerWriteRequest request)
    {
        if (!await EnsureSdkReadyAsync()) return false;
        return await RunLanguageServerMutationAsync(item, async () =>
        {
            var server = await _sdk!.UpdateLanguageServerAsync(new(
                SelectedProject?.ProjectId,
                item.ServerId,
                item.Revision,
                Guid.NewGuid().ToString("N"),
                request));
            UpsertLanguageServer(server);
        }, "Language server changes saved and applied.");
    }

    public async Task<bool> SetLanguageServerEnabledAsync(LanguageServerItem item, bool enabled)
    {
        if (!await EnsureSdkReadyAsync()) return false;
        return await RunLanguageServerMutationAsync(item, async () =>
        {
            var server = await _sdk!.SetLanguageServerEnabledAsync(new(
                SelectedProject?.ProjectId,
                item.ServerId,
                item.Revision,
                Guid.NewGuid().ToString("N"),
                enabled));
            UpsertLanguageServer(server);
        }, enabled ? "Language server enabled." : "Language server disabled.");
    }

    public async Task<bool> RetryLanguageServerAsync(LanguageServerItem item)
    {
        if (!await EnsureSdkReadyAsync() || SelectedProject is null) return false;
        return await RunLanguageServerMutationAsync(item, async () =>
        {
            var server = await _sdk!.RetryLanguageServerAsync(SelectedProject.ProjectId, item.ServerId);
            UpsertLanguageServer(server);
        }, "Language server restarted.");
    }

    public async Task<bool> DeleteLanguageServerAsync(LanguageServerItem item)
    {
        if (!await EnsureSdkReadyAsync()) return false;
        return await RunLanguageServerMutationAsync(item, async () =>
        {
            var result = await _sdk!.DeleteLanguageServerAsync(new(
                item.ServerId,
                item.Revision,
                Guid.NewGuid().ToString("N")));
            if (result.Removed) LanguageServers.Remove(item);
            NotifyLanguageServerCollectionChanged();
        }, "Language server deleted.");
    }

    private async Task<bool> RunLanguageServerMutationAsync(
        LanguageServerItem? item,
        Func<Task> operation,
        string success)
    {
        item?.IsPending = true;
        try
        {
            await operation();
            LanguageServerStatusText = success;
            return true;
        }
        catch (Exception exception)
        {
            LanguageServerStatusText = exception.Message;
            ReportError(exception);
            return false;
        }
        finally
        {
            if (item is not null) item.IsPending = false;
        }
    }

    private void ReplaceLanguageServers(IReadOnlyList<LanguageServer> servers)
    {
        for (var targetIndex = 0; targetIndex < servers.Count; targetIndex++)
        {
            var server = servers[targetIndex];
            var existing = LanguageServers.FirstOrDefault(item => item.ServerId == server.LanguageServerId);
            if (existing is null)
            {
                LanguageServers.Insert(targetIndex, new LanguageServerItem(server));
                continue;
            }
            existing.Replace(server);
            var currentIndex = LanguageServers.IndexOf(existing);
            if (currentIndex != targetIndex) LanguageServers.Move(currentIndex, targetIndex);
        }
        var ids = servers.Select(server => server.LanguageServerId).ToHashSet(StringComparer.Ordinal);
        for (var index = LanguageServers.Count - 1; index >= 0; index--)
        {
            if (!ids.Contains(LanguageServers[index].ServerId)) LanguageServers.RemoveAt(index);
        }
        NotifyLanguageServerCollectionChanged();
    }

    private void UpsertLanguageServer(LanguageServer server)
    {
        var existing = LanguageServers.FirstOrDefault(item => item.ServerId == server.LanguageServerId);
        if (existing is null) LanguageServers.Add(new LanguageServerItem(server));
        else existing.Replace(server);
        NotifyLanguageServerCollectionChanged();
    }

    private void NotifyLanguageServerCollectionChanged()
    {
        OnPropertyChanged(nameof(HasLanguageServers));
        OnPropertyChanged(nameof(HasNoLanguageServers));
    }
}

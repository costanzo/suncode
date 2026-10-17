using System.Collections.ObjectModel;
using SunCode.Desktop.Infrastructure;
using SunCode.Desktop.Models;
using SunCode.Sdk.Models;

namespace SunCode.Desktop.ViewModels;

public sealed class LanguageServersViewModel : ObservableObject, IPolledPage
{
    Task IPolledPage.ActivateAsync() => Task.WhenAll(StartProjectAsync(), LoadServersAsync());
    Task IPolledPage.PollAsync() => LoadServersAsync();
    void IPolledPage.Deactivate() { }

    private readonly IViewModelHost _host;
    private bool _loading;
    private string _statusText = string.Empty;

    internal LanguageServersViewModel(IViewModelHost host) => _host = host;

    public ObservableCollection<LanguageServerItem> Servers { get; } = [];
    public string ProjectName => _host.SelectedProject?.DisplayName ?? string.Empty;
    public string ProjectPath => _host.SelectedProject?.CanonicalRoot ?? string.Empty;
    public bool HasProject => _host.SelectedProject is not null;
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

    public async Task<bool> SaveJdtlsAsync(
        LanguageServerItem item,
        string executable,
        string? javaHome,
        string dataPathMode,
        string? dataPath,
        IReadOnlyList<string> rootMarkers,
        LanguageServerEnvironmentChanges? environment,
        ulong startupTimeout,
        ulong requestTimeout,
        bool enabled)
    {
        if (_host.SelectedProject is null || !await _host.EnsureSdkReadyAsync()) return false;
        return await RunMutationAsync(item, async () =>
        {
            var server = await _host.Sdk!.ConfigureJdtlsLanguageServerAsync(new(
                _host.SelectedProject.ProjectId,
                item.Revision,
                Guid.NewGuid().ToString("N"),
                new(
                    executable,
                    javaHome,
                    dataPathMode,
                    dataPath,
                    rootMarkers,
                    environment,
                    startupTimeout,
                    requestTimeout,
                    enabled)));
            UpsertServer(server);
        }, enabled
            ? "jdtls configuration saved. Restarting the project language server."
            : "jdtls configuration saved as disabled for this project.");
    }

    public async Task<bool> SaveVtslsAsync(
        LanguageServerItem item,
        string executable,
        string nodePath,
        string typescriptSdkMode,
        string? typescriptSdkPath,
        IReadOnlyList<string> rootMarkers,
        ulong maxTsServerMemory,
        LanguageServerEnvironmentChanges? environment,
        ulong startupTimeout,
        ulong requestTimeout,
        bool enabled)
    {
        if (_host.SelectedProject is null || !await _host.EnsureSdkReadyAsync()) return false;
        return await RunMutationAsync(item, async () =>
        {
            var server = await _host.Sdk!.ConfigureVtslsLanguageServerAsync(new(
                _host.SelectedProject.ProjectId,
                item.Revision,
                Guid.NewGuid().ToString("N"),
                new(
                    executable,
                    nodePath,
                    typescriptSdkMode,
                    typescriptSdkPath,
                    rootMarkers,
                    maxTsServerMemory,
                    environment,
                    startupTimeout,
                    requestTimeout,
                    enabled)));
            UpsertServer(server);
        }, enabled
            ? "vtsls configuration saved. Restarting the project language server."
            : "vtsls configuration saved as disabled for this project.");
    }

    public async Task<bool> SetServerEnabledAsync(LanguageServerItem item, bool enabled)
    {
        if (_host.SelectedProject is null || !await _host.EnsureSdkReadyAsync()) return false;
        return await RunMutationAsync(item, async () =>
        {
            var server = await _host.Sdk!.SetLanguageServerEnabledAsync(new(
                _host.SelectedProject.ProjectId,
                item.ServerId,
                item.Revision,
                Guid.NewGuid().ToString("N"),
                enabled));
            UpsertServer(server);
        }, enabled
            ? $"{item.DisplayName} enabled for this project."
            : $"{item.DisplayName} disabled for this project.");
    }

    public async Task<bool> RetryServerAsync(LanguageServerItem item)
    {
        if (_host.SelectedProject is null || !await _host.EnsureSdkReadyAsync()) return false;
        return await RunMutationAsync(item, async () =>
        {
            var server = await _host.Sdk!.RetryLanguageServerAsync(
                _host.SelectedProject.ProjectId,
                item.ServerId);
            UpsertServer(server);
        }, $"{item.DisplayName} restarted for this project.");
    }

    private async Task<bool> RunMutationAsync(
        LanguageServerItem item,
        Func<Task> operation,
        string success)
    {
        item.IsPending = true;
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
            item.IsPending = false;
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
        OnPropertyChanged(nameof(ProjectName));
        OnPropertyChanged(nameof(ProjectPath));
        OnPropertyChanged(nameof(HasProject));
    }

    private void UpsertServer(LanguageServer server)
    {
        var existing = Servers.FirstOrDefault(item => item.ServerId == server.LanguageServerId);
        if (existing is null) Servers.Add(new LanguageServerItem(server));
        else existing.Replace(server);
    }
}

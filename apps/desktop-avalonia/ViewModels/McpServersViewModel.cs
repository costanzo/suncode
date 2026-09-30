using System.Collections.ObjectModel;
using SunCode.Desktop.Infrastructure;
using SunCode.Desktop.Models;
using SunCode.Sdk.Models;

namespace SunCode.Desktop.ViewModels;

public sealed class McpServersViewModel : ObservableObject
{
    private readonly IViewModelHost _host;
    private bool _loading;
    private string _statusText = string.Empty;
    private McpLoadProgress? _loadProgress;

    internal McpServersViewModel(IViewModelHost host) => _host = host;

    public ObservableCollection<McpServerItem> Servers { get; } = [];
    public bool HasServers => Servers.Count > 0;
    public bool HasNoServers => !HasServers;
    public bool HasProjectContext => _host.SelectedProject is not null;
    public string ProjectContext => _host.SelectedProject is { } project
        ? $"Project connection: {project.DisplayName}"
        : "Open a project to start MCP connections.";
    public string StatusText
    {
        get => _statusText;
        private set => SetProperty(ref _statusText, value);
    }

    public bool IsLoading => _loadProgress?.Loading == true;
    public double LoadPercent => _loadProgress is { Total: > 0 } progress
        ? progress.Settled * 100d / progress.Total
        : 0d;
    public string LoadText => _loadProgress is { } progress
        ? $"MCP {progress.Settled}/{progress.Total}"
        : string.Empty;
    public string LoadDetails => _loadProgress is { } progress
        ? $"{progress.Connected} connected{(progress.Failed > 0 ? $" · {progress.Failed} failed" : string.Empty)} · {Math.Max(0, progress.Total - progress.Settled)} starting"
        : string.Empty;

    public async Task StartProjectAsync()
    {
        if (_host.SelectedProject is null || !await _host.EnsureSdkReadyAsync()) return;
        try
        {
            _loadProgress = await _host.Sdk!.StartMcpProjectAsync(_host.SelectedProject.ProjectId);
            NotifyProgressChanged();
        }
        catch (Exception exception)
        {
            StatusText = exception.Message;
            _host.ReportError(exception);
        }
    }

    public async Task RefreshLoadProgressAsync()
    {
        if (!IsLoading || _host.SelectedProject is null || !_host.EnsureSdk()) return;
        try
        {
            _loadProgress = await _host.Sdk!.GetMcpLoadProgressAsync(_host.SelectedProject.ProjectId);
            NotifyProgressChanged();
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
            var result = await _host.Sdk!.GetMcpServersAsync(_host.SelectedProject?.ProjectId);
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

    public async Task<bool> CreateServerAsync(McpServerWriteRequest request)
    {
        if (!await _host.EnsureSdkReadyAsync()) return false;
        return await RunMutationAsync(null, async () =>
        {
            var server = await _host.Sdk!.CreateMcpServerAsync(new(
                _host.SelectedProject?.ProjectId,
                Guid.NewGuid().ToString("N"),
                request));
            UpsertServer(server);
        }, request.Enabled ? "Saved to SQLite. Connection state updated." : "Saved to SQLite as disabled.");
    }

    public async Task<bool> UpdateServerAsync(McpServerItem item, McpServerWriteRequest request)
    {
        if (!await _host.EnsureSdkReadyAsync()) return false;
        return await RunMutationAsync(item, async () =>
        {
            var server = await _host.Sdk!.UpdateMcpServerAsync(new(
                _host.SelectedProject?.ProjectId,
                item.ServerId,
                item.Revision,
                Guid.NewGuid().ToString("N"),
                request));
            UpsertServer(server);
        }, "Server changes saved and applied.");
    }

    public async Task<bool> SetServerEnabledAsync(McpServerItem item, bool enabled)
    {
        if (!await _host.EnsureSdkReadyAsync()) return false;
        return await RunMutationAsync(item, async () =>
        {
            var server = await _host.Sdk!.SetMcpServerEnabledAsync(new(
                _host.SelectedProject?.ProjectId,
                item.ServerId,
                item.Revision,
                Guid.NewGuid().ToString("N"),
                enabled));
            UpsertServer(server);
        }, enabled ? "Server enabled and reconciled." : "Server disabled. Its tools are unavailable to new calls.");
    }

    public async Task<bool> RetryServerAsync(McpServerItem item)
    {
        if (!await _host.EnsureSdkReadyAsync() || _host.SelectedProject is null) return false;
        return await RunMutationAsync(item, async () =>
        {
            var server = await _host.Sdk!.RetryMcpServerAsync(_host.SelectedProject.ProjectId, item.ServerId);
            UpsertServer(server);
        }, "MCP server connection retried.");
    }

    public async Task<bool> DeleteServerAsync(McpServerItem item)
    {
        if (!await _host.EnsureSdkReadyAsync()) return false;
        return await RunMutationAsync(item, async () =>
        {
            var result = await _host.Sdk!.DeleteMcpServerAsync(new(
                item.ServerId,
                item.Revision,
                Guid.NewGuid().ToString("N")));
            if (result.Removed) Servers.Remove(item);
            NotifyCollectionChanged();
        }, "Server deleted. Its tools were removed from new model requests.");
    }

    private async Task<bool> RunMutationAsync(
        McpServerItem? item,
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

    private void ReplaceServers(IReadOnlyList<McpServer> servers)
    {
        for (var targetIndex = 0; targetIndex < servers.Count; targetIndex++)
        {
            var server = servers[targetIndex];
            var existing = Servers.FirstOrDefault(item => item.ServerId == server.McpServerId);
            if (existing is null)
            {
                Servers.Insert(targetIndex, new McpServerItem(server));
                continue;
            }
            existing.Replace(server);
            var currentIndex = Servers.IndexOf(existing);
            if (currentIndex != targetIndex) Servers.Move(currentIndex, targetIndex);
        }
        var ids = servers.Select(server => server.McpServerId).ToHashSet(StringComparer.Ordinal);
        for (var index = Servers.Count - 1; index >= 0; index--)
        {
            if (!ids.Contains(Servers[index].ServerId)) Servers.RemoveAt(index);
        }
        NotifyCollectionChanged();
    }

    private void UpsertServer(McpServer server)
    {
        var existing = Servers.FirstOrDefault(item => item.ServerId == server.McpServerId);
        if (existing is null)
        {
            Servers.Add(new McpServerItem(server));
        }
        else
        {
            existing.Replace(server);
        }
        NotifyCollectionChanged();
    }

    internal void OnProjectChanged()
    {
        OnPropertyChanged(nameof(ProjectContext));
        OnPropertyChanged(nameof(HasProjectContext));
    }

    private void NotifyCollectionChanged()
    {
        OnPropertyChanged(nameof(HasServers));
        OnPropertyChanged(nameof(HasNoServers));
        OnPropertyChanged(nameof(ProjectContext));
        OnPropertyChanged(nameof(HasProjectContext));
    }

    private void NotifyProgressChanged()
    {
        OnPropertyChanged(nameof(IsLoading));
        OnPropertyChanged(nameof(LoadPercent));
        OnPropertyChanged(nameof(LoadText));
        OnPropertyChanged(nameof(LoadDetails));
    }
}

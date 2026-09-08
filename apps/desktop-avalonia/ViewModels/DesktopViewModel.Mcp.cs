using System.Collections.ObjectModel;
using SunCode.Desktop.Models;
using SunCode.Sdk.Models;

namespace SunCode.Desktop.ViewModels;

public sealed partial class DesktopViewModel
{
    private bool _mcpLoading;
    private string _mcpStatusText = string.Empty;

    public ObservableCollection<McpServerItem> McpServers { get; } = [];
    public bool McpHasServers => McpServers.Count > 0;
    public bool McpHasNoServers => !McpHasServers;
    public bool HasMcpProjectContext => SelectedProject is not null;
    public string McpProjectContext => SelectedProject is { } project
        ? $"Project connection: {project.DisplayName}"
        : "Open a project to start MCP connections.";
    public string McpStatusText
    {
        get => _mcpStatusText;
        private set => SetProperty(ref _mcpStatusText, value);
    }

    public async Task LoadMcpServersAsync()
    {
        if (_mcpLoading || !await EnsureSdkReadyAsync()) return;
        _mcpLoading = true;
        try
        {
            var result = await _sdk!.GetMcpServersAsync(SelectedProject?.ProjectId);
            ReplaceMcpServers(result.Servers);
            McpStatusText = string.Empty;
        }
        catch (Exception exception)
        {
            McpStatusText = exception.Message;
            ReportError(exception);
        }
        finally
        {
            _mcpLoading = false;
        }
    }

    public async Task<bool> CreateMcpServerAsync(McpServerWriteRequest request)
    {
        if (!await EnsureSdkReadyAsync()) return false;
        return await RunMcpMutationAsync(null, async () =>
        {
            var server = await _sdk!.CreateMcpServerAsync(new(
                SelectedProject?.ProjectId,
                Guid.NewGuid().ToString("N"),
                request));
            UpsertMcpServer(server);
        }, request.Enabled ? "Saved to SQLite. Connection state updated." : "Saved to SQLite as disabled.");
    }

    public async Task<bool> UpdateMcpServerAsync(McpServerItem item, McpServerWriteRequest request)
    {
        if (!await EnsureSdkReadyAsync()) return false;
        return await RunMcpMutationAsync(item, async () =>
        {
            var server = await _sdk!.UpdateMcpServerAsync(new(
                SelectedProject?.ProjectId,
                item.ServerId,
                item.Revision,
                Guid.NewGuid().ToString("N"),
                request));
            UpsertMcpServer(server);
        }, "Server changes saved and applied.");
    }

    public async Task<bool> SetMcpServerEnabledAsync(McpServerItem item, bool enabled)
    {
        if (!await EnsureSdkReadyAsync()) return false;
        return await RunMcpMutationAsync(item, async () =>
        {
            var server = await _sdk!.SetMcpServerEnabledAsync(new(
                SelectedProject?.ProjectId,
                item.ServerId,
                item.Revision,
                Guid.NewGuid().ToString("N"),
                enabled));
            UpsertMcpServer(server);
        }, enabled ? "Server enabled and reconciled." : "Server disabled. Its tools are unavailable to new calls.");
    }

    public async Task<bool> RetryMcpServerAsync(McpServerItem item)
    {
        if (!await EnsureSdkReadyAsync() || SelectedProject is null) return false;
        return await RunMcpMutationAsync(item, async () =>
        {
            var server = await _sdk!.RetryMcpServerTypedAsync(SelectedProject.ProjectId, item.ServerId);
            UpsertMcpServer(server);
        }, "MCP server connection retried.");
    }

    public async Task<bool> DeleteMcpServerAsync(McpServerItem item)
    {
        if (!await EnsureSdkReadyAsync()) return false;
        return await RunMcpMutationAsync(item, async () =>
        {
            var result = await _sdk!.DeleteMcpServerAsync(new(
                item.ServerId,
                item.Revision,
                Guid.NewGuid().ToString("N")));
            if (result.Removed) McpServers.Remove(item);
            NotifyMcpCollectionChanged();
        }, "Server deleted. Its tools were removed from new model requests.");
    }

    private async Task<bool> RunMcpMutationAsync(
        McpServerItem? item,
        Func<Task> operation,
        string success)
    {
        item?.IsPending = true;
        try
        {
            await operation();
            McpStatusText = success;
            return true;
        }
        catch (Exception exception)
        {
            McpStatusText = exception.Message;
            ReportError(exception);
            return false;
        }
        finally
        {
            if (item is not null) item.IsPending = false;
        }
    }

    private void ReplaceMcpServers(IReadOnlyList<McpServer> servers)
    {
        for (var targetIndex = 0; targetIndex < servers.Count; targetIndex++)
        {
            var server = servers[targetIndex];
            var existing = McpServers.FirstOrDefault(item => item.ServerId == server.McpServerId);
            if (existing is null)
            {
                McpServers.Insert(targetIndex, new McpServerItem(server));
                continue;
            }
            existing.Replace(server);
            var currentIndex = McpServers.IndexOf(existing);
            if (currentIndex != targetIndex) McpServers.Move(currentIndex, targetIndex);
        }
        var ids = servers.Select(server => server.McpServerId).ToHashSet(StringComparer.Ordinal);
        for (var index = McpServers.Count - 1; index >= 0; index--)
        {
            if (!ids.Contains(McpServers[index].ServerId)) McpServers.RemoveAt(index);
        }
        NotifyMcpCollectionChanged();
    }

    private void UpsertMcpServer(McpServer server)
    {
        var existing = McpServers.FirstOrDefault(item => item.ServerId == server.McpServerId);
        if (existing is null)
        {
            McpServers.Add(new McpServerItem(server));
        }
        else
        {
            existing.Replace(server);
        }
        NotifyMcpCollectionChanged();
    }

    private void NotifyMcpCollectionChanged()
    {
        OnPropertyChanged(nameof(McpHasServers));
        OnPropertyChanged(nameof(McpHasNoServers));
        OnPropertyChanged(nameof(McpProjectContext));
        OnPropertyChanged(nameof(HasMcpProjectContext));
    }
}

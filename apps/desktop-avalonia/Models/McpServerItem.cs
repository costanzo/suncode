using SunCode.Desktop.Infrastructure;
using SunCode.Sdk.Models;

namespace SunCode.Desktop.Models;

public sealed class McpServerItem : ObservableObject
{
    private bool _isPending;

    public McpServerItem(McpServer server) => Server = server;

    public McpServer Server { get; private set; }
    public string ServerId => Server.McpServerId;
    public string DisplayName => Server.DisplayName;
    public string ToolPrefix => Server.ToolPrefix;
    public string TransportType => Server.TransportType;
    public string? Command => Server.Command;
    public IReadOnlyList<string> Arguments => Server.Arguments;
    public string? WorkingDirectory => Server.WorkingDirectory;
    public string? Url => Server.Url;
    public IReadOnlyList<string> EnvironmentKeys => Server.EnvironmentKeys;
    public IReadOnlyList<string> HeaderKeys => Server.HeaderKeys;
    public ulong StartupTimeoutSeconds => Server.StartupTimeoutSeconds;
    public ulong RequestTimeoutSeconds => Server.RequestTimeoutSeconds;
    public bool Enabled => Server.Enabled;
    public ulong Revision => Server.Revision;
    public string RuntimeStatus => Server.RuntimeStatus;
    public string? Error => Server.Error;
    public bool HasError => !string.IsNullOrWhiteSpace(Error);
    public bool CanRetry => Enabled && RuntimeStatus == "failed" && !IsPending;
    public bool IsConnected => RuntimeStatus == "connected";
    public bool IsConnecting => RuntimeStatus == "connecting";
    public bool IsFailed => RuntimeStatus == "failed";
    public bool IsInactive => !IsConnected && !IsConnecting && !IsFailed;
    public string StatusLabel => RuntimeStatus switch
    {
        "connected" => "Connected",
        "connecting" => "Connecting",
        "failed" => "Failed",
        "disabled" => "Disabled",
        _ => "Not started"
    };
    public string StatusDetail => RuntimeStatus switch
    {
        "connected" => $"{Server.ToolCount} tools",
        "connecting" => "Discovering tools",
        "failed" => "No tools available",
        _ => "Not running"
    };
    public string EndpointSummary => TransportType == "stdio"
        ? string.Join(' ', new[] { Command }.Concat(Arguments).Where(value => !string.IsNullOrWhiteSpace(value)))
        : Url ?? string.Empty;
    public string ToggleLabel => Enabled ? "On" : "Off";

    public bool IsPending
    {
        get => _isPending;
        set
        {
            if (!SetProperty(ref _isPending, value)) return;
            OnPropertyChanged(nameof(CanRetry));
        }
    }

    public void Replace(McpServer server)
    {
        Server = server;
        OnPropertyChanged(string.Empty);
    }
}

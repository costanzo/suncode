using SunCode.Desktop.Infrastructure;
using SunCode.Sdk.Models;

namespace SunCode.Desktop.Models;

public sealed class LanguageServerItem : ObservableObject
{
    private bool _isPending;

    public LanguageServerItem(LanguageServer server) => Server = server;

    public LanguageServer Server { get; private set; }
    public string ServerId => Server.LanguageServerId;
    public string DisplayName => Server.DisplayName;
    public string Command => Server.Command;
    public IReadOnlyList<string> Arguments => Server.Arguments;
    public IReadOnlyList<string> LanguageIds => Server.LanguageIds;
    public IReadOnlyList<string> RootMarkers => Server.RootMarkers;
    public System.Text.Json.JsonElement InitializationOptions => Server.InitializationOptions;
    public IReadOnlyList<string> EnvironmentKeys => Server.EnvironmentKeys;
    public ulong StartupTimeoutSeconds => Server.StartupTimeoutSeconds;
    public ulong RequestTimeoutSeconds => Server.RequestTimeoutSeconds;
    public bool Enabled => Server.Enabled;
    public ulong Revision => Server.Revision;
    public long SortOrder => Server.SortOrder;
    public string RuntimeStatus => Server.RuntimeStatus;
    public string? Error => Server.Error;
    public bool HasError => !string.IsNullOrWhiteSpace(Error);
    public bool IsReady => RuntimeStatus == "ready";
    public bool IsStarting => RuntimeStatus == "starting";
    public bool IsIndexing => RuntimeStatus == "indexing";
    public bool IsFailed => RuntimeStatus == "failed";
    public bool IsInactive => !IsReady && !IsStarting && !IsIndexing && !IsFailed;
    public bool CanRetry => Enabled && IsFailed && !IsPending;
    public bool CanToggle => !IsPending && !IsStarting;
    public string StatusLabel => RuntimeStatus switch
    {
        "ready" => "Ready",
        "starting" => "Starting",
        "indexing" => "Indexing",
        "failed" => "Failed",
        "disabled" => "Disabled",
        _ => "Not started"
    };
    public string StatusDetail => RuntimeStatus switch
    {
        "ready" => $"{Server.CapabilityCount} capabilities",
        "starting" => "Initializing",
        "indexing" => "Semantic data loading",
        "failed" => "No semantic results",
        _ => "Not running"
    };
    public string CommandSummary => string.Join(' ',
        new[] { Command }.Concat(Arguments).Where(value => !string.IsNullOrWhiteSpace(value)));
    public string ToggleLabel => Enabled ? "On" : "Off";

    public bool IsPending
    {
        get => _isPending;
        set
        {
            if (!SetProperty(ref _isPending, value)) return;
            OnPropertyChanged(nameof(CanRetry));
            OnPropertyChanged(nameof(CanToggle));
        }
    }

    public void Replace(LanguageServer server)
    {
        Server = server;
        OnPropertyChanged(string.Empty);
    }
}

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
        "ready" => LocalizationService.GetString("Loc_Ready", "Ready"),
        "starting" => LocalizationService.GetString("Loc_Starting", "Starting"),
        "indexing" => LocalizationService.GetString("Loc_Indexing", "Indexing"),
        "failed" => LocalizationService.GetString("Loc_Failed", "Failed"),
        "disabled" => LocalizationService.GetString("Loc_Disabled", "Disabled"),
        _ => LocalizationService.GetString("Loc_NotStarted", "Not started")
    };
    public string StatusDetail => RuntimeStatus switch
    {
        "ready" => $"{Server.CapabilityCount} {LocalizationService.GetString("Loc_Capabilities", "capabilities")}",
        "starting" => LocalizationService.GetString("Loc_Initializing", "Initializing"),
        "indexing" => LocalizationService.GetString("Loc_SemanticDataLoading", "Semantic data loading"),
        "failed" => LocalizationService.GetString("Loc_NoSemanticResults", "No semantic results"),
        _ => LocalizationService.GetString("Loc_NotRunning", "Not running")
    };
    public string CommandSummary => string.Join(' ',
        new[] { Command }.Concat(Arguments).Where(value => !string.IsNullOrWhiteSpace(value)));
    public string ToggleLabel => Enabled
        ? LocalizationService.GetString("Loc_On", "On")
        : LocalizationService.GetString("Loc_Off", "Off");

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

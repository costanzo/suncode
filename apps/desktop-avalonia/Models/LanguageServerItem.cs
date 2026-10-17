using SunCode.Desktop.Infrastructure;
using SunCode.Sdk.Models;

namespace SunCode.Desktop.Models;

public sealed class LanguageServerItem : ObservableObject
{
    private bool _isPending;

    public LanguageServerItem(LanguageServer server) => Server = server;

    public LanguageServer Server { get; private set; }
    public string ServerId => Server.LanguageServerId;
    public string Kind => Server.Kind;
    public string DisplayName => Server.DisplayName;
    public string Description => Server.Description;
    public string Language => Server.Language;
    public bool Detected => Server.Detected;
    public string? DetectedVersion => Server.DetectedVersion;
    public string? DetectedExecutable => Server.DetectedExecutable;
    public string? DetectedJavaHome => Server.DetectedJavaHome;
    public bool IsJdtls => Kind == "jdtls";
    public bool IsVtsls => Kind == "vtsls";
    public bool IsBasedpyright => Kind == "basedpyright";
    public VtslsLanguageServerConfiguration? Vtsls => Server.Vtsls;
    public string VtslsExecutable => Vtsls?.Executable ?? Command;
    public string NodePath => Vtsls?.NodePath ?? string.Empty;
    public string? DetectedNodePath => Vtsls?.DetectedNodePath;
    public string? DetectedNodeVersion => Vtsls?.DetectedNodeVersion;
    public string TypeScriptSdkMode => Vtsls?.TypeScriptSdkMode ?? "workspace-preferred";
    public string? TypeScriptSdkPath => Vtsls?.TypeScriptSdkPath;
    public ulong MaxTsServerMemory => Vtsls?.MaxTsServerMemory ?? 3072;
    public BasedpyrightLanguageServerConfiguration? Basedpyright => Server.Basedpyright;
    public string BasedpyrightExecutable => Basedpyright?.Executable ?? Command;
    public string? PythonPath => Basedpyright?.PythonPath;
    public string? DetectedPythonPath => Basedpyright?.DetectedPythonPath;
    public string? DetectedPythonVersion => Basedpyright?.DetectedPythonVersion;
    public string DiagnosticMode => Basedpyright?.DiagnosticMode ?? "openFilesOnly";
    public string TypeCheckingMode => Basedpyright?.TypeCheckingMode ?? "recommended";
    public string ConfigPathMode => Basedpyright?.ConfigPathMode ?? "automatic";
    public string? ConfigPath => Basedpyright?.ConfigPath;
    public bool ProjectConfigurable => Server.ProjectConfigurable;
    public string Command => Server.Command;
    public IReadOnlyList<string> Arguments => Server.Arguments;
    public IReadOnlyList<string> LanguageIds => Server.LanguageIds;
    public IReadOnlyList<string> RootMarkers => Server.RootMarkers;
    public System.Text.Json.JsonElement InitializationOptions => Server.InitializationOptions;
    public IReadOnlyList<string> EnvironmentKeys => Server.EnvironmentKeys;
    public ulong StartupTimeoutSeconds => Server.StartupTimeoutSeconds;
    public ulong RequestTimeoutSeconds => Server.RequestTimeoutSeconds;
    public string? JavaHome => Server.JavaHome;
    public string DataPathMode => Server.DataPathMode;
    public string DataPath => Server.DataPath;
    public string? MatchedRootMarker => Server.MatchedRootMarker;
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
    public bool CanRetry => ProjectConfigurable && Enabled && IsFailed && !IsPending;
    public bool CanToggle => ProjectConfigurable && !IsPending && !IsStarting;
    public bool CanConfigure => ProjectConfigurable && !IsPending;
    public string StatusLabel => RuntimeStatus switch
    {
        "ready" => LocalizationService.GetString("LocReady", "Ready"),
        "starting" => LocalizationService.GetString("LocStarting", "Starting"),
        "indexing" => LocalizationService.GetString("LocIndexing", "Indexing"),
        "failed" => LocalizationService.GetString("LocFailed", "Failed"),
        "disabled" => LocalizationService.GetString("LocDisabled", "Disabled"),
        _ => LocalizationService.GetString("LocNotStarted", "Not started")
    };
    public string StatusDetail => RuntimeStatus switch
    {
        "ready" => $"{Server.CapabilityCount} {LocalizationService.GetString("LocCapabilities", "capabilities")}",
        "starting" => LocalizationService.GetString("LocInitializing", "Initializing"),
        "indexing" => LocalizationService.GetString("LocSemanticDataLoading", "Semantic data loading"),
        "failed" => string.IsNullOrWhiteSpace(Error)
            ? LocalizationService.GetString("LocNoSemanticResults", "No semantic results")
            : Error,
        _ => LocalizationService.GetString("LocNotRunning", "Not running")
    };
    public string DetectionLabel => Detected
        ? $"{LocalizationService.GetString("LocDetected", "Detected")}{(string.IsNullOrWhiteSpace(DetectedVersion) ? string.Empty : $" · {DetectedVersion}")}"
        : LocalizationService.GetString("LocNotDetected", "Not detected");
    public string ToggleLabel => Enabled
        ? LocalizationService.GetString("LocOn", "On")
        : LocalizationService.GetString("LocOff", "Off");

    public bool IsPending
    {
        get => _isPending;
        set
        {
            if (!SetProperty(ref _isPending, value)) return;
            OnPropertyChanged(nameof(CanRetry));
            OnPropertyChanged(nameof(CanToggle));
            OnPropertyChanged(nameof(CanConfigure));
        }
    }

    public void Replace(LanguageServer server)
    {
        Server = server;
        OnPropertyChanged(string.Empty);
    }
}

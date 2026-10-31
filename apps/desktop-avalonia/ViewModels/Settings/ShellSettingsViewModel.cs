using System.Windows.Input;
using SunCode.Desktop.Controls;
using SunCode.Desktop.Infrastructure;
using SunCode.Sdk;
using SunCode.Sdk.Models;
using static SunCode.Desktop.ViewModels.SettingsFormatting;

namespace SunCode.Desktop.ViewModels;

public sealed class ShellEditorViewModel : ObservableObject, IDisposable
{
    private readonly string _target;
    private readonly Func<ShellCheckRequest, Task<ShellCheckResult>> _check;
    private readonly Func<ShellCheckRequest, Task<ShellCheckResult>> _save;
    private readonly AsyncRelayCommand _checkCommand;
    private readonly AsyncRelayCommand _saveCommand;
    private CancellationTokenSource? _debounce;
    private readonly SemaphoreSlim _probeGate = new(1, 1);
    private ShellConfiguration _baseline = new();
    private ShellConfiguration? _validated;
    private IReadOnlyList<SCComboBoxItem> _options = [];
    private SCComboBoxItem? _selected;
    private string _customPath = string.Empty;
    private string _resolvedPath = string.Empty;
    private string _status = string.Empty;
    private StatusTone _tone;
    private bool _loading;
    private bool _checking;
    private bool _saving;
    private bool _locked;
    private bool _disposed;
    private long _revision;

    internal ShellEditorViewModel(string target, Func<ShellCheckRequest, Task<ShellCheckResult>> check, Func<ShellCheckRequest, Task<ShellCheckResult>> save)
    {
        _target = target; _check = check; _save = save;
        _checkCommand = new AsyncRelayCommand(() => CheckAsync(false), () => CanEdit && !_checking && (Mode != "custom" || !string.IsNullOrWhiteSpace(CustomPath)));
        _saveCommand = new AsyncRelayCommand(SaveAsync, () => CanEdit && !_checking && IsDirty && _validated == Draft);
        RebuildOptions(false);
    }
    internal event Action? SavedConfigurationChanged;
    internal ShellConfiguration SavedConfiguration => _baseline;
    public string Title => _target == "agent" ? L("LocAgentShell", "Agent shell") : L("LocTerminalShell", "Terminal shell");
    public string Hint => _target == "agent" ? L("LocAgentShellTiming", "Applies to the next turn.") : L("LocTerminalShellTiming", "Applies to new or restarted terminal tabs.");
    public string PathLabel => _target == "agent" ? L("LocAgentShellPath", "Agent shell executable path") : L("LocTerminalShellPath", "Terminal shell executable path");
    public IReadOnlyList<SCComboBoxItem> Options { get => _options; private set => SetProperty(ref _options, value); }
    public SCComboBoxItem? SelectedOption
    {
        get => _selected;
        set { if (!SetProperty(ref _selected, value) || _loading) return; OnPropertyChanged(nameof(IsCustom)); OnPropertyChanged(nameof(IsResolvedPathVisible)); ScheduleCheck(); }
    }
    private string Mode => SelectedOption?.Value as string ?? "system";
    public bool IsCustom => Mode == "custom";
    public bool IsResolvedPathVisible => !IsCustom;
    public string CustomPath { get => _customPath; set { if (SetProperty(ref _customPath, value) && !_loading) ScheduleCheck(); } }
    public string ResolvedPath { get => _resolvedPath; private set => SetProperty(ref _resolvedPath, value); }
    public string StatusText { get => _status; private set => SetProperty(ref _status, value); }
    public StatusTone StatusTone { get => _tone; private set => SetProperty(ref _tone, value); }
    public bool CanEdit => !_locked && !_saving;
    public bool IsDirty => Draft != _baseline;
    public ICommand CheckCommand => _checkCommand;
    public ICommand SaveCommand => _saveCommand;
    private ShellConfiguration Draft => new(Mode, IsCustom ? CustomPath.Trim() : string.Empty);

    internal void SetLocked(bool locked) { _locked = locked; RefreshCommands(); }
    internal void Apply(ShellCheckResult result, bool managed, bool force = false)
    {
        if (_disposed) return;
        var keepDraft = IsDirty && !force;
        var draft = Draft;
        _baseline = result.Configuration;
        SavedConfigurationChanged?.Invoke();
        _loading = true;
        RebuildOptions(managed);
        var configuration = keepDraft ? draft : result.Configuration;
        _selected = Options.FirstOrDefault(option => Equals(option.Value, configuration.Mode));
        _customPath = configuration.Mode == "custom" ? configuration.Path : _customPath;
        _loading = false;
        OnPropertyChanged(nameof(SelectedOption)); OnPropertyChanged(nameof(CustomPath));
        OnPropertyChanged(nameof(IsCustom)); OnPropertyChanged(nameof(IsResolvedPathVisible));
        if (!keepDraft) { ++_revision; _debounce?.Cancel(); _checking = false; _validated = result.State == "available" ? Draft : null; Present(result); }
        RefreshCommands();
    }
    private void RebuildOptions(bool managed)
    {
        var options = new List<SCComboBoxItem> { new(L("LocSystemDefaultShell", "System default"), "system") };
        if (managed) options.Add(new("PowerShell 7", "managed"));
        options.Add(new(L("LocCustomShellPath", "Custom path…"), "custom")); Options = options;
        _selected ??= Options[0];
    }
    private void ScheduleCheck()
    {
        _validated = null; _debounce?.Cancel();
        _ = CheckAsync(true);
    }
    private async Task CheckAsync(bool debounce)
    {
        var revision = ++_revision;
        _debounce?.Cancel(); _debounce?.Dispose(); _debounce = new();
        var token = _debounce.Token;
        var draft = Draft;
        _validated = null;
        if (draft.Mode == "custom" && string.IsNullOrWhiteSpace(draft.Path))
        {
            _checking = false; StatusText = L("LocChooseShellExecutable", "Choose a shell executable."); StatusTone = StatusTone.Neutral; RefreshCommands(); return;
        }
        _checking = true; StatusText = L("LocCheckingShell", "Checking shell…"); StatusTone = StatusTone.Neutral; RefreshCommands();
        try
        {
            if (debounce) await Task.Delay(400, token);
            ShellCheckResult result;
            await _probeGate.WaitAsync(token);
            try
            {
                if (_disposed || revision != _revision) return;
                result = await _check(new(_target, draft));
            }
            finally { _probeGate.Release(); }
            if (_disposed || revision != _revision) return;
            _validated = result.State == "available" ? draft : null;
            Present(result);
        }
        catch (OperationCanceledException) { }
        catch (Exception exception) { if (!_disposed && revision == _revision) { StatusText = exception.Message; StatusTone = StatusTone.Danger; } }
        finally { if (!_disposed && revision == _revision) { _checking = false; RefreshCommands(); } }
    }
    private void Present(ShellCheckResult result)
    {
        ResolvedPath = result.Path;
        StatusText = result.State == "available" ? $"{result.Message} · {L("LocShellAvailable", "Available")}" : result.Message;
        StatusTone = result.State == "available" ? StatusTone.Success : StatusTone.Danger;
    }
    private async Task SaveAsync()
    {
        var draft = Draft;
        if (_validated != draft || !IsDirty || !CanEdit) return;
        _saving = true; RefreshCommands();
        try
        {
            var result = await _save(new(_target, draft));
            if (_disposed) return;
            if (result.State != "available") { _validated = null; Present(result); return; }
            _baseline = result.Configuration; _validated = draft; SavedConfigurationChanged?.Invoke();
            StatusText = $"{result.Message} · {L("LocShellSaved", "Saved")}"; StatusTone = StatusTone.Success;
        }
        catch (Exception exception) { if (!_disposed) { StatusText = exception.Message; StatusTone = StatusTone.Danger; } }
        finally { _saving = false; if (!_disposed) RefreshCommands(); }
    }
    private void RefreshCommands() { OnPropertyChanged(nameof(CanEdit)); _checkCommand.RaiseCanExecuteChanged(); _saveCommand.RaiseCanExecuteChanged(); }
    internal void RefreshLocalizedText()
    {
        var mode = Mode; _loading = true;
        RebuildOptions(Options.Any(o => Equals(o.Value, "managed")));
        _selected = Options.FirstOrDefault(o => Equals(o.Value, mode)); _loading = false;
        OnPropertyChanged(nameof(SelectedOption)); OnPropertyChanged(nameof(Title)); OnPropertyChanged(nameof(Hint)); OnPropertyChanged(nameof(PathLabel));
    }
    public void Dispose() { _disposed = true; ++_revision; _debounce?.Cancel(); _debounce?.Dispose(); }
}

public sealed class ShellSettingsViewModel : ObservableObject, IPolledPage, IDisposable
{
    private readonly IViewModelHost _host;
    private readonly AsyncRelayCommand _install;
    private readonly AsyncRelayCommand _cancel;
    private readonly AsyncRelayCommand _useBoth;
    private readonly AsyncRelayCommand _useExisting;
    private ShellCheckResult? _existing;
    private bool _loaded;
    private bool _discovering = true;
    private bool _disposed;
    private bool _busy;
    private bool _selectingExisting;
    private bool _installed;
    private bool _useForBoth = true;
    private string _directory = string.Empty;
    private string _installedPath = string.Empty;
    private string _installedVersion = "PowerShell 7";
    private string _architecture = string.Empty;
    private string _message = string.Empty;
    private StatusTone _tone;
    private string _stage = "idle";
    private int _progress;
    private bool _lastRunning;
    private string _platform = OperatingSystem.IsWindows() ? "windows" : OperatingSystem.IsMacOS() ? "macos" : "linux";

    internal ShellSettingsViewModel(IViewModelHost host)
    {
        _host = host;
        Agent = new("agent", request => Setup.CheckAsync(request), request => Setup.SaveAsync(request));
        Terminal = new("terminal", request => Setup.CheckAsync(request), request => Setup.SaveAsync(request));
        _install = new AsyncRelayCommand(InstallAsync, () => CanInstall && IsWindows);
        _cancel = new AsyncRelayCommand(CancelAsync, () => _busy);
        _useBoth = new AsyncRelayCommand(UseBothAsync, () => !_busy && !_selectingExisting && Installed && !BothManaged);
        _useExisting = new AsyncRelayCommand(UseExistingAsync, () => !_busy && !_selectingExisting && HasExisting && !BothExisting);
        Agent.SavedConfigurationChanged += RefreshCommands; Terminal.SavedConfigurationChanged += RefreshCommands;
    }
    private ShellSetupService Setup => ShellSetupService.Current;
    public ShellEditorViewModel Agent { get; }
    public ShellEditorViewModel Terminal { get; }
    public bool IsWindows => _platform == "windows";
    public bool IsInstalling { get => _busy; private set { if (SetProperty(ref _busy, value)) { OnPropertyChanged(nameof(CanChooseInstallTargets)); OnPropertyChanged(nameof(CanInstall)); OnPropertyChanged(nameof(ShowInstallChoices)); RefreshCommands(); } } }
    public bool CanChooseInstallTargets => !IsInstalling;
    public bool IsDiscovering { get => _discovering; private set { if (SetProperty(ref _discovering, value)) { OnPropertyChanged(nameof(CanInstall)); OnPropertyChanged(nameof(ShowInstallChoices)); RefreshCommands(); } } }
    public bool CanInstall => _loaded && !IsDiscovering && !IsInstalling && !_selectingExisting && !Installed && !HasExisting;
    public bool ShowInstallChoices => _loaded && !IsDiscovering && !Installed && !HasExisting;
    public bool HasExisting => PowerShellDiscovery.IsPowerShell7(_existing);
    public string ExistingVersion => _existing?.Shell?.Version ?? string.Empty;
    public string ExistingPath => _existing?.Shell?.Path ?? string.Empty;
    private bool BothExisting => HasExisting && new[] { Agent.SavedConfiguration, Terminal.SavedConfiguration }
        .All(configuration => configuration.Mode == "custom" && string.Equals(configuration.Path, ExistingPath, StringComparison.OrdinalIgnoreCase));
    public string UseExistingLabel => BothExisting ? L("LocShellBothAlreadySelected", "Selected for both shells") : L("LocShellUseExisting", "Use existing installation for both shells");
    public bool Installed { get => _installed; private set { if (SetProperty(ref _installed, value)) { OnPropertyChanged(nameof(CanInstall)); OnPropertyChanged(nameof(ShowInstallChoices)); RefreshCommands(); } } }
    public bool UseForBoth { get => _useForBoth; set => SetProperty(ref _useForBoth, value); }
    public string InstallDirectory { get => _directory; private set => SetProperty(ref _directory, value); }
    public string InstalledSummary => $"{_installedVersion} · {L("LocShellInstalled", "Installed")}";
    public string InstalledPath { get => _installedPath; private set => SetProperty(ref _installedPath, value); }
    public string Architecture { get => _architecture; private set => SetProperty(ref _architecture, value); }
    public string Message { get => _message; private set => SetProperty(ref _message, value); }
    public StatusTone MessageTone { get => _tone; private set => SetProperty(ref _tone, value); }
    public string StageText => _stage switch { "downloading" => L("LocShellDownloading", "Downloading PowerShell 7…"), "verifying" => L("LocShellVerifying", "Verifying download…"), "extracting" => L("LocShellExtracting", "Installing PowerShell 7…"), "checking" => L("LocShellCheckingInstalled", "Checking PowerShell 7…"), _ => string.Empty };
    public int Progress { get => _progress; private set => SetProperty(ref _progress, value); }
    private bool BothManaged => Agent.SavedConfiguration.Mode == "managed" && Terminal.SavedConfiguration.Mode == "managed";
    public string UseBothLabel => BothManaged ? L("LocShellBothAlreadySelected", "Selected for both shells") : L("LocShellUseBoth", "Use for both shells");
    public string InstallLabel => _stage == "failed" ? L("LocShellRetryInstall", "Retry download and install") : L("LocShellDownloadInstall", "Download and install");
    public ICommand InstallCommand => _install;
    public ICommand CancelCommand => _cancel;
    public ICommand UseBothCommand => _useBoth;
    public ICommand UseExistingCommand => _useExisting;
    public async Task ActivateAsync() { if (!_loaded) await LoadAsync(); await PollAsync(); }
    public async Task PollAsync()
    {
        if (_disposed || !await _host.EnsureSdkReadyAsync()) return;
        try
        {
            var state = Setup.Status;
            if (_disposed) return;
            ApplyStatus(state);
            if (_lastRunning && !state.Running) await LoadAsync(state.Stage == "installed" && state.UseForBoth);
            _lastRunning = state.Running;
        }
        catch (Exception exception) { if (!_disposed) { Message = exception.Message; MessageTone = StatusTone.Danger; } }
    }
    public void Deactivate() { }
    private async Task LoadAsync(bool force = false)
    {
        if (!await _host.EnsureSdkReadyAsync()) return;
        IsDiscovering = true;
        try
        {
            var result = await Setup.SettingsAsync();
            if (_disposed) return;
            ApplySettings(result, force);
        }
        catch (Exception exception) { if (!_disposed) { Message = exception.Message; MessageTone = StatusTone.Danger; } }
        finally { if (!_disposed) IsDiscovering = false; }
    }
    internal void ApplySettings(ShellSettingsResult result, bool force = false)
    {
        _loaded = true; _platform = result.Platform; OnPropertyChanged(nameof(IsWindows));
        IsDiscovering = false;
        _existing = result.Existing;
        OnPropertyChanged(nameof(HasExisting)); OnPropertyChanged(nameof(ExistingVersion)); OnPropertyChanged(nameof(ExistingPath));
        OnPropertyChanged(nameof(CanInstall)); OnPropertyChanged(nameof(ShowInstallChoices));
        Installed = result.Managed?.State == "available";
        InstallDirectory = result.InstallDirectory ?? string.Empty;
        InstalledPath = result.Managed?.Path ?? string.Empty;
        _installedVersion = result.Managed?.Shell?.Version ?? "PowerShell 7"; OnPropertyChanged(nameof(InstalledSummary));
        Architecture = result.Architecture == "arm64" ? "Windows ARM64" : "Windows x64";
        Agent.Apply(result.Agent, Installed, force); Terminal.Apply(result.Terminal, Installed, force);
        RefreshCommands();
    }

    private void ApplyStatus(ShellInstallationStatus state)
    {
        IsInstalling = state.Running; _stage = state.Stage; Progress = state.Progress; OnPropertyChanged(nameof(StageText)); OnPropertyChanged(nameof(InstallLabel));
        Message = state.Message; MessageTone = state.ErrorCode is null ? StatusTone.Success : state.Stage == "cancelled" ? StatusTone.Neutral : StatusTone.Danger;
        Agent.SetLocked(_selectingExisting || state.Running && state.UseForBoth); Terminal.SetLocked(_selectingExisting || state.Running && state.UseForBoth);
    }
    private async Task InstallAsync()
    {
        if (!await _host.EnsureSdkReadyAsync()) return;
        try { ApplyStatus(Setup.StartInstallation(UseForBoth)); _lastRunning = true; }
        catch (Exception exception) { Message = exception.Message; MessageTone = StatusTone.Danger; }
    }
    private async Task CancelAsync() { try { ApplyStatus(await Setup.CancelInstallationAsync()); } catch (Exception exception) { Message = exception.Message; MessageTone = StatusTone.Danger; } }
    private async Task UseBothAsync()
    {
        try
        {
            // Desktop verifies the runtime; the SDK commits both preferences atomically.
            var request = new ShellConfiguration("managed");
            var result = await Setup.SaveAsync(new("both", request));
            if (result.Shell is null) throw new SdkException(result.ErrorCode ?? "shell_unavailable", result.Message);
            await LoadAsync(true);
            Message = L("LocShellBothSelected", "PowerShell 7 selected for both shells."); MessageTone = StatusTone.Success;
        }
        catch (Exception exception) { Message = exception.Message; MessageTone = StatusTone.Danger; }
    }
    private async Task UseExistingAsync()
    {
        if (!HasExisting || !await _host.EnsureSdkReadyAsync()) return;
        _selectingExisting = true;
        Agent.SetLocked(true); Terminal.SetLocked(true); RefreshCommands(); OnPropertyChanged(nameof(CanInstall));
        try
        {
            // Save probes the exact discovered path again before the atomic commit.
            var result = await Setup.SaveAsync(new("both", new("custom", ExistingPath)), requirePowerShell7: true);
            if (!PowerShellDiscovery.IsPowerShell7(result)) throw new SdkException(result.ErrorCode ?? "shell_unavailable", result.Message);
            await LoadAsync(true);
            Message = L("LocShellBothSelected", "PowerShell 7 selected for both shells."); MessageTone = StatusTone.Success;
        }
        catch (Exception exception)
        {
            await LoadAsync();
            Message = exception.Message; MessageTone = StatusTone.Danger;
        }
        finally
        {
            _selectingExisting = false;
            Agent.SetLocked(_busy && UseForBoth); Terminal.SetLocked(_busy && UseForBoth);
            RefreshCommands(); OnPropertyChanged(nameof(CanInstall));
        }
    }
    private void RefreshCommands() { OnPropertyChanged(nameof(UseBothLabel)); OnPropertyChanged(nameof(UseExistingLabel)); _install.RaiseCanExecuteChanged(); _cancel.RaiseCanExecuteChanged(); _useBoth.RaiseCanExecuteChanged(); _useExisting.RaiseCanExecuteChanged(); }
    internal void RefreshLocalizedText() { Agent.RefreshLocalizedText(); Terminal.RefreshLocalizedText(); OnPropertyChanged(nameof(StageText)); OnPropertyChanged(nameof(InstallLabel)); OnPropertyChanged(nameof(UseBothLabel)); OnPropertyChanged(nameof(UseExistingLabel)); OnPropertyChanged(nameof(InstalledSummary)); }
    public void Dispose() { _disposed = true; Agent.Dispose(); Terminal.Dispose(); }
}

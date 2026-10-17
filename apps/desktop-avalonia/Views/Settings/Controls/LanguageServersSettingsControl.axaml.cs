using Avalonia.Controls;
using Avalonia.Interactivity;
using SunCode.Desktop.Controls;
using SunCode.Desktop.Infrastructure;
using SunCode.Desktop.Models;
using SunCode.Desktop.ViewModels;
using SunCode.Sdk.Models;

namespace SunCode.Desktop.Views.Settings.Controls;

public sealed partial class LanguageServersSettingsControl : UserControl
{
    private LanguageServerItem? _selected;
    private string _managedDataPath = string.Empty;
    private ServerDraft? _baseline;
    private bool _populating;
    private readonly SCComboBoxItem _managedDataPathOption;
    private readonly SCComboBoxItem _customDataPathOption;
    private readonly SCComboBoxItem _workspacePreferredOption;
    private readonly SCComboBoxItem _bundledTypeScriptOption;
    private readonly SCComboBoxItem _customTypeScriptOption;
    private readonly SCComboBoxItem _openFilesOnlyOption;
    private readonly SCComboBoxItem _workspaceDiagnosticOption;
    private readonly SCComboBoxItem _recommendedCheckingOption;
    private readonly SCComboBoxItem _standardCheckingOption;
    private readonly SCComboBoxItem _basicCheckingOption;
    private readonly SCComboBoxItem _strictCheckingOption;
    private readonly SCComboBoxItem _offCheckingOption;
    private readonly SCComboBoxItem _allCheckingOption;
    private readonly SCComboBoxItem _automaticConfigOption;
    private readonly SCComboBoxItem _customConfigOption;
    private readonly SCComboBoxItem _defaultCargoFeaturesOption;
    private readonly SCComboBoxItem _allCargoFeaturesOption;
    private readonly SCComboBoxItem _customCargoFeaturesOption;
    private readonly SCComboBoxItem _cargoCheckOption;
    private readonly SCComboBoxItem _clippyOption;
    private readonly SCComboBoxItem _hostRustTargetOption;
    private readonly SCComboBoxItem _customRustTargetOption;

    public LanguageServersSettingsControl()
    {
        InitializeComponent();
        _managedDataPathOption = new(L("LocManagedBySunCode", "Managed by SunCode"), "managed");
        _customDataPathOption = new(L("LocCustom", "Custom"), "custom");
        DataPathModeSelector.ItemsSource = [_managedDataPathOption, _customDataPathOption];
        DataPathModeSelector.SelectedItem = _managedDataPathOption;
        _workspacePreferredOption = new(L("LocWorkspacePreferred", "Workspace preferred"), "workspace-preferred");
        _bundledTypeScriptOption = new(L("LocBundledWithVtsls", "Bundled with vtsls"), "bundled");
        _customTypeScriptOption = new(L("LocCustomSdk", "Custom SDK"), "custom");
        TypeScriptSdkModeSelector.ItemsSource =
            [_workspacePreferredOption, _bundledTypeScriptOption, _customTypeScriptOption];
        TypeScriptSdkModeSelector.SelectedItem = _workspacePreferredOption;
        _openFilesOnlyOption = new(L("LocOpenFilesOnly", "Open files only"), "openFilesOnly");
        _workspaceDiagnosticOption = new(L("LocEntireWorkspace", "Entire workspace"), "workspace");
        DiagnosticModeSelector.ItemsSource = [_openFilesOnlyOption, _workspaceDiagnosticOption];
        DiagnosticModeSelector.SelectedItem = _openFilesOnlyOption;
        _recommendedCheckingOption = new(L("LocRecommended", "Recommended"), "recommended");
        _standardCheckingOption = new(L("LocStandard", "Standard"), "standard");
        _basicCheckingOption = new(L("LocBasic", "Basic"), "basic");
        _strictCheckingOption = new(L("LocStrict", "Strict"), "strict");
        _offCheckingOption = new(L("LocOff", "Off"), "off");
        _allCheckingOption = new(L("LocAllRules", "All rules"), "all");
        TypeCheckingModeSelector.ItemsSource =
            [_recommendedCheckingOption, _standardCheckingOption, _basicCheckingOption,
             _strictCheckingOption, _offCheckingOption, _allCheckingOption];
        TypeCheckingModeSelector.SelectedItem = _recommendedCheckingOption;
        _automaticConfigOption = new(L("LocAutomaticFromProject", "Automatic from project"), "automatic");
        _customConfigOption = new(L("LocCustomLocation", "Custom location"), "custom");
        ConfigPathModeSelector.ItemsSource = [_automaticConfigOption, _customConfigOption];
        ConfigPathModeSelector.SelectedItem = _automaticConfigOption;
        _defaultCargoFeaturesOption = new(L("LocDefaultFeatures", "Default features"), "default");
        _allCargoFeaturesOption = new(L("LocAllFeatures", "All features"), "all");
        _customCargoFeaturesOption = new(L("LocCustomFeatures", "Custom features"), "custom");
        CargoFeatureModeSelector.ItemsSource =
            [_defaultCargoFeaturesOption, _allCargoFeaturesOption, _customCargoFeaturesOption];
        CargoFeatureModeSelector.SelectedItem = _defaultCargoFeaturesOption;
        _cargoCheckOption = new(L("LocCargoCheck", "Cargo check"), "check");
        _clippyOption = new(L("LocClippy", "Clippy"), "clippy");
        RustCheckCommandSelector.ItemsSource = [_cargoCheckOption, _clippyOption];
        RustCheckCommandSelector.SelectedItem = _cargoCheckOption;
        _hostRustTargetOption = new(L("LocHostToolchainTarget", "Host toolchain target"), "host");
        _customRustTargetOption = new(L("LocCustomTarget", "Custom target"), "custom");
        RustTargetModeSelector.ItemsSource = [_hostRustTargetOption, _customRustTargetOption];
        RustTargetModeSelector.SelectedItem = _hostRustTargetOption;
        WireDirtyTracking();
    }

    private LanguageServersViewModel? ViewModel => DataContext as LanguageServersViewModel;

    private static string L(string key, string fallback) => LocalizationService.GetString(key, fallback);

    private void ConfigureClicked(object? sender, RoutedEventArgs e)
    {
        if (sender is not Button { Tag: LanguageServerItem item } || !item.CanConfigure) return;
        _selected = item;
        Populate(item);
        CatalogPanel.IsVisible = false;
        DetailPanel.IsVisible = true;
    }

    private void BackClicked(object? sender, RoutedEventArgs e)
    {
        DetailPanel.IsVisible = false;
        CatalogPanel.IsVisible = true;
        _selected = null;
        _baseline = null;
        UpdateDirtyState();
    }

    private async void ToggleClicked(object? sender, RoutedEventArgs e)
    {
        if (sender is not ToggleSwitch { Tag: LanguageServerItem item } toggle || ViewModel is null)
            return;
        toggle.IsEnabled = false;
        var saved = await ViewModel.SetServerEnabledAsync(item, toggle.IsChecked == true);
        if (!saved) toggle.IsChecked = item.Enabled;
        toggle.IsEnabled = item.CanToggle;
    }

    private async void DetectClicked(object? sender, RoutedEventArgs e)
    {
        if (ViewModel is null || _selected is null) return;
        await ViewModel.LoadServersAsync();
        var refreshed = ViewModel.Servers.FirstOrDefault(item => item.ServerId == _selected.ServerId);
        if (refreshed is null) return;
        _selected = refreshed;
        UpdateRuntimePresentation(refreshed);
        UpdateDirtyState();
        if (!string.IsNullOrWhiteSpace(refreshed.DetectedExecutable))
            ExecutableInput.Text = refreshed.DetectedExecutable;
        if (refreshed.IsJdtls && !string.IsNullOrWhiteSpace(refreshed.DetectedJavaHome))
            JavaHomeInput.Text = refreshed.DetectedJavaHome;
        if (refreshed.IsVtsls && !string.IsNullOrWhiteSpace(refreshed.DetectedNodePath))
            NodePathInput.Text = refreshed.DetectedNodePath;
        if (refreshed.IsBasedpyright && !string.IsNullOrWhiteSpace(refreshed.DetectedPythonPath))
            PythonPathInput.Text = refreshed.DetectedPythonPath;
        if (refreshed.IsRustAnalyzer)
            RustToolchainText.Text = RustToolchainLabel(refreshed);
    }

    private async void RetryClicked(object? sender, RoutedEventArgs e)
    {
        if (ViewModel is null || _selected is null) return;
        RetryButton.IsEnabled = false;
        var retried = await ViewModel.RetryServerAsync(_selected);
        if (!retried)
        {
            ValidationText.Text = ViewModel.StatusText;
            RetryButton.IsEnabled = _selected.CanRetry;
            return;
        }
        var refreshed = ViewModel.Servers.FirstOrDefault(item => item.ServerId == _selected.ServerId);
        if (refreshed is null) return;
        _selected = refreshed;
        UpdateRuntimePresentation(refreshed);
        UpdateDirtyState();
    }

    private void ResetClicked(object? sender, RoutedEventArgs e)
    {
        if (_selected is not null) Populate(_selected);
    }

    private async void SaveClicked(object? sender, RoutedEventArgs e)
    {
        if (_selected is null || ViewModel is null || !TryCreateRequest(out var environment, out var startup, out var request))
            return;
        bool saved;
        if (_selected.IsVtsls)
        {
            saved = await ViewModel.SaveVtslsAsync(
                _selected,
                ExecutableInput.Text?.Trim() ?? string.Empty,
                NodePathInput.Text?.Trim() ?? string.Empty,
                TypeScriptSdkMode,
                IsCustomTypeScriptSdk ? NullIfEmpty(TypeScriptSdkPathInput.Text) : null,
                Lines(RootMarkersInput.Text),
                (ulong)(TsServerMemoryInput.Value ?? 3072),
                environment,
                startup,
                request,
                EnabledToggle.IsChecked == true);
        }
        else if (_selected.IsRustAnalyzer)
        {
            saved = await ViewModel.SaveRustAnalyzerAsync(
                _selected,
                ExecutableInput.Text?.Trim() ?? string.Empty,
                Lines(RootMarkersInput.Text),
                CargoFeatureMode,
                IsCustomCargoFeatures ? Lines(CargoFeaturesInput.Text) : [],
                AllTargetsToggle.IsChecked == true,
                BuildScriptsToggle.IsChecked == true,
                ProcMacrosToggle.IsChecked == true,
                CheckOnSaveToggle.IsChecked == true,
                RustCheckCommand,
                RustTargetMode,
                IsCustomRustTarget ? NullIfEmpty(RustTargetInput.Text) : null,
                environment,
                startup,
                request,
                EnabledToggle.IsChecked == true);
        }
        else if (_selected.IsBasedpyright)
        {
            saved = await ViewModel.SaveBasedpyrightAsync(
                _selected,
                ExecutableInput.Text?.Trim() ?? string.Empty,
                NullIfEmpty(PythonPathInput.Text),
                Lines(RootMarkersInput.Text),
                DiagnosticMode,
                TypeCheckingMode,
                ConfigPathMode,
                IsCustomConfigPath ? NullIfEmpty(ConfigPathInput.Text) : null,
                environment,
                startup,
                request,
                EnabledToggle.IsChecked == true);
        }
        else
        {
            saved = await ViewModel.SaveJdtlsAsync(
                _selected,
                ExecutableInput.Text?.Trim() ?? string.Empty,
                NullIfEmpty(JavaHomeInput.Text),
                IsCustomDataPath ? "custom" : "managed",
                IsCustomDataPath ? NullIfEmpty(DataPathInput.Text) : null,
                Lines(RootMarkersInput.Text),
                environment,
                startup,
                request,
                EnabledToggle.IsChecked == true);
        }
        if (!saved)
        {
            ValidationText.Text = ViewModel.StatusText;
            return;
        }
        var refreshed = ViewModel.Servers.FirstOrDefault(item => item.ServerId == _selected.ServerId);
        if (refreshed is not null)
        {
            _selected = refreshed;
            Populate(refreshed);
        }
    }

    private bool TryCreateRequest(
        out LanguageServerEnvironmentChanges? environment,
        out ulong startup,
        out ulong request)
    {
        environment = null;
        startup = 0;
        request = 0;
        ValidationText.Text = string.Empty;
        if (string.IsNullOrWhiteSpace(ExecutableInput.Text))
        {
            ValidationText.Text = $"{_selected?.DisplayName ?? "Language server"} executable is required.";
            return false;
        }
        if (_selected?.IsJdtls == true && IsCustomDataPath && string.IsNullOrWhiteSpace(DataPathInput.Text))
        {
            ValidationText.Text = "A custom data directory is required.";
            return false;
        }
        if (_selected?.IsVtsls == true && string.IsNullOrWhiteSpace(NodePathInput.Text))
        {
            ValidationText.Text = "A Node.js executable is required.";
            return false;
        }
        if (_selected?.IsVtsls == true && IsCustomTypeScriptSdk
            && string.IsNullOrWhiteSpace(TypeScriptSdkPathInput.Text))
        {
            ValidationText.Text = "A custom TypeScript SDK directory is required.";
            return false;
        }
        if (_selected?.IsVtsls == true
            && (TsServerMemoryInput.Value is not { } memoryValue || memoryValue is < 512 or > 16384))
        {
            ValidationText.Text = "tsserver memory must be between 512 and 16384 MiB.";
            return false;
        }
        if (_selected?.IsBasedpyright == true && IsCustomConfigPath
            && string.IsNullOrWhiteSpace(ConfigPathInput.Text))
        {
            ValidationText.Text = "A custom basedpyright configuration location is required.";
            return false;
        }
        if (_selected?.IsRustAnalyzer == true && IsCustomRustTarget
            && string.IsNullOrWhiteSpace(RustTargetInput.Text))
        {
            ValidationText.Text = "A custom Rust target triple is required.";
            return false;
        }
        if (StartupTimeoutInput.Value is not { } startupValue || startupValue is < 1 or > 120
            || RequestTimeoutInput.Value is not { } requestValue || requestValue is < 1 or > 600)
        {
            ValidationText.Text = "Startup timeout must be 1-120 seconds and request timeout must be 1-600 seconds.";
            return false;
        }
        startup = (ulong)startupValue;
        request = (ulong)requestValue;
        if (!TryParseEnvironment(out environment)) return false;
        return true;
    }

    private bool TryParseEnvironment(out LanguageServerEnvironmentChanges? changes)
    {
        var set = new Dictionary<string, string>(StringComparer.Ordinal);
        var remove = new List<string>();
        foreach (var line in Lines(EnvironmentInput.Text))
        {
            if (line.StartsWith('-'))
            {
                var key = line[1..].Trim();
                if (key.Length == 0 || key.Contains('='))
                {
                    ValidationText.Text = "Environment removals must use -NAME.";
                    changes = null;
                    return false;
                }
                remove.Add(key);
                continue;
            }
            var separator = line.IndexOf('=');
            if (separator <= 0 || separator == line.Length - 1)
            {
                ValidationText.Text = "Environment entries must use NAME=value, or -NAME to remove a stored key.";
                changes = null;
                return false;
            }
            set[line[..separator].Trim()] = line[(separator + 1)..];
        }
        changes = set.Count == 0 && remove.Count == 0 ? null : new(set, remove);
        return true;
    }

    private void Populate(LanguageServerItem item)
    {
        _populating = true;
        try
        {
            UpdateRuntimePresentation(item);
            EnabledToggle.IsChecked = item.Enabled;
            MatchedRootMarkerText.Text = string.IsNullOrWhiteSpace(item.MatchedRootMarker)
                ? "No root marker matched"
                : $"Matched  {item.MatchedRootMarker}";
            var isVtsls = item.IsVtsls;
            var isBasedpyright = item.IsBasedpyright;
            var isRustAnalyzer = item.IsRustAnalyzer;
            DetailSubtitle.Text = isRustAnalyzer
                ? L("LocProjectRustAnalysis", "Project configuration for Rust semantic analysis.")
                : isBasedpyright
                ? L("LocProjectPythonAnalysis", "Project configuration for Python semantic analysis.")
                : isVtsls
                    ? L("LocProjectJavaScriptTypeScriptAnalysis", "Project JavaScript and TypeScript analysis")
                    : L("LocProjectJavaAnalysis", "Project Java analysis");
            EnableHint.Text = isRustAnalyzer
                ? L("LocEnableRustAnalyzerHint", "Starts one rust-analyzer process for Rust files in this project.")
                : isBasedpyright
                ? L("LocEnableBasedpyrightHint", "Starts one basedpyright process for Python files in this project.")
                : isVtsls
                    ? L("LocEnableVtslsHint", "Starts one vtsls process for JavaScript and TypeScript files in this project.")
                    : L("LocEnableJdtlsHint", "Starts one jdtls process and keeps a separate index for this project.");
            ExecutableLabel.Text = isRustAnalyzer
                ? L("LocRustAnalyzerExecutable", "rust-analyzer executable")
                : isBasedpyright
                ? L("LocBasedpyrightExecutable", "basedpyright executable")
                : isVtsls
                    ? L("LocVtslsExecutable", "vtsls executable")
                    : L("LocJdtlsExecutable", "jdtls executable");
            ExecutableHint.Text = isRustAnalyzer
                ? L("LocRustAnalyzerExecutableHint", "The rust-analyzer executable SunCode starts over standard input and output.")
                : isBasedpyright
                ? L("LocBasedpyrightExecutableHint", "The basedpyright-langserver executable SunCode starts with --stdio.")
                : isVtsls
                    ? L("LocVtslsExecutableHint", "The @vtsls/language-server executable SunCode starts with --stdio.")
                    : L("LocJdtlsExecutableHint", "Executable used for this project.");
            ExecutableInput.DialogTitle = isRustAnalyzer
                ? "Choose rust-analyzer executable"
                : isBasedpyright
                ? "Choose basedpyright executable"
                : isVtsls ? "Choose vtsls executable" : "Choose jdtls executable";
            RootMarkersHint.Text = isRustAnalyzer
                ? L("LocRustRootMarkersHint", "The first match identifies a Rust project.")
                : isBasedpyright
                ? L("LocPythonRootMarkersHint", "The first match identifies a Python project.")
                : isVtsls
                    ? L("LocVtslsRootMarkersHint", "The first match identifies a JavaScript or TypeScript project.")
                    : L("LocJdtlsRootMarkersHint", "The first match identifies a Java project.");
            ProjectConfigurationLabel.Text = isBasedpyright || isRustAnalyzer
                ? L("LocProjectMatchingCaps", "PROJECT MATCHING")
                : L("LocProjectIndexCaps", "PROJECT INDEX");
            JavaRuntimeRow.IsVisible = item.IsJdtls;
            NodeRuntimeRow.IsVisible = isVtsls;
            RustToolchainRow.IsVisible = isRustAnalyzer;
            PythonEnvironmentDivider.IsVisible = isBasedpyright;
            PythonEnvironmentSection.IsVisible = isBasedpyright;
            PythonAnalysisDivider.IsVisible = isBasedpyright;
            PythonAnalysisSection.IsVisible = isBasedpyright;
            RustCargoDivider.IsVisible = isRustAnalyzer;
            RustCargoSection.IsVisible = isRustAnalyzer;
            RustDiagnosticsDivider.IsVisible = isRustAnalyzer;
            RustDiagnosticsSection.IsVisible = isRustAnalyzer;
            DataPathModeRow.IsVisible = item.IsJdtls;
            DataPathRow.IsVisible = item.IsJdtls;
            TypeScriptSdkModeRow.IsVisible = isVtsls;
            TsServerMemoryRow.IsVisible = isVtsls;
            ExecutableInput.Text = isRustAnalyzer
                ? item.RustAnalyzerExecutable
                : isBasedpyright
                ? item.BasedpyrightExecutable
                : isVtsls ? item.VtslsExecutable : item.Command;
            JavaHomeInput.Text = item.JavaHome ?? string.Empty;
            NodePathInput.Text = item.NodePath;
            NodeRuntimeHint.Text = string.IsNullOrWhiteSpace(item.DetectedNodeVersion)
                ? L("LocNodeRuntimeHint", "vtsls requires Node.js 18 or newer.")
                : $"Node.js {item.DetectedNodeVersion} detected. vtsls requires version 18 or newer.";
            RustToolchainText.Text = RustToolchainLabel(item);
            PythonPathInput.Text = item.PythonPath ?? string.Empty;
            PythonInterpreterHint.Text = string.IsNullOrWhiteSpace(item.DetectedPythonVersion)
                ? L("LocPythonInterpreterHint", "Used to resolve installed packages and infer the target Python version.")
                : $"{L("LocPythonInterpreterHint", "Used to resolve installed packages and infer the target Python version.")} Detected Python {item.DetectedPythonVersion}.";
            _managedDataPath = item.DataPathMode == "managed" ? item.DataPath : string.Empty;
            DataPathModeSelector.SelectedItem = item.DataPathMode == "custom"
                ? _customDataPathOption
                : _managedDataPathOption;
            DataPathInput.Text = item.DataPath;
            RootMarkersInput.Text = string.Join(Environment.NewLine, item.RootMarkers);
            TypeScriptSdkModeSelector.SelectedItem = item.TypeScriptSdkMode switch
            {
                "bundled" => _bundledTypeScriptOption,
                "custom" => _customTypeScriptOption,
                _ => _workspacePreferredOption
            };
            TypeScriptSdkPathInput.Text = item.TypeScriptSdkPath ?? string.Empty;
            TsServerMemoryInput.Value = item.MaxTsServerMemory;
            DiagnosticModeSelector.SelectedItem = item.DiagnosticMode == "workspace"
                ? _workspaceDiagnosticOption
                : _openFilesOnlyOption;
            TypeCheckingModeSelector.SelectedItem = item.TypeCheckingMode switch
            {
                "standard" => _standardCheckingOption,
                "basic" => _basicCheckingOption,
                "strict" => _strictCheckingOption,
                "off" => _offCheckingOption,
                "all" => _allCheckingOption,
                _ => _recommendedCheckingOption
            };
            ConfigPathModeSelector.SelectedItem = item.ConfigPathMode == "custom"
                ? _customConfigOption
                : _automaticConfigOption;
            ConfigPathInput.Text = item.ConfigPath ?? string.Empty;
            CargoFeatureModeSelector.SelectedItem = item.CargoFeatureMode switch
            {
                "all" => _allCargoFeaturesOption,
                "custom" => _customCargoFeaturesOption,
                _ => _defaultCargoFeaturesOption
            };
            CargoFeaturesInput.Text = string.Join(Environment.NewLine, item.CargoFeatures);
            AllTargetsToggle.IsChecked = item.AllCargoTargets;
            BuildScriptsToggle.IsChecked = item.BuildScripts;
            ProcMacrosToggle.IsChecked = item.ProcMacros;
            CheckOnSaveToggle.IsChecked = item.CheckOnSave;
            RustCheckCommandSelector.SelectedItem = item.RustCheckCommand == "clippy"
                ? _clippyOption
                : _cargoCheckOption;
            RustTargetModeSelector.SelectedItem = item.RustTargetMode == "custom"
                ? _customRustTargetOption
                : _hostRustTargetOption;
            RustTargetInput.Text = item.RustTarget ?? string.Empty;
            EnvironmentInput.Text = string.Empty;
            EnvironmentInput.PlaceholderText = item.EnvironmentKeys.Count > 0
                ? $"Stored keys: {string.Join(", ", item.EnvironmentKeys)}"
                : "NAME=value";
            StartupTimeoutInput.Value = item.StartupTimeoutSeconds;
            RequestTimeoutInput.Value = item.RequestTimeoutSeconds;
            ValidationText.Text = string.Empty;
            UpdateDataPathMode();
            UpdateTypeScriptSdkMode();
            UpdateBasedpyrightOptions();
            UpdateRustOptions();
        }
        finally
        {
            _populating = false;
        }
        _baseline = CaptureDraft();
        UpdateDirtyState();
    }

    private void UpdateRuntimePresentation(LanguageServerItem item)
    {
        DetailTitle.Text = item.DisplayName;
        FailureTitle.Text = item.IsRustAnalyzer
            ? L("LocRustAnalyzerCouldNotStart", "rust-analyzer could not start")
            : item.IsBasedpyright
            ? L("LocBasedpyrightCouldNotStart", "basedpyright could not start")
            : item.IsVtsls
                ? L("LocVtslsCouldNotStart", "vtsls could not start")
                : L("LocJdtlsCouldNotStart", "jdtls could not start");
        DetailStatus.Text = item.StatusLabel;
        DetailStatusText.Text = item.StatusDetail;
        DetailStatus.Classes.Set("tone-danger", item.IsFailed);
        DetailStatusText.Classes.Set("tone-danger", item.IsFailed);
        FailureBanner.IsVisible = item.IsFailed;
        DetailErrorText.Text = item.StatusDetail;
        RetryButton.IsEnabled = item.CanRetry;
    }

    private void DataPathModeChanged(object? sender, SelectionChangedEventArgs e)
    {
        UpdateDataPathMode();
        UpdateDirtyState();
    }

    private void TypeScriptSdkModeChanged(object? sender, SelectionChangedEventArgs e)
    {
        UpdateTypeScriptSdkMode();
        UpdateDirtyState();
    }

    private void BasedpyrightOptionChanged(object? sender, SelectionChangedEventArgs e)
    {
        UpdateBasedpyrightOptions();
        UpdateDirtyState();
    }

    private void ConfigPathModeChanged(object? sender, SelectionChangedEventArgs e)
    {
        UpdateBasedpyrightOptions();
        UpdateDirtyState();
    }

    private void RustOptionChanged(object? sender, SelectionChangedEventArgs e)
    {
        UpdateRustOptions();
        UpdateDirtyState();
    }

    private bool IsCustomDataPath =>
        string.Equals(DataPathModeSelector.SelectedItem?.Value as string, "custom", StringComparison.Ordinal);

    private string TypeScriptSdkMode =>
        TypeScriptSdkModeSelector.SelectedItem?.Value as string ?? "workspace-preferred";

    private bool IsCustomTypeScriptSdk => TypeScriptSdkMode == "custom";

    private string DiagnosticMode =>
        DiagnosticModeSelector.SelectedItem?.Value as string ?? "openFilesOnly";

    private string TypeCheckingMode =>
        TypeCheckingModeSelector.SelectedItem?.Value as string ?? "recommended";

    private string ConfigPathMode =>
        ConfigPathModeSelector.SelectedItem?.Value as string ?? "automatic";

    private bool IsCustomConfigPath => ConfigPathMode == "custom";

    private string CargoFeatureMode =>
        CargoFeatureModeSelector.SelectedItem?.Value as string ?? "default";

    private bool IsCustomCargoFeatures => CargoFeatureMode == "custom";

    private string RustCheckCommand =>
        RustCheckCommandSelector.SelectedItem?.Value as string ?? "check";

    private string RustTargetMode =>
        RustTargetModeSelector.SelectedItem?.Value as string ?? "host";

    private bool IsCustomRustTarget => RustTargetMode == "custom";

    private void UpdateDataPathMode()
    {
        var custom = IsCustomDataPath;
        DataPathInput.IsEnabled = custom;
        if (!custom && _managedDataPath.Length > 0) DataPathInput.Text = _managedDataPath;
    }

    private void UpdateTypeScriptSdkMode()
    {
        TypeScriptSdkPathRow.IsVisible = _selected?.IsVtsls == true && IsCustomTypeScriptSdk;
        TypeScriptSdkPathInput.IsEnabled = IsCustomTypeScriptSdk;
    }

    private void UpdateBasedpyrightOptions()
    {
        CustomConfigPathRow.IsVisible = _selected?.IsBasedpyright == true && IsCustomConfigPath;
        ConfigPathInput.IsEnabled = IsCustomConfigPath;
        DiagnosticModeHint.Text = DiagnosticMode == "workspace"
            ? L("LocEntireWorkspaceHint", "Analyzes every Python file selected by the project configuration.")
            : L("LocOpenFilesOnlyHint", "Analyzes files after SunCode opens them for semantic work.");
        ConfigPathModeHint.Text = IsCustomConfigPath
            ? L("LocCustomConfigHint", "Uses a specific configuration file or directory for this project.")
            : L("LocAutomaticConfigHint", "Finds pyproject.toml or pyrightconfig.json from the project root.");
    }

    private void UpdateRustOptions()
    {
        CustomCargoFeaturesRow.IsVisible = _selected?.IsRustAnalyzer == true && IsCustomCargoFeatures;
        CargoFeaturesInput.IsEnabled = IsCustomCargoFeatures;
        CargoFeaturesHint.Text = CargoFeatureMode switch
        {
            "all" => L("LocAllCargoFeaturesHint", "Activates every Cargo feature while rust-analyzer loads the project."),
            "custom" => L("LocCustomCargoFeaturesHint", "Activates only the listed Cargo features in addition to default features."),
            _ => L("LocDefaultCargoFeaturesHint", "Uses the default Cargo feature set declared by the project.")
        };
        CustomRustTargetRow.IsVisible = _selected?.IsRustAnalyzer == true && IsCustomRustTarget;
        RustTargetInput.IsEnabled = IsCustomRustTarget;
        RustTargetModeHint.Text = IsCustomRustTarget
            ? L("LocCustomRustTargetHint", "Analyzes the project for a specific Rust target triple.")
            : L("LocHostRustTargetHint", "Uses the host target reported by the selected Rust toolchain.");
    }

    private void WireDirtyTracking()
    {
        EnabledToggle.Click += (_, _) => UpdateDirtyState();
        ExecutableInput.TextChanged += (_, _) => UpdateDirtyState();
        JavaHomeInput.TextChanged += (_, _) => UpdateDirtyState();
        NodePathInput.TextChanged += (_, _) => UpdateDirtyState();
        PythonPathInput.TextChanged += (_, _) => UpdateDirtyState();
        DataPathInput.TextChanged += (_, _) => UpdateDirtyState();
        RootMarkersInput.TextChanged += (_, _) => UpdateDirtyState();
        TypeScriptSdkPathInput.TextChanged += (_, _) => UpdateDirtyState();
        TsServerMemoryInput.ValueChanged += (_, _) => UpdateDirtyState();
        ConfigPathInput.TextChanged += (_, _) => UpdateDirtyState();
        CargoFeaturesInput.TextChanged += (_, _) => UpdateDirtyState();
        AllTargetsToggle.Click += (_, _) => UpdateDirtyState();
        BuildScriptsToggle.Click += (_, _) => UpdateDirtyState();
        ProcMacrosToggle.Click += (_, _) => UpdateDirtyState();
        CheckOnSaveToggle.Click += (_, _) => UpdateDirtyState();
        RustTargetInput.TextChanged += (_, _) => UpdateDirtyState();
        EnvironmentInput.TextChanged += (_, _) => UpdateDirtyState();
        StartupTimeoutInput.ValueChanged += (_, _) => UpdateDirtyState();
        RequestTimeoutInput.ValueChanged += (_, _) => UpdateDirtyState();
    }

    private void UpdateDirtyState()
    {
        if (_populating) return;
        var isDirty = _baseline is not null && CaptureDraft() != _baseline;
        SaveButton.IsEnabled = isDirty;
        ResetButton.IsEnabled = isDirty;
    }

    private ServerDraft CaptureDraft() => new(
        _selected?.Kind ?? string.Empty,
        EnabledToggle.IsChecked == true,
        ExecutableInput.Text?.Trim() ?? string.Empty,
        _selected?.IsJdtls == true ? NullIfEmpty(JavaHomeInput.Text) : null,
        _selected?.IsVtsls == true ? NullIfEmpty(NodePathInput.Text) : null,
        _selected?.IsBasedpyright == true ? NullIfEmpty(PythonPathInput.Text) : null,
        _selected?.IsJdtls == true && IsCustomDataPath ? "custom" : "managed",
        _selected?.IsJdtls == true && IsCustomDataPath ? NullIfEmpty(DataPathInput.Text) : null,
        _selected?.IsVtsls == true ? TypeScriptSdkMode : null,
        _selected?.IsVtsls == true && IsCustomTypeScriptSdk ? NullIfEmpty(TypeScriptSdkPathInput.Text) : null,
        _selected?.IsVtsls == true ? TsServerMemoryInput.Value : null,
        _selected?.IsBasedpyright == true ? DiagnosticMode : null,
        _selected?.IsBasedpyright == true ? TypeCheckingMode : null,
        _selected?.IsBasedpyright == true ? ConfigPathMode : null,
        _selected?.IsBasedpyright == true && IsCustomConfigPath ? NullIfEmpty(ConfigPathInput.Text) : null,
        _selected?.IsRustAnalyzer == true ? CargoFeatureMode : null,
        _selected?.IsRustAnalyzer == true && IsCustomCargoFeatures
            ? string.Join('\n', Lines(CargoFeaturesInput.Text))
            : null,
        _selected?.IsRustAnalyzer == true ? AllTargetsToggle.IsChecked == true : null,
        _selected?.IsRustAnalyzer == true ? BuildScriptsToggle.IsChecked == true : null,
        _selected?.IsRustAnalyzer == true ? ProcMacrosToggle.IsChecked == true : null,
        _selected?.IsRustAnalyzer == true ? CheckOnSaveToggle.IsChecked == true : null,
        _selected?.IsRustAnalyzer == true ? RustCheckCommand : null,
        _selected?.IsRustAnalyzer == true ? RustTargetMode : null,
        _selected?.IsRustAnalyzer == true && IsCustomRustTarget ? NullIfEmpty(RustTargetInput.Text) : null,
        string.Join('\n', Lines(RootMarkersInput.Text)),
        string.Join('\n', Lines(EnvironmentInput.Text)),
        StartupTimeoutInput.Value,
        RequestTimeoutInput.Value);

    private static string? NullIfEmpty(string? value) =>
        string.IsNullOrWhiteSpace(value) ? null : value.Trim();

    private static string RustToolchainLabel(LanguageServerItem item)
    {
        var value = string.Join(" · ", new[]
        {
            string.IsNullOrWhiteSpace(item.DetectedRustVersion) ? null : $"Rust {item.DetectedRustVersion}",
            string.IsNullOrWhiteSpace(item.DetectedCargoVersion) ? null : $"Cargo {item.DetectedCargoVersion}"
        }.Where(version => version is not null));
        return value.Length > 0 ? value : L("LocNotDetected", "Not detected");
    }

    private static string[] Lines(string? value) => (value ?? string.Empty)
        .Split(['\r', '\n'], StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries)
        .Where(line => line.Length > 0)
        .ToArray();

    private sealed record ServerDraft(
        string Kind,
        bool Enabled,
        string Executable,
        string? JavaHome,
        string? NodePath,
        string? PythonPath,
        string DataPathMode,
        string? DataPath,
        string? TypeScriptSdkMode,
        string? TypeScriptSdkPath,
        decimal? MaxTsServerMemory,
        string? DiagnosticMode,
        string? TypeCheckingMode,
        string? ConfigPathMode,
        string? ConfigPath,
        string? CargoFeatureMode,
        string? CargoFeatures,
        bool? AllCargoTargets,
        bool? BuildScripts,
        bool? ProcMacros,
        bool? CheckOnSave,
        string? RustCheckCommand,
        string? RustTargetMode,
        string? RustTarget,
        string RootMarkers,
        string Environment,
        decimal? StartupTimeout,
        decimal? RequestTimeout);
}

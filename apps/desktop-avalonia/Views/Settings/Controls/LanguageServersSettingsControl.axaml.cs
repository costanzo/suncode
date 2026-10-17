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
    private JdtlsDraft? _baseline;
    private bool _populating;
    private readonly SCComboBoxItem _managedDataPathOption;
    private readonly SCComboBoxItem _customDataPathOption;

    public LanguageServersSettingsControl()
    {
        InitializeComponent();
        _managedDataPathOption = new(L("LocManagedBySunCode", "Managed by SunCode"), "managed");
        _customDataPathOption = new(L("LocCustom", "Custom"), "custom");
        DataPathModeSelector.ItemsSource = [_managedDataPathOption, _customDataPathOption];
        DataPathModeSelector.SelectedItem = _managedDataPathOption;
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
        if (!string.IsNullOrWhiteSpace(refreshed.DetectedJavaHome))
            JavaHomeInput.Text = refreshed.DetectedJavaHome;
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
        var saved = await ViewModel.SaveJdtlsAsync(
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
            ValidationText.Text = "jdtls executable is required.";
            return false;
        }
        if (IsCustomDataPath && string.IsNullOrWhiteSpace(DataPathInput.Text))
        {
            ValidationText.Text = "A custom data directory is required.";
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
            ExecutableInput.Text = item.Command;
            JavaHomeInput.Text = item.JavaHome ?? string.Empty;
            _managedDataPath = item.DataPathMode == "managed" ? item.DataPath : string.Empty;
            DataPathModeSelector.SelectedItem = item.DataPathMode == "custom"
                ? _customDataPathOption
                : _managedDataPathOption;
            DataPathInput.Text = item.DataPath;
            RootMarkersInput.Text = string.Join(Environment.NewLine, item.RootMarkers);
            EnvironmentInput.Text = string.Empty;
            EnvironmentInput.PlaceholderText = item.EnvironmentKeys.Count > 0
                ? $"Stored keys: {string.Join(", ", item.EnvironmentKeys)}"
                : "NAME=value";
            StartupTimeoutInput.Value = item.StartupTimeoutSeconds;
            RequestTimeoutInput.Value = item.RequestTimeoutSeconds;
            ValidationText.Text = string.Empty;
            UpdateDataPathMode();
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

    private bool IsCustomDataPath =>
        string.Equals(DataPathModeSelector.SelectedItem?.Value as string, "custom", StringComparison.Ordinal);

    private void UpdateDataPathMode()
    {
        var custom = IsCustomDataPath;
        DataPathInput.IsEnabled = custom;
        if (!custom && _managedDataPath.Length > 0) DataPathInput.Text = _managedDataPath;
    }

    private void WireDirtyTracking()
    {
        EnabledToggle.Click += (_, _) => UpdateDirtyState();
        ExecutableInput.TextChanged += (_, _) => UpdateDirtyState();
        JavaHomeInput.TextChanged += (_, _) => UpdateDirtyState();
        DataPathInput.TextChanged += (_, _) => UpdateDirtyState();
        RootMarkersInput.TextChanged += (_, _) => UpdateDirtyState();
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

    private JdtlsDraft CaptureDraft() => new(
        EnabledToggle.IsChecked == true,
        ExecutableInput.Text?.Trim() ?? string.Empty,
        NullIfEmpty(JavaHomeInput.Text),
        IsCustomDataPath ? "custom" : "managed",
        IsCustomDataPath ? NullIfEmpty(DataPathInput.Text) : null,
        string.Join('\n', Lines(RootMarkersInput.Text)),
        string.Join('\n', Lines(EnvironmentInput.Text)),
        StartupTimeoutInput.Value,
        RequestTimeoutInput.Value);

    private static string? NullIfEmpty(string? value) =>
        string.IsNullOrWhiteSpace(value) ? null : value.Trim();

    private static string[] Lines(string? value) => (value ?? string.Empty)
        .Split(['\r', '\n'], StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries)
        .Where(line => line.Length > 0)
        .ToArray();

    private sealed record JdtlsDraft(
        bool Enabled,
        string Executable,
        string? JavaHome,
        string DataPathMode,
        string? DataPath,
        string RootMarkers,
        string Environment,
        decimal? StartupTimeout,
        decimal? RequestTimeout);
}

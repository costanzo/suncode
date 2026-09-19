using System.Text.Json;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Interactivity;
using SunCode.Desktop.Models;
using SunCode.Desktop.ViewModels;
using SunCode.Sdk.Models;

namespace SunCode.Desktop.Views.Settings;

public sealed partial class LanguageServerEditorWindow : Window
{
    private readonly DesktopViewModel? _viewModel;
    private readonly LanguageServerItem? _server;

    public event Action? Saved;

    public LanguageServerEditorWindow()
    {
        InitializeComponent();
        SetIcon();
    }

    public LanguageServerEditorWindow(DesktopViewModel viewModel, LanguageServerItem? server = null)
    {
        InitializeComponent();
        _viewModel = viewModel;
        _server = server;
        SetIcon();
        Populate();
        AddHandler(KeyDownEvent, WindowKeyDown, RoutingStrategies.Tunnel);
        Opened += (_, _) => ServerNameInput.Focus();
    }

    private void SetIcon() => Icon = new WindowIcon(
        Avalonia.Platform.AssetLoader.Open(new Uri("avares://SunCode/Assets/logo/suncode-logo-128.png")));

    private void Populate()
    {
        var editing = _server is not null;
        Title = editing ? "Edit language server" : "Add language server";
        HeadingText.Text = Title;
        SaveButton.Content = editing ? "Save changes" : "Add server";
        ServerNameInput.Text = _server?.DisplayName ?? string.Empty;
        CommandInput.Text = _server?.Command ?? string.Empty;
        ArgumentsInput.Text = _server is null ? string.Empty : string.Join(Environment.NewLine, _server.Arguments);
        LanguageIdsInput.Text = _server is null ? string.Empty : string.Join(Environment.NewLine, _server.LanguageIds);
        RootMarkersInput.Text = _server is null ? string.Empty : string.Join(Environment.NewLine, _server.RootMarkers);
        InitializationOptionsInput.Text = _server is null
            ? string.Empty
            : _server.InitializationOptions.ValueKind is JsonValueKind.Null or JsonValueKind.Undefined
                ? string.Empty
                : JsonSerializer.Serialize(_server.InitializationOptions, new JsonSerializerOptions { WriteIndented = true });
        StartupTimeoutInput.Text = (_server?.StartupTimeoutSeconds ?? 30).ToString();
        RequestTimeoutInput.Text = (_server?.RequestTimeoutSeconds ?? 30).ToString();
        EnabledToggle.IsChecked = _server?.Enabled ?? true;
        StoredKeysText.IsVisible = _server?.EnvironmentKeys.Count > 0;
        StoredKeysText.Text = _server?.EnvironmentKeys.Count > 0
            ? $"Stored keys: {string.Join(", ", _server.EnvironmentKeys)}"
            : string.Empty;
        EnvironmentInput.PlaceholderText = editing ? "NAME=value replaces; -NAME removes" : "NAME=value";
    }

    private void WindowKeyDown(object? sender, KeyEventArgs e)
    {
        if (e.Key != Key.Escape) return;
        e.Handled = true;
        Close();
    }

    private void CancelClicked(object? sender, RoutedEventArgs e) => Close();

    private async void SaveClicked(object? sender, RoutedEventArgs e)
    {
        if (_viewModel is null || !TryCreateRequest(out var request)) return;
        SaveButton.IsEnabled = false;
        ValidationText.Text = string.Empty;
        var saved = _server is null
            ? await _viewModel.CreateLanguageServerAsync(request)
            : await _viewModel.UpdateLanguageServerAsync(_server, request);
        SaveButton.IsEnabled = true;
        if (!saved)
        {
            ValidationText.Text = _viewModel.LanguageServerStatusText;
            return;
        }
        Saved?.Invoke();
        Close();
    }

    private bool TryCreateRequest(out LanguageServerWriteRequest request)
    {
        request = null!;
        var displayName = ServerNameInput.Text?.Trim() ?? string.Empty;
        var command = CommandInput.Text?.Trim() ?? string.Empty;
        if (displayName.Length == 0)
        {
            ValidationText.Text = "Server name is required.";
            return false;
        }
        if (command.Length == 0)
        {
            ValidationText.Text = "Executable is required.";
            return false;
        }
        var languageIds = Lines(LanguageIdsInput.Text);
        if (languageIds.Length == 0)
        {
            ValidationText.Text = "At least one language ID is required.";
            return false;
        }
        if (!ulong.TryParse(StartupTimeoutInput.Text?.Trim(), out var startupTimeout) || startupTimeout is < 1 or > 120
            || !ulong.TryParse(RequestTimeoutInput.Text?.Trim(), out var requestTimeout) || requestTimeout is < 1 or > 600)
        {
            ValidationText.Text = "Startup timeout must be 1-120 seconds and request timeout must be 1-600 seconds.";
            return false;
        }
        JsonElement initializationOptions;
        try
        {
            using var document = JsonDocument.Parse(string.IsNullOrWhiteSpace(InitializationOptionsInput.Text)
                ? "{}"
                : InitializationOptionsInput.Text);
            if (document.RootElement.ValueKind != JsonValueKind.Object)
            {
                ValidationText.Text = "Initialization options must be a JSON object.";
                return false;
            }
            initializationOptions = document.RootElement.Clone();
        }
        catch (JsonException)
        {
            ValidationText.Text = "Initialization options contain invalid JSON.";
            return false;
        }
        if (!TryParseEnvironment(out var environment)) return false;

        request = new LanguageServerWriteRequest(
            displayName,
            command,
            Lines(ArgumentsInput.Text),
            languageIds,
            Lines(RootMarkersInput.Text),
            initializationOptions,
            environment,
            startupTimeout,
            requestTimeout,
            EnabledToggle.IsChecked == true,
            _server?.SortOrder ?? 0);
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
        changes = set.Count == 0 && remove.Count == 0
            ? null
            : new LanguageServerEnvironmentChanges(set, remove);
        return true;
    }

    private static string[] Lines(string? value) => (value ?? string.Empty)
        .Split(['\r', '\n'], StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries)
        .Where(line => line.Length > 0)
        .ToArray();
}

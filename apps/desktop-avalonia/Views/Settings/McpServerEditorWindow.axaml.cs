using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Interactivity;
using SunCode.Desktop.Controls;
using SunCode.Desktop.Models;
using SunCode.Desktop.ViewModels;
using SunCode.Sdk.Models;

namespace SunCode.Desktop.Views.Settings;

public sealed partial class McpServerEditorWindow : Window
{
    private readonly SCComboBoxItem[] _transportItems =
    [
        new("Local process (stdio)", "stdio"),
        new("Remote (Streamable HTTP)", "streamable_http")
    ];
    private readonly SCComboBoxItem[] _workingDirectoryItems =
    [
        new("Project directory", "project"),
        new("Application data directory", "application_data")
    ];
    private readonly DesktopViewModel? _viewModel;
    private readonly McpServerItem? _server;

    public event Action? Saved;

    public McpServerEditorWindow()
    {
        InitializeComponent();
        InitializeSelectors();
        SetIcon();
    }

    public McpServerEditorWindow(DesktopViewModel viewModel, McpServerItem? server = null)
    {
        InitializeComponent();
        InitializeSelectors();
        _viewModel = viewModel;
        _server = server;
        SetIcon();
        Populate();
        AddHandler(KeyDownEvent, WindowKeyDown, RoutingStrategies.Tunnel);
        Opened += (_, _) => ServerNameInput.Focus();
    }

    private bool IsStdio => TransportSelector.SelectedItem?.Value as string != "streamable_http";

    private void InitializeSelectors() 
    {
        TransportSelector.ItemsSource = _transportItems;
        WorkingDirectorySelector.ItemsSource = _workingDirectoryItems;
    }

    private void SetIcon() => Icon = new WindowIcon(
        Avalonia.Platform.AssetLoader.Open(new Uri("avares://SunCode/Assets/logo/suncode-logo-128.png")));

    private void Populate()
    {
        var editing = _server is not null;
        Title = editing ? "Edit MCP server" : "Add MCP server";
        HeadingText.Text = Title;
        SaveButton.Content = editing ? "Save changes" : "Add server";
        TransportSelector.SelectedItem = _server?.TransportType == "streamable_http" ? _transportItems[1] : _transportItems[0];
        WorkingDirectorySelector.SelectedItem = _server?.WorkingDirectory == "application_data" ? _workingDirectoryItems[1] : _workingDirectoryItems[0];
        ServerNameInput.Text = _server?.DisplayName ?? string.Empty;
        CommandInput.Text = _server?.Command ?? string.Empty;
        ArgumentsInput.Text = _server is null ? string.Empty : string.Join(Environment.NewLine, _server.Arguments);
        UrlInput.Text = _server?.Url ?? string.Empty;
        StartupTimeoutInput.Text = (_server?.StartupTimeoutSeconds ?? 30).ToString();
        RequestTimeoutInput.Text = (_server?.RequestTimeoutSeconds ?? 60).ToString();
        EnabledToggle.IsChecked = _server?.Enabled ?? true;
        RefreshTransportFields();
    }

    private void TransportChanged(object? sender, SelectionChangedEventArgs e) => RefreshTransportFields();

    private void RefreshTransportFields()
    {
        if (StdioCommandField is null) return;
        StdioCommandField.IsVisible = IsStdio;
        WorkingDirectoryField.IsVisible = IsStdio;
        ArgumentsField.IsVisible = IsStdio;
        HttpUrlField.IsVisible = !IsStdio;
        SecretsLabel.Text = IsStdio ? "Environment" : "HTTP headers";
        SecretsInput.PlaceholderText = _server is null
            ? IsStdio ? "NAME=value" : "Authorization=Bearer ..."
            : "NAME=value replaces; -NAME removes";
        var keys = IsStdio ? _server?.EnvironmentKeys : _server?.HeaderKeys;
        StoredKeysText.IsVisible = keys is { Count: > 0 };
        StoredKeysText.Text = keys is { Count: > 0 }
            ? $"Stored keys: {string.Join(", ", keys)}"
            : string.Empty;
        EnableHintText.Text = IsStdio
            ? "Starts a local process with your user authority."
            : "Connects to the remote endpoint after saving.";
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
            ? await _viewModel.CreateMcpServerAsync(request)
            : await _viewModel.UpdateMcpServerAsync(_server, request);
        SaveButton.IsEnabled = true;
        if (!saved)
        {
            ValidationText.Text = _viewModel.McpStatusText;
            return;
        }
        Saved?.Invoke();
        Close();
    }

    private bool TryCreateRequest(out McpServerWriteRequest request)
    {
        request = null!;
        var displayName = ServerNameInput.Text?.Trim() ?? string.Empty;
        if (displayName.Length == 0)
        {
            ValidationText.Text = "Server name is required.";
            return false;
        }
        if (!ulong.TryParse(StartupTimeoutInput.Text?.Trim(), out var startupTimeout) || startupTimeout is < 1 or > 120
            || !ulong.TryParse(RequestTimeoutInput.Text?.Trim(), out var requestTimeout) || requestTimeout is < 1 or > 600)
        {
            ValidationText.Text = "Startup timeout must be 1-120 seconds and request timeout must be 1-600 seconds.";
            return false;
        }
        if (!TryParseSecrets(out var secretChanges)) return false;

        McpTransportRequest transport;
        if (IsStdio)
        {
            var command = CommandInput.Text?.Trim() ?? string.Empty;
            if (command.Length == 0)
            {
                ValidationText.Text = "Executable is required.";
                return false;
            }
            var arguments = (ArgumentsInput.Text ?? string.Empty)
                .Split(['\r', '\n'], StringSplitOptions.RemoveEmptyEntries)
                .Select(value => value.Trim())
                .Where(value => value.Length > 0)
                .ToArray();
            transport = new McpStdioTransportRequest(
                command,
                arguments,
                WorkingDirectorySelector.SelectedItem?.Value as string ?? "project",
                secretChanges,
                startupTimeout,
                requestTimeout);
        }
        else
        {
            var url = UrlInput.Text?.Trim() ?? string.Empty;
            if (url.Length == 0)
            {
                ValidationText.Text = "Server URL is required.";
                return false;
            }
            transport = new McpStreamableHttpTransportRequest(
                url,
                secretChanges,
                startupTimeout,
                requestTimeout);
        }

        request = new McpServerWriteRequest(
            displayName,
            transport,
            EnabledToggle.IsChecked == true,
            _server?.Server.SortOrder ?? 0);
        return true;
    }

    private bool TryParseSecrets(out McpSecretChanges? changes)
    {
        var set = new Dictionary<string, string>(StringComparer.Ordinal);
        var remove = new List<string>();
        var lines = (SecretsInput.Text ?? string.Empty)
            .Split(['\r', '\n'], StringSplitOptions.RemoveEmptyEntries);
        foreach (var rawLine in lines)
        {
            var line = rawLine.Trim();
            if (line.Length == 0) continue;
            if (line.StartsWith('-'))
            {
                var key = line[1..].Trim();
                if (key.Length == 0 || key.Contains('='))
                {
                    ValidationText.Text = "Secret removals must use -NAME.";
                    changes = null;
                    return false;
                }
                remove.Add(key);
                continue;
            }
            var separator = line.IndexOf('=');
            if (separator <= 0 || separator == line.Length - 1)
            {
                ValidationText.Text = "Secrets must use NAME=value, or -NAME to remove a stored key.";
                changes = null;
                return false;
            }
            set[line[..separator].Trim()] = line[(separator + 1)..];
        }
        changes = set.Count == 0 && remove.Count == 0 ? null : new McpSecretChanges(set, remove);
        return true;
    }
}

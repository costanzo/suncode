using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Interactivity;
using SunCode.Desktop.Controls;
using SunCode.Desktop.Models;
using SunCode.Desktop.ViewModels;
using SunCode.Sdk.Models;
using SunCode.Desktop.Infrastructure;

namespace SunCode.Desktop.Views.Settings;

public sealed partial class McpServerEditorWindow : Window
{
    private static string L(string key, string fallback) => LocalizationService.GetString(key, fallback);

    private readonly SCComboBoxItem[] _transportItems =
    [
        new(L("LocMcpTransportStdio", "Local process (stdio)"), "stdio"),
        new(L("LocMcpTransportHttp", "Remote (Streamable HTTP)"), "streamable_http")
    ];
    private readonly SCComboBoxItem[] _workingDirectoryItems =
    [
        new(L("LocProjectDirectory", "Project directory"), "project"),
        new(L("LocApplicationDataDirectory", "Application data directory"), "application_data")
    ];
    private readonly McpServersViewModel? _viewModel;
    private readonly McpServerItem? _server;

    public event Action? Saved;

    public McpServerEditorWindow()
    {
        InitializeComponent();
        InitializeSelectors();
        SetIcon();
    }

    public McpServerEditorWindow(McpServersViewModel viewModel, McpServerItem? server = null)
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

    private void SetIcon() => Icon = AppIcon.Window;

    private void Populate()
    {
        var editing = _server is not null;
        Title = editing ? L("LocEditMcpServer", "Edit MCP server") : L("LocAddMcpServer", "Add MCP server");
        HeadingText.Text = Title;
        SaveButton.Content = editing ? L("LocSaveChanges", "Save changes") : L("LocAddServer", "Add server");
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
        SecretsLabel.Text = IsStdio ? L("LocEnvironment", "Environment") : L("LocHttpHeaders", "HTTP headers");
        SecretsInput.PlaceholderText = _server is null
            ? IsStdio ? L("LocNameValue", "NAME=value") : "Authorization=Bearer ..."
            : L("LocNameValueEditHint", "NAME=value replaces; -NAME removes");
        var keys = IsStdio ? _server?.EnvironmentKeys : _server?.HeaderKeys;
        StoredKeysText.IsVisible = keys is { Count: > 0 };
        StoredKeysText.Text = keys is { Count: > 0 }
            ? string.Format(L("LocStoredKeys", "Stored keys: {0}"), string.Join(", ", keys))
            : string.Empty;
        EnableHintText.Text = IsStdio
            ? L("LocLocalProcessHint", "Starts a local process with your user authority.")
            : L("LocRemoteEndpointHint", "Connects to the remote endpoint after saving.");
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
            ? await _viewModel.CreateServerAsync(request)
            : await _viewModel.UpdateServerAsync(_server, request);
        SaveButton.IsEnabled = true;
        if (!saved)
        {
            ValidationText.Text = _viewModel.StatusText;
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
            ValidationText.Text = L("LocServerNameRequired", "Server name is required.");
            return false;
        }
        if (!ulong.TryParse(StartupTimeoutInput.Text?.Trim(), out var startupTimeout) || startupTimeout is < 1 or > 120
            || !ulong.TryParse(RequestTimeoutInput.Text?.Trim(), out var requestTimeout) || requestTimeout is < 1 or > 600)
        {
            ValidationText.Text = L("LocTimeoutRangeInvalid", "Startup timeout must be 1-120 seconds and request timeout must be 1-600 seconds.");
            return false;
        }
        if (!TryParseSecrets(out var secretChanges)) return false;

        McpTransportRequest transport;
        if (IsStdio)
        {
            var command = CommandInput.Text?.Trim() ?? string.Empty;
            if (command.Length == 0)
            {
                ValidationText.Text = L("LocExecutableRequired", "Executable is required.");
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
                ValidationText.Text = L("LocServerUrlRequired", "Server URL is required.");
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
                    ValidationText.Text = L("LocSecretRemovalFormat", "Secret removals must use -NAME.");
                    changes = null;
                    return false;
                }
                remove.Add(key);
                continue;
            }
            var separator = line.IndexOf('=');
            if (separator <= 0 || separator == line.Length - 1)
            {
                ValidationText.Text = L("LocSecretFormat", "Secrets must use NAME=value, or -NAME to remove a stored key.");
                changes = null;
                return false;
            }
            set[line[..separator].Trim()] = line[(separator + 1)..];
        }
        changes = set.Count == 0 && remove.Count == 0 ? null : new McpSecretChanges(set, remove);
        return true;
    }
}

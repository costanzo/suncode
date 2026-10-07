using Avalonia.Controls;
using Avalonia.Styling;
using SunCode.Desktop.Models;
using SunCode.Desktop.ViewModels;
using SunCode.Desktop.Views.About;
using SunCode.Desktop.Views.DialogWindow;
using SunCode.Desktop.Views.Settings;

namespace SunCode.Desktop.Infrastructure;

/// <summary>
/// Shows the app-level modal dialogs (Settings, About, session confirmations) and
/// disables every other app window while one is open. UI-thread only.
/// </summary>
internal sealed class DialogService
{
    private readonly Func<DesktopViewModel?> _fallbackViewModel;
    private readonly Action<Window, bool> _setOtherWindowsEnabled;
    private readonly Dictionary<Control, bool> _suspendedSettingsTooltips = [];
    private SettingsWindow? _settingsWindow;
    private AboutWindow? _aboutWindow;

    public DialogService(Func<DesktopViewModel?> fallbackViewModel, Action<Window, bool> setOtherWindowsEnabled)
    {
        _fallbackViewModel = fallbackViewModel;
        _setOtherWindowsEnabled = setOtherWindowsEnabled;
    }

    public void ShowSettings(Window owner, Control? source = null)
    {
        var viewModel = owner.DataContext switch
        {
            DesktopViewModel windowViewModel => windowViewModel,
            ProjectHubViewModel hub => hub.Services,
            _ => _fallbackViewModel()
        };
        if (viewModel is null) return;
        if (_settingsWindow is not null)
        {
            _settingsWindow.Activate();
            return;
        }
        SuspendSettingsTooltip(source);

        _settingsWindow = new SettingsWindow { DataContext = viewModel };
        _setOtherWindowsEnabled(owner, false);
        _settingsWindow.Closed += (_, _) =>
        {
            _setOtherWindowsEnabled(owner, true);
            RestoreSettingsTooltips();
            _settingsWindow = null;
        };
        _ = _settingsWindow.ShowDialog(owner);
    }

    public void ShowAbout(Window owner)
    {
        if (_aboutWindow is not null)
        {
            _aboutWindow.Activate();
            return;
        }

        _aboutWindow = new AboutWindow();
        _setOtherWindowsEnabled(owner, false);
        _aboutWindow.Closed += (_, _) =>
        {
            _setOtherWindowsEnabled(owner, true);
            _aboutWindow = null;
        };
        _ = _aboutWindow.ShowDialog(owner);
    }

    public void ShowArchiveConfirmation(Window owner, SessionItem session, Action confirm) =>
        ShowSessionConfirmation(
            owner,
            "Archive this session?",
            "It will leave the active session list, but can be reopened later.",
            session.DisplayTitle,
            confirm,
            "Archive session");

    public void ShowSessionConfirmation(
        Window owner,
        string title,
        string description,
        string target,
        Action confirm,
        string confirmLabel)
    {
        var dialog = new DialogWindow(title, description, target, confirm, confirmLabel: confirmLabel);
        _setOtherWindowsEnabled(owner, false);
        dialog.Closed += (_, _) => _setOtherWindowsEnabled(owner, true);
        _ = dialog.ShowDialog(owner);
    }

    public void ApplyTheme(ThemeVariant variant)
    {
        if (_settingsWindow is not null) _settingsWindow.RequestedThemeVariant = variant;
        if (_aboutWindow is not null) _aboutWindow.RequestedThemeVariant = variant;
    }

    public void CloseAll()
    {
        _settingsWindow?.Close();
        _aboutWindow?.Close();
    }

    private void SuspendSettingsTooltip(Control? source)
    {
        if (source is null) return;
        if (!_suspendedSettingsTooltips.ContainsKey(source))
            _suspendedSettingsTooltips[source] = ToolTip.GetServiceEnabled(source);

        // The pointer is still over the button when ShowDialog disables its owner.
        // Stop the tooltip service before the modal transition so it cannot reopen
        // against a disabled owner and keep invalidating the UI layout.
        ToolTip.SetIsOpen(source, false);
        ToolTip.SetServiceEnabled(source, false);
    }

    private void RestoreSettingsTooltips()
    {
        foreach (var (control, wasEnabled) in _suspendedSettingsTooltips)
            ToolTip.SetServiceEnabled(control, wasEnabled);
        _suspendedSettingsTooltips.Clear();
    }
}

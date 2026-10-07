using System.Windows.Input;
using SunCode.Desktop.Controls;
using SunCode.Desktop.Infrastructure;
using SunCode.Desktop.Models;
using static SunCode.Desktop.ViewModels.SettingsFormatting;

namespace SunCode.Desktop.ViewModels;

// Defaults page: the global default model (saved on selection) and the
// project tool-call limit (saved explicitly while it differs from the baseline).
public sealed class DefaultsSettingsViewModel : ObservableObject
{
    private readonly DesktopViewModel _desktop;
    private readonly AsyncRelayCommand _saveToolCallLimit;
    private IReadOnlyList<SCComboBoxItem> _modelOptions = [];
    private SCComboBoxItem? _selectedModel;
    private decimal? _toolCallLimit;
    private int _baselineToolCallLimit;
    private bool _loaded;
    private string _statusText = string.Empty;
    private StatusTone _statusTone;

    internal DefaultsSettingsViewModel(DesktopViewModel desktop)
    {
        _desktop = desktop;
        _saveToolCallLimit = new AsyncRelayCommand(SaveToolCallLimitAsync, () => !_loaded || IsToolCallLimitDirty);
    }

    public ICommand SaveToolCallLimitCommand => _saveToolCallLimit;

    public IReadOnlyList<SCComboBoxItem> ModelOptions { get => _modelOptions; private set => SetProperty(ref _modelOptions, value); }

    public SCComboBoxItem? SelectedModel
    {
        get => _selectedModel;
        set
        {
            if (SetProperty(ref _selectedModel, value) && value?.Value is ModelItem model)
                _ = _desktop.SaveDefaultModelAsync(model);
        }
    }

    public decimal? ToolCallLimit
    {
        get => _toolCallLimit;
        set
        {
            if (!SetProperty(ref _toolCallLimit, value)) return;
            OnPropertyChanged(nameof(IsToolCallLimitDirty));
            _saveToolCallLimit.RaiseCanExecuteChanged();
        }
    }

    public bool CanEditToolCallLimit => _desktop.IsProjectOpen;

    public string ToolCallLimitScopeText => !_loaded ? string.Empty
        : _desktop.SelectedProject is { } project
            ? LF("LocProjectScope", "Project: {0}", project.DisplayName)
            : L("LocOpenProjectToConfigure", "Open a project to configure this setting.");

    public bool IsToolCallLimitDirty =>
        _desktop.IsProjectOpen
        && ToolCallLimit is { } value
        && (int)value != _baselineToolCallLimit;

    public string StatusText { get => _statusText; private set => SetProperty(ref _statusText, value); }
    public StatusTone StatusTone { get => _statusTone; private set => SetProperty(ref _statusTone, value); }

    // Call after the project tool-call limit has been loaded.
    public void Load()
    {
        _loaded = true;
        _toolCallLimit = _desktop.ToolCallLimit;
        _baselineToolCallLimit = _desktop.ToolCallLimit;
        OnPropertyChanged(nameof(ToolCallLimit));
        OnPropertyChanged(nameof(CanEditToolCallLimit));
        OnPropertyChanged(nameof(ToolCallLimitScopeText));
        RefreshModelOptions();
        RefreshDirty();
    }

    // Rebuilds the model list and re-selects the window's selected model without saving.
    public void RefreshModelOptions()
    {
        ModelOptions = _desktop.Models.Select(model => new SCComboBoxItem(model.Id, model)).ToArray();
        _selectedModel = ModelOptions.FirstOrDefault(item => item.Value is ModelItem model && model.Id == _desktop.SelectedModel?.Id);
        OnPropertyChanged(nameof(SelectedModel));
    }

    private async Task SaveToolCallLimitAsync()
    {
        if (ToolCallLimit is not { } value) return;
        var saved = await _desktop.SaveProjectToolCallLimitAsync(decimal.ToInt32(value));
        StatusText = _desktop.StatusText;
        StatusTone = SaveResultTone(saved);
        if (saved) _baselineToolCallLimit = _desktop.ToolCallLimit;
        RefreshDirty();
    }

    private void RefreshDirty()
    {
        OnPropertyChanged(nameof(IsToolCallLimitDirty));
        _saveToolCallLimit.RaiseCanExecuteChanged();
    }
}

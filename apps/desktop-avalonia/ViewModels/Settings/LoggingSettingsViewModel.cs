using System.Globalization;
using System.Windows.Input;
using SunCode.Desktop.Controls;
using SunCode.Desktop.Infrastructure;
using static SunCode.Desktop.ViewModels.SettingsFormatting;

namespace SunCode.Desktop.ViewModels;

// Edit state of the Logging settings page: diagnostic log output and image
// storage. Each section tracks its own saved baseline so its Save command is
// enabled only while that section has unsaved changes.
public sealed class LoggingSettingsViewModel : ObservableObject
{
    private const long Megabyte = 1024 * 1024;

    public static IReadOnlyList<SCComboBoxItem> LogLevelOptions { get; } =
    [
        new("TRACE", "TRACE"),
        new("DEBUG", "DEBUG"),
        new("INFO", "INFO"),
        new("WARN", "WARN"),
        new("ERROR", "ERROR"),
        new("OFF", "OFF")
    ];

    private readonly IViewModelHost _host;
    private readonly AppSettingsViewModel _settings;
    private readonly AsyncRelayCommand _saveLogging;
    private readonly AsyncRelayCommand _saveImageDirectory;
    private SCComboBoxItem? _selectedLogLevel;
    private string? _logDirectory;
    private decimal? _logMaxMegabytes;
    private decimal? _logRetention;
    private string? _imageDirectory;
    private string _loggingStatusText = string.Empty;
    private StatusTone _loggingStatusTone;
    private string _imageDirectoryStatusText = string.Empty;
    private StatusTone _imageDirectoryStatusTone;
    private string _baselineLogLevel = string.Empty;
    private string _baselineLogDirectory = string.Empty;
    private long _baselineLogMaxBytes;
    private int _baselineLogRetention;
    private string _baselineImageDirectory = string.Empty;

    internal LoggingSettingsViewModel(IViewModelHost host, AppSettingsViewModel settings)
    {
        _host = host;
        _settings = settings;
        _saveLogging = new AsyncRelayCommand(SaveLoggingAsync, () => IsLoggingDirty);
        _saveImageDirectory = new AsyncRelayCommand(SaveImageDirectoryAsync, () => IsImageDirectoryDirty);
    }

    public ICommand SaveLoggingCommand => _saveLogging;
    public ICommand SaveImageDirectoryCommand => _saveImageDirectory;

    public SCComboBoxItem? SelectedLogLevel
    {
        get => _selectedLogLevel;
        set { if (SetProperty(ref _selectedLogLevel, value)) RefreshLoggingDirty(); }
    }

    public string? LogDirectory
    {
        get => _logDirectory;
        set { if (SetProperty(ref _logDirectory, value)) RefreshLoggingDirty(); }
    }

    public decimal? LogMaxMegabytes
    {
        get => _logMaxMegabytes;
        set { if (SetProperty(ref _logMaxMegabytes, value)) RefreshLoggingDirty(); }
    }

    public decimal? LogRetention
    {
        get => _logRetention;
        set { if (SetProperty(ref _logRetention, value)) RefreshLoggingDirty(); }
    }

    public string? ImageDirectory
    {
        get => _imageDirectory;
        set
        {
            if (!SetProperty(ref _imageDirectory, value)) return;
            OnPropertyChanged(nameof(IsImageDirectoryDirty));
            _saveImageDirectory.RaiseCanExecuteChanged();
        }
    }

    public string LoggingStatusText { get => _loggingStatusText; private set => SetProperty(ref _loggingStatusText, value); }
    public StatusTone LoggingStatusTone { get => _loggingStatusTone; private set => SetProperty(ref _loggingStatusTone, value); }
    public string ImageDirectoryStatusText { get => _imageDirectoryStatusText; private set => SetProperty(ref _imageDirectoryStatusText, value); }
    public StatusTone ImageDirectoryStatusTone { get => _imageDirectoryStatusTone; private set => SetProperty(ref _imageDirectoryStatusTone, value); }

    private string CurrentLogLevel => SelectedLogLevel?.Value as string ?? _settings.LogLevel;

    public bool IsLoggingDirty =>
        !string.Equals(CurrentLogLevel, _baselineLogLevel, StringComparison.Ordinal)
        || !string.Equals(NormalizeDirectory(LogDirectory), _baselineLogDirectory, StringComparison.Ordinal)
        || (LogMaxMegabytes is { } megabytes && decimal.ToInt64(megabytes) * Megabyte != _baselineLogMaxBytes)
        || (LogRetention is { } retention && decimal.ToInt32(retention) != _baselineLogRetention);

    public bool IsImageDirectoryDirty =>
        !string.Equals(NormalizeDirectory(ImageDirectory), _baselineImageDirectory, StringComparison.Ordinal);

    // Loads the persisted values into the editors and makes them the baseline.
    public void Load()
    {
        _selectedLogLevel = LogLevelOptions.FirstOrDefault(item => Equals(item.Value, _settings.LogLevel));
        _logDirectory = _settings.EffectiveLogDirectory;
        _logMaxMegabytes = Math.Max(1, _settings.LogMaxBytes / Megabyte);
        _logRetention = _settings.LogRetention;
        _imageDirectory = _settings.EffectiveImageDirectory;
        ResetLoggingBaseline();
        _baselineImageDirectory = NormalizeDirectory(_settings.EffectiveImageDirectory);
        LoggingStatusText = L("LocLocalSettings", "Local settings");
        ImageDirectoryStatusText = L("LocLocalSettings", "Local settings");
        LoggingStatusTone = StatusTone.Neutral;
        ImageDirectoryStatusTone = StatusTone.Neutral;
        OnPropertyChanged(nameof(SelectedLogLevel));
        OnPropertyChanged(nameof(LogDirectory));
        OnPropertyChanged(nameof(LogMaxMegabytes));
        OnPropertyChanged(nameof(LogRetention));
        OnPropertyChanged(nameof(ImageDirectory));
        RefreshLoggingDirty();
        OnPropertyChanged(nameof(IsImageDirectoryDirty));
        _saveImageDirectory.RaiseCanExecuteChanged();
    }

    private async Task SaveLoggingAsync()
    {
        if (LogMaxMegabytes is not { } megabytes
            || megabytes is < 1 or > 1000
            || decimal.Truncate(megabytes) != megabytes
            || LogRetention is not { } retention
            || retention is < 0 or > 100
            || decimal.Truncate(retention) != retention)
        {
            LoggingStatusText = L("LocLogValidation", "Log size must be 1–1000 MB and retained backups must be 0–100");
            LoggingStatusTone = StatusTone.Danger;
            return;
        }

        var saved = await _settings.SaveLoggingSettingsAsync(
            _host,
            CurrentLogLevel,
            LogDirectory,
            checked(decimal.ToInt64(megabytes) * Megabyte).ToString(CultureInfo.InvariantCulture),
            decimal.ToInt32(retention).ToString(CultureInfo.InvariantCulture));
        LoggingStatusText = _host.StatusText;
        LoggingStatusTone = SaveResultTone(saved);
        if (saved) ResetLoggingBaseline();
        RefreshLoggingDirty();
    }

    private async Task SaveImageDirectoryAsync()
    {
        var saved = await _settings.SaveImageDirectoryAsync(_host, ImageDirectory);
        ImageDirectoryStatusText = _host.StatusText;
        ImageDirectoryStatusTone = SaveResultTone(saved);
        if (saved) _baselineImageDirectory = NormalizeDirectory(_settings.EffectiveImageDirectory);
        OnPropertyChanged(nameof(IsImageDirectoryDirty));
        _saveImageDirectory.RaiseCanExecuteChanged();
    }

    private void ResetLoggingBaseline()
    {
        _baselineLogLevel = _settings.LogLevel;
        _baselineLogDirectory = NormalizeDirectory(_settings.EffectiveLogDirectory);
        _baselineLogMaxBytes = _settings.LogMaxBytes;
        _baselineLogRetention = _settings.LogRetention;
    }

    private void RefreshLoggingDirty()
    {
        OnPropertyChanged(nameof(IsLoggingDirty));
        _saveLogging.RaiseCanExecuteChanged();
    }
}

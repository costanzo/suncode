using SunCode.Desktop.Infrastructure;
using SunCode.Desktop.Models;
using SunCode.Sdk.Models;

namespace SunCode.Desktop.ViewModels;

// Latest context-window usage for the selected session, measured against the
// selected model's auto-compact or input-token limit.
public sealed class ContextUsageViewModel : ObservableObject
{
    private ulong? _inputTokens;
    private ulong? _outputTokens;
    private ulong? _cachedTokens;
    private readonly Func<ModelItem?> _model;
    private bool _expanded;

    internal ContextUsageViewModel(Func<ModelItem?> model) => _model = model;

    public bool Expanded
    {
        get => _expanded;
        set => SetProperty(ref _expanded, value);
    }
    public ulong? Limit => _model()?.AutoCompactTokens ?? _model()?.MaxInputTokens;
    public bool Known => _inputTokens.HasValue && Limit is > 0;
    public string Text => Known
        ? $"{CompactTokenCount(_inputTokens!.Value)} / {CompactTokenCount(Limit!.Value)} tokens"
        : "Unavailable";
    public string PercentText => Known ? $"{Percent:0}%" : "--";
    public double Percent => Known
        ? Math.Min(100d, _inputTokens!.Value * 100d / Limit!.Value)
        : 0d;
    public bool IsWarning => Known && Percent >= 75d && Percent < 90d;
    public bool IsDanger => Known && Percent >= 90d;
    public bool IsNormal => Known && !IsWarning && !IsDanger;
    public bool IsUnknown => !Known;
    public string InputTokenText => _inputTokens is { } input ? CompactTokenCount(input) : "--";
    public string OutputTokenText => _outputTokens is { } output ? CompactTokenCount(output) : "--";
    public string CachedTokenText => _cachedTokens is { } cached ? CompactTokenCount(cached) : "--";

    internal void Reset()
    {
        _inputTokens = null;
        _outputTokens = null;
        _cachedTokens = null;
        Expanded = false;
        NotifyChanged();
    }

    internal void Update(AgentUsage usage)
    {
        _inputTokens = usage.InputTokens;
        _outputTokens = usage.OutputTokens;
        _cachedTokens = usage.CacheReadTokens ?? usage.CacheWriteTokens;
        NotifyChanged();
    }

    private void NotifyChanged()
    {
        OnPropertyChanged(nameof(Known));
        OnPropertyChanged(nameof(Limit));
        OnPropertyChanged(nameof(Text));
        OnPropertyChanged(nameof(PercentText));
        OnPropertyChanged(nameof(Percent));
        OnPropertyChanged(nameof(IsWarning));
        OnPropertyChanged(nameof(IsDanger));
        OnPropertyChanged(nameof(IsNormal));
        OnPropertyChanged(nameof(IsUnknown));
        OnPropertyChanged(nameof(InputTokenText));
        OnPropertyChanged(nameof(OutputTokenText));
        OnPropertyChanged(nameof(CachedTokenText));
    }

    private static string CompactTokenCount(ulong value) => value switch
    {
        >= 1_000_000 => $"{value / 1_000_000d:0.#}m",
        >= 1_000 => $"{value / 1_000d:0.#}k",
        _ => value.ToString()
    };
}

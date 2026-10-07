using Avalonia.Threading;
using SunCode.Desktop.Infrastructure;
using SunCode.Desktop.Models;
using SunCode.Sdk.Models;

namespace SunCode.Desktop.ViewModels;

// Live upload and download rates for in-flight built-in LLM provider requests,
// sampled once per second while any transfer is active.
public sealed class ProviderTrafficViewModel : ObservableObject
{
    private readonly DispatcherTimer _timer = new() { Interval = TimeSpan.FromSeconds(1) };
    private readonly Dictionary<string, ProviderTransferState> _transfers = new(StringComparer.Ordinal);
    private DateTimeOffset _sampleAt;
    private ulong _uploadSampleBytes;
    private ulong _downloadSampleBytes;
    private double _uploadRate;
    private double _downloadRate;

    public string Text => $"↑ {FormatTransferRate(_uploadRate)}   ↓ {FormatTransferRate(_downloadRate)}";
    public string Details => _transfers.Count == 0
        ? "No active LLM requests"
        : string.Join(Environment.NewLine, _transfers.Values
            .OrderBy(item => item.Provider, StringComparer.Ordinal)
            .Select(item => $"{item.Provider} / {item.Model}: ↑ {FormatBytes(item.UploadedBytes)}   ↓ {FormatBytes(item.DownloadedBytes)}"));

    internal void Apply(AgentEvent value)
    {
        var payload = value.Payload;
        var exchangeId = payload.ExchangeId;
        if (string.IsNullOrWhiteSpace(exchangeId)) return;

        if (value.EventType == AgentEventTypes.ProviderExchangeStarted)
        {
            _transfers[exchangeId] = new ProviderTransferState(payload.Provider ?? "LLM", payload.ModelId ?? payload.WireModel ?? "model");
            StartTimer();
        }
        else if (value.EventType == AgentEventTypes.ProviderExchangeProgress)
        {
            if (!_transfers.TryGetValue(exchangeId, out var transfer))
            {
                transfer = new ProviderTransferState(payload.Provider ?? "LLM", payload.ModelId ?? "model");
                _transfers.Add(exchangeId, transfer);
                StartTimer();
            }
            var uploaded = payload.UploadedBytes ?? transfer.UploadedBytes;
            var downloaded = payload.DownloadedBytes ?? transfer.DownloadedBytes;
            _uploadSampleBytes = SaturatingAdd(_uploadSampleBytes, uploaded >= transfer.UploadedBytes ? uploaded - transfer.UploadedBytes : 0);
            _downloadSampleBytes = SaturatingAdd(_downloadSampleBytes, downloaded >= transfer.DownloadedBytes ? downloaded - transfer.DownloadedBytes : 0);
            transfer.UploadedBytes = uploaded;
            transfer.DownloadedBytes = downloaded;
        }
        else if (value.EventType is AgentEventTypes.ProviderExchangeCompleted or AgentEventTypes.ProviderExchangeFailed)
        {
            _transfers.Remove(exchangeId);
        }
        OnPropertyChanged(nameof(Details));
    }

    private void StartTimer()
    {
        if (_timer.IsEnabled) return;
        _sampleAt = DateTimeOffset.UtcNow;
        _uploadSampleBytes = 0;
        _downloadSampleBytes = 0;
        _timer.Tick += Tick;
        _timer.Start();
    }

    private void Tick(object? sender, EventArgs args)
    {
        var now = DateTimeOffset.UtcNow;
        var elapsed = Math.Max(0.001, (now - _sampleAt).TotalSeconds);
        _uploadRate = _uploadSampleBytes / elapsed;
        _downloadRate = _downloadSampleBytes / elapsed;
        _sampleAt = now;
        _uploadSampleBytes = 0;
        _downloadSampleBytes = 0;
        OnPropertyChanged(nameof(Text));
        OnPropertyChanged(nameof(Details));
    }

    internal void Clear()
    {
        _transfers.Clear();
        _uploadSampleBytes = 0;
        _downloadSampleBytes = 0;
        _uploadRate = 0;
        _downloadRate = 0;
        _timer.Stop();
        _timer.Tick -= Tick;
        OnPropertyChanged(nameof(Text));
        OnPropertyChanged(nameof(Details));
    }

    private static ulong SaturatingAdd(ulong left, ulong right) => ulong.MaxValue - left < right ? ulong.MaxValue : left + right;

    private static string FormatTransferRate(double bytesPerSecond) => $"{FormatBytes((ulong)Math.Max(0, bytesPerSecond))}/s";

    private static string FormatBytes(ulong bytes)
    {
        if (bytes < 1024) return $"{bytes} B";
        var value = bytes / 1024d;
        var unit = "KiB";
        if (value >= 1024) { value /= 1024; unit = "MiB"; }
        if (value >= 1024) { value /= 1024; unit = "GiB"; }
        return $"{value:0.#} {unit}";
    }

    private sealed class ProviderTransferState(string provider, string model)
    {
        public string Provider { get; } = provider;
        public string Model { get; } = model;
        public ulong UploadedBytes { get; set; }
        public ulong DownloadedBytes { get; set; }
    }
}

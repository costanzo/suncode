using Avalonia.Threading;
using SunCode.Sdk.Models;

namespace SunCode.Desktop.ViewModels;

public sealed partial class DesktopViewModel
{
    private readonly DispatcherTimer _providerTrafficTimer = new() { Interval = TimeSpan.FromSeconds(3) };
    private readonly Dictionary<string, ProviderTransferState> _providerTransfers = new(StringComparer.Ordinal);
    private DateTimeOffset _providerTrafficSampleAt;
    private ulong _providerUploadSampleBytes;
    private ulong _providerDownloadSampleBytes;
    private double _providerUploadRate;
    private double _providerDownloadRate;

    public bool IsProviderTrafficVisible => IsProjectOpen;
    public string ProviderTrafficText => $"↑ {FormatTransferRate(_providerUploadRate)}   ↓ {FormatTransferRate(_providerDownloadRate)}";
    public string ProviderTrafficDetails => _providerTransfers.Count == 0
        ? "No active LLM requests"
        : string.Join(Environment.NewLine, _providerTransfers.Values
            .OrderBy(item => item.Provider, StringComparer.Ordinal)
            .Select(item => $"{item.Provider} / {item.Model}: ↑ {FormatBytes(item.UploadedBytes)}   ↓ {FormatBytes(item.DownloadedBytes)}"));

    internal void ApplyProviderTransferEvent(AgentEvent value)
    {
        var payload = value.Payload;
        var exchangeId = payload.ExchangeId;
        if (string.IsNullOrWhiteSpace(exchangeId)) return;

        if (value.EventType == "provider.exchange.started")
        {
            _providerTransfers[exchangeId] = new ProviderTransferState(payload.Provider ?? "LLM", payload.ModelId ?? payload.WireModel ?? "model");
            StartProviderTrafficTimer();
        }
        else if (value.EventType == "provider.exchange.progress")
        {
            if (!_providerTransfers.TryGetValue(exchangeId, out var transfer))
            {
                transfer = new ProviderTransferState(payload.Provider ?? "LLM", payload.ModelId ?? "model");
                _providerTransfers.Add(exchangeId, transfer);
                StartProviderTrafficTimer();
            }
            var uploaded = payload.UploadedBytes ?? transfer.UploadedBytes;
            var downloaded = payload.DownloadedBytes ?? transfer.DownloadedBytes;
            _providerUploadSampleBytes = SaturatingAdd(_providerUploadSampleBytes, uploaded >= transfer.UploadedBytes ? uploaded - transfer.UploadedBytes : 0);
            _providerDownloadSampleBytes = SaturatingAdd(_providerDownloadSampleBytes, downloaded >= transfer.DownloadedBytes ? downloaded - transfer.DownloadedBytes : 0);
            transfer.UploadedBytes = uploaded;
            transfer.DownloadedBytes = downloaded;
        }
        else if (value.EventType is "provider.exchange.completed" or "provider.exchange.failed")
        {
            _providerTransfers.Remove(exchangeId);
        }
        OnPropertyChanged(nameof(IsProviderTrafficVisible));
        OnPropertyChanged(nameof(ProviderTrafficDetails));
    }

    private void StartProviderTrafficTimer()
    {
        if (_providerTrafficTimer.IsEnabled) return;
        _providerTrafficSampleAt = DateTimeOffset.UtcNow;
        _providerUploadSampleBytes = 0;
        _providerDownloadSampleBytes = 0;
        _providerTrafficTimer.Tick += ProviderTrafficTick;
        _providerTrafficTimer.Start();
    }

    private void ProviderTrafficTick(object? sender, EventArgs args)
    {
        var now = DateTimeOffset.UtcNow;
        var elapsed = Math.Max(0.001, (now - _providerTrafficSampleAt).TotalSeconds);
        _providerUploadRate = _providerUploadSampleBytes / elapsed;
        _providerDownloadRate = _providerDownloadSampleBytes / elapsed;
        _providerTrafficSampleAt = now;
        _providerUploadSampleBytes = 0;
        _providerDownloadSampleBytes = 0;
        OnPropertyChanged(nameof(ProviderTrafficText));
        OnPropertyChanged(nameof(IsProviderTrafficVisible));
        OnPropertyChanged(nameof(ProviderTrafficDetails));
    }

    private void ClearProviderTraffic()
    {
        _providerTransfers.Clear();
        _providerUploadSampleBytes = 0;
        _providerDownloadSampleBytes = 0;
        _providerUploadRate = 0;
        _providerDownloadRate = 0;
        _providerTrafficTimer.Stop();
        _providerTrafficTimer.Tick -= ProviderTrafficTick;
        OnPropertyChanged(nameof(ProviderTrafficText));
        OnPropertyChanged(nameof(IsProviderTrafficVisible));
        OnPropertyChanged(nameof(ProviderTrafficDetails));
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

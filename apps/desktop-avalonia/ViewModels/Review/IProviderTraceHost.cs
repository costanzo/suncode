using SunCode.Sdk.Models;

namespace SunCode.Desktop.ViewModels;

// Session context the provider trace drawer needs beyond the shared host:
// which session and model are active, whether an async load is still current,
// and where to report the latest context-window usage.
internal interface IProviderTraceHost : IViewModelHost
{
    string? SelectedSessionId { get; }
    string? SelectedModelId { get; }
    bool IsSessionContextCurrent(string sessionId, long? loadVersion);
    void UpdateContextUsage(AgentUsage usage);
    Task<ProviderExchangesResult> GetProviderExchangesAsync(string sessionId) => Sdk!.GetProviderExchangesAsync(sessionId);
    Task<ProviderExchangeDetails> GetProviderExchangeAsync(string sessionId, string exchangeId) => Sdk!.GetProviderExchangeAsync(sessionId, exchangeId);
}

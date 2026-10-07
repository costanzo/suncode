using SunCode.Desktop.Models;
using SunCode.Desktop.ViewModels;
using SunCode.Sdk;
using SunCode.Sdk.Models;

namespace SunCode.Desktop.Tests;

// Host with no SDK connection, for exercising feature view models in isolation.
internal sealed class FakeViewModelHost : IProviderTraceHost
{
    public List<Exception> Errors { get; } = [];
    public List<string> Successes { get; } = [];
    public List<string> PresentationErrors { get; } = [];
    public List<bool> BusyChanges { get; } = [];
    public List<AgentUsage> Usages { get; } = [];
    public AgentSdk? Sdk => null;
    public ProjectItem? SelectedProject { get; set; }
    public bool EnsureSdk() => false;
    public Task<bool> EnsureSdkReadyAsync() => Task.FromResult(false);
    public string StatusText { get; set; } = string.Empty;
    public void ReportError(Exception exception) { Errors.Add(exception); StatusText = exception.Message; }
    public void ReportSuccess(string message) { Successes.Add(message); StatusText = message; }
    public void ReportPresentationError(string message) { PresentationErrors.Add(message); StatusText = message; }
    public void SetBusy(bool busy) => BusyChanges.Add(busy);
    public string? SelectedSessionId { get; set; }
    public string? SelectedModelId { get; set; }
    public bool IsSessionContextCurrent(string sessionId, long? loadVersion) => SelectedSessionId == sessionId;
    public void UpdateContextUsage(AgentUsage usage) => Usages.Add(usage);
}

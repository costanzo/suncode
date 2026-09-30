using SunCode.Desktop.Models;
using SunCode.Desktop.ViewModels;
using SunCode.Sdk;

namespace SunCode.Desktop.Tests;

// Host with no SDK connection, for exercising feature view models in isolation.
internal sealed class FakeViewModelHost : IViewModelHost
{
    public List<Exception> Errors { get; } = [];
    public List<string> Successes { get; } = [];
    public List<string> PresentationErrors { get; } = [];
    public List<bool> BusyChanges { get; } = [];
    public AgentSdk? Sdk => null;
    public ProjectItem? SelectedProject { get; set; }
    public bool EnsureSdk() => false;
    public Task<bool> EnsureSdkReadyAsync() => Task.FromResult(false);
    public void ReportError(Exception exception) => Errors.Add(exception);
    public void ReportSuccess(string message) => Successes.Add(message);
    public void ReportPresentationError(string message) => PresentationErrors.Add(message);
    public void SetBusy(bool busy) => BusyChanges.Add(busy);
}
